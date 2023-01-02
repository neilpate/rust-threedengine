use minifb::{Key, Scale, Window, WindowOptions};
use std::time::Instant;

use crate::raster::{draw_triangle, Point};

use crate::colour::*;

mod threed;

mod raster;

mod colour;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

struct Core {
    view_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    proj_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    _cam_pos: threed::vec3,
    light_dir: threed::vec3,
}

fn init() -> Core {
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

    let cam_pos = threed::vec3 {
        x: 0.,
        y: 5.,
        z: -20.,
    };

    let view_mat = threed::create_view_matrix(0., cam_pos);

    let light_dir = threed::vec3 {
        x: 0.,
        y: 10.,
        z: -10.,
    };

    Core {
        view_mat,
        proj_mat,
        _cam_pos: cam_pos,
        light_dir,
    }
}

fn main() {
    let core = init();

    let mut window = Window::new(
        "3D Renderer",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale: Scale::X1,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    // Limit to max ~100 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_millis(10)));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let cube_path = "c:\\temp\\cube.obj";
    let cube = threed::Object::create_from_file(cube_path.to_string()).unwrap();

    let teapot_path = "c:\\temp\\teapot.obj";
    let teapot = threed::Object::create_from_file(teapot_path.to_string()).unwrap();

    let objects = vec![cube, teapot];

    let mut prev = Instant::now();
    let mut count = 0;

    let mut rot_x = 0f32;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let len = HEIGHT * WIDTH;

        let fill_colour = Colour::new(59, 59, 59);

        buffer[0..len].fill(fill_colour.as_0rgb());

        let now = Instant::now();
        let delta_time = (now - prev).as_secs_f32();
        prev = now;
        let fps = 1. / delta_time;

        let degrees_per_second = 36.;

        rot_x += delta_time * degrees_per_second;

        let mut tris: Vec<(raster::Tri, threed::vec3)> = Vec::new();

        for object in &objects {
            let rot_x_mat = threed::create_x_rotation_matrix(rot_x);
            let rot_y_mat = threed::create_y_rotation_matrix(15.);
            let rot_z_mat = threed::create_z_rotation_matrix(-15.);
            let trans_mat = threed::create_translation_matrix(0., 0., -0.);

            for tri in &object.tris {
                let proc_tri =
                    process_tri(&core, tri, &rot_z_mat, &rot_y_mat, &rot_x_mat, &trans_mat);

                match proc_tri {
                    Some(tri2) => tris.push(tri2),
                    None => (),
                }
            }
        }

        let mut z_vals = Vec::new();
        for tri in &tris {
            let z = tri.0.p1.z + tri.0.p2.z + tri.0.p3.z;
            z_vals.push((z * 1000000.) as u32); //This weird multiplication is just to be able to sort by z
        }

        let mut indices = (0..tris.len()).collect::<Vec<_>>();
        indices.sort_by_key(|&i| z_vals[i]);
        indices.reverse();

        for index in 0..tris.len() {
            let tri = &tris[indices[index]];

            let colour = Colour::new(190, 255, 136);

            let colour = threed::calc_tri_illum(&core.light_dir, &tri.1, colour);

            //  if (index == 4) | (index == 5) {
            draw_triangle(&mut buffer, &tri.0, colour.as_0rgb());
            //  }
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        count += 1;
        if count > 100 {
            count = 0;
            println!("FPS: {fps}");
            let vis_tris = tris.len();
            println!("Visible tris: {vis_tris}");
        }
    }
}

fn process_tri(
    core: &Core,
    tri: &threed::Tri,
    rot_z_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_y_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_x_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    trans_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
) -> Option<(raster::Tri, threed::Vert)> {
    let mut tri = transform_tri(tri, rot_z_mat, rot_y_mat, rot_x_mat, trans_mat);

    let normal = threed::normal(&tri);

    if normal.z <= 0. {
        tri.v1 = threed::mult_vec3_mat4(tri.v1, &core.view_mat);
        tri.v1 = threed::mult_vec3_mat4(tri.v1, &core.proj_mat);

        tri.v2 = threed::mult_vec3_mat4(tri.v2, &core.view_mat);
        tri.v2 = threed::mult_vec3_mat4(tri.v2, &core.proj_mat);

        tri.v3 = threed::mult_vec3_mat4(tri.v3, &core.view_mat);
        tri.v3 = threed::mult_vec3_mat4(tri.v3, &core.proj_mat);

        tri.v1.x += 1.;
        tri.v1.x *= 0.5 * (WIDTH as f32);
        tri.v1.y += 1.;
        tri.v1.y *= 0.5 * (HEIGHT as f32);
        tri.v1.z += 1.;
        tri.v1.z *= 0.5;

        tri.v2.x += 1.;
        tri.v2.x *= 0.5 * (WIDTH as f32);
        tri.v2.y += 1.;
        tri.v2.y *= 0.5 * (HEIGHT as f32);
        tri.v2.z += 1.;
        tri.v2.z *= 0.5;

        tri.v3.x += 1.;
        tri.v3.x *= 0.5 * (WIDTH as f32);
        tri.v3.y += 1.;
        tri.v3.y *= 0.5 * (HEIGHT as f32);
        tri.v3.z += 1.;
        tri.v3.z *= 0.5;

        let p1 = Point {
            x: tri.v1.x.round() as u32,
            y: tri.v1.y.round() as u32,
            z: tri.v1.z,
        };

        let p2 = Point {
            x: tri.v2.x.round() as u32,
            y: tri.v2.y.round() as u32,
            z: tri.v2.z,
        };

        let p3 = Point {
            x: tri.v3.x.round() as u32,
            y: tri.v3.y.round() as u32,
            z: tri.v3.z,
        };

        Some((raster::Tri { p1, p2, p3 }, normal))
    } else {
        None
    }
}

fn transform_tri(
    tri: &threed::Tri,
    rot_z_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_y_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_x_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    trans_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
) -> threed::Tri {
    let mut v1 = threed::mult_vec3_mat4(tri.v1, rot_z_mat);
    v1 = threed::mult_vec3_mat4(v1, rot_y_mat);
    v1 = threed::mult_vec3_mat4(v1, rot_x_mat);
    v1 = threed::mult_vec3_mat4(v1, trans_mat);
    let mut v2 = threed::mult_vec3_mat4(tri.v2, rot_z_mat);
    v2 = threed::mult_vec3_mat4(v2, rot_y_mat);
    v2 = threed::mult_vec3_mat4(v2, rot_x_mat);
    v2 = threed::mult_vec3_mat4(v2, trans_mat);
    let mut v3 = threed::mult_vec3_mat4(tri.v3, rot_z_mat);
    v3 = threed::mult_vec3_mat4(v3, rot_y_mat);
    v3 = threed::mult_vec3_mat4(v3, rot_x_mat);
    v3 = threed::mult_vec3_mat4(v3, trans_mat);
    threed::Tri { v1, v2, v3 }
}
