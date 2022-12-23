use minifb::{Key, Scale, Window, WindowOptions};
use std::time::Instant;

use crate::raster::{draw_triangle, Point};

mod threed;

mod raster;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let cube = threed::Object::create_from_file("c:\\temp\\cube.obj".to_string()).unwrap();

    println!("{cube:?}");

    let screen = threed::Screen {
        width: 800,
        height: 600,
    };
    let camera = threed::Camera {
        fov: 60.,
        near_plane: 0.1,
        far_plane: 1000.,
    };

    let proj_mat = threed::create_projection_matrix(screen, camera);
    println!("Projection matrix:");
    println!("{}", proj_mat);

    let cam_pos = threed::vec3 {
        x: 0.,
        y: 5.,
        z: -20.,
    };

    let view_mat = threed::create_view_matrix(0., cam_pos);
    println!("View matrix:");
    println!("{}", view_mat);

    let mut window = Window::new(
        "Plasma + Text Example",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_millis(10)));

    let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);

    let mut size = (0, 0);

    let mut _prev = Instant::now();
    let mut _count = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_size = (window.get_size().0, window.get_size().1);
        if new_size != size {
            size = new_size;
            buffer.resize(size.0 * size.1, 0);
        }

        let rot_x_mat = threed::create_x_rotation_matrix(0.);
        let rot_y_mat = threed::create_y_rotation_matrix(0.);
        let rot_z_mat = threed::create_z_rotation_matrix(0.);

        let trans_mat = threed::create_translation_matrix(0., 0., 0.);

        let tris: Vec<raster::Tri> = Vec::new();
        let mut new_obj = raster::Object::new(tris);

        for tri in &cube.tris {
            let mut v1 = threed::mult_vec3_mat4(tri.v1, &rot_z_mat);
            v1 = threed::mult_vec3_mat4(v1, &rot_y_mat);
            v1 = threed::mult_vec3_mat4(v1, &rot_x_mat);
            v1 = threed::mult_vec3_mat4(v1, &trans_mat);

            let mut v2 = threed::mult_vec3_mat4(tri.v2, &rot_z_mat);
            v2 = threed::mult_vec3_mat4(v2, &rot_y_mat);
            v2 = threed::mult_vec3_mat4(v2, &rot_x_mat);
            v2 = threed::mult_vec3_mat4(v2, &trans_mat);

            let mut v3 = threed::mult_vec3_mat4(tri.v3, &rot_z_mat);
            v3 = threed::mult_vec3_mat4(v3, &rot_z_mat);
            v3 = threed::mult_vec3_mat4(v3, &rot_y_mat);
            v3 = threed::mult_vec3_mat4(v3, &rot_x_mat);
            v3 = threed::mult_vec3_mat4(v3, &trans_mat);

            let normal = threed::normal(&tri.v1, &tri.v2, &tri.v3);

            if normal.z <= 0. {
                v1 = threed::mult_vec3_mat4(v1, &view_mat);
                v1 = threed::mult_vec3_mat4(v1, &proj_mat);
                v2 = threed::mult_vec3_mat4(v2, &view_mat);
                v2 = threed::mult_vec3_mat4(v2, &proj_mat);
                v3 = threed::mult_vec3_mat4(v3, &view_mat);
                v3 = threed::mult_vec3_mat4(v3, &proj_mat);

                v1.x += 1.;
                v1.x *= 0.5 * (WIDTH as f32);
                v1.y += 1.;
                v1.y *= 0.5 * (HEIGHT as f32);

                v2.x += 1.;
                v2.x *= 0.5 * (WIDTH as f32);
                v2.y += 1.;
                v2.y *= 0.5 * (HEIGHT as f32);

                v3.x += 1.;
                v3.x *= 0.5 * (WIDTH as f32);
                v3.y += 1.;
                v3.y *= 0.5 * (HEIGHT as f32);

                let p1 = Point {
                    x: v1.x as u32,
                    y: v1.y as u32,
                    z: v1.z as i32,
                };

                let p2 = Point {
                    x: v2.x as u32,
                    y: v2.y as u32,
                    z: v2.z as i32,
                };

                let p3 = Point {
                    x: v3.x as u32,
                    y: v3.y as u32,
                    z: v3.z as i32,
                };

                let tri = raster::Tri { p1, p2, p3 };

                new_obj.tris.push(tri);
            }
        }

        for tri in new_obj.tris {
            draw_triangle(&mut buffer, tri.p1, tri.p2, tri.p3, 123456);
        }

        window
            .update_with_buffer(&buffer, new_size.0, new_size.1)
            .unwrap();
    }
}
