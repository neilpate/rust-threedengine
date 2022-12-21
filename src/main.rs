use minifb::{Key, Scale, Window, WindowOptions};
use std::{
    fmt::Error,
    time::{Duration, Instant},
};

use crate::{
    raster::{draw_horiz_line, draw_triangle, Point},
    threed::Tri,
};

use rand::Rng;

mod threed;

mod raster;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let mut rng = rand::thread_rng();

    let mut cube = threed::Object::create_from_file("c:\\temp\\cube.obj".to_string()).unwrap();

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

        let new = cube.tris.into_iter().map(|tri| {

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
            };

            let p2 = Point {
                x: v2.x as u32,
                y: v2.y as u32,
            };

            let p3 = Point {
                x: v3.x as u32,
                y: v3.y as u32,
            };
            Some((p1, p2, p3))

        }
        else{
            None
        }


         }).collect::<Vec<_>>();

    //        draw_triangle(&mut buffer, p1, p2, p3, 123456);

        //  update_plasma(&mut buffer, time);
        //  draw_horiz_line(&mut buffer, 0, WIDTH as u32, 100);

        // let p1 = Point { x: 200, y: 200 };
        // let p2 = Point { x: 50, y: 150 };
        // let p3 = Point { x: 150, y: 50 };
        // let colour = 12345;

        let number_of_triangles = 0;
        let before = Instant::now();
        let average_line_length = 100;
        for _i in 0..number_of_triangles {
            let p1 = Point {
                x: rng.gen_range(0..average_line_length),
                y: rng.gen_range(0..average_line_length),
            };

            // let p2 = Point {
            //     x: rng.gen_range(0..(WIDTH as u32)),
            //     y: rng.gen_range(0..(HEIGHT as u32)),
            // };

            let p2 = Point {
                x: rng.gen_range(0..average_line_length),
                y: rng.gen_range(0..average_line_length),
            };

            let p3 = Point {
                x: rng.gen_range(0..average_line_length),
                y: rng.gen_range(0..average_line_length),
            };

            let colour = rng.gen_range(0..10000000);

            draw_triangle(&mut buffer, p1, p2, p3, colour);
        }

        let mut current = Instant::now();
        _count += 1;

        // if count == 1000 {
        //     count = 0;
        //     let delta_time = (current - prev).as_secs_f32();
        //     let frame_rate = 1. / delta_time;
        //     println!("{frame_rate:.1} FPS",);
        // }
        // let time_to_draw = (Instant::now() - before).as_millis();

        // println!("Time to draw {number_of_triangles} triangles: {time_to_draw} ms");

        _prev = current;

        window
            .update_with_buffer(&buffer, new_size.0, new_size.1)
            .unwrap();

    }
}

fn update_plasma(buffer: &mut Vec<u32>, time: f32) {
    let y_step = 1.0 / HEIGHT as f32;
    let x_step = 1.0 / WIDTH as f32;
    let mut y = 0.0;
    for yi in 0..HEIGHT {
        let buf = &mut buffer[yi * WIDTH..(yi + 1) * WIDTH];
        let mut x = 0.0f32;

        // So this code is really slow, but good enough as example :)
        for xi in 0..WIDTH {
            let k = 0.1 + (y + (0.148 - time).sin()).cos() + 2.4 * time;
            let w = 0.9 + (x + (0.628 + time).cos()).cos() - 0.7 * time;
            let d = (x * x + y * y).sqrt();
            let s = 7.0 * (d + w).cos() * (k + w).sin();
            let r = ((s + 0.2).cos() * 255.0) as u32;
            let g = ((s + 0.5).cos() * 255.0) as u32;
            let b = ((s + 0.7).cos() * 255.0) as u32;
            buf[xi] = (r << 16) | (g << 8) | b;

            x += x_step;
        }

        y += y_step;
    }
}
