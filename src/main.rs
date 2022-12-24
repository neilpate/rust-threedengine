use minifb::{Key, Scale, Window, WindowOptions};
use std::time::Instant;

use crate::raster::{draw_triangle, Point};

mod threed;

mod raster;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

struct Core {
    view_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    proj_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    _cam_pos: threed::vec3,
}

fn main() {
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

    let core = Core {
        view_mat,
        proj_mat,
        _cam_pos: cam_pos,
    };

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

    // Limit to max ~60 fps update rate
    //  window.limit_update_rate(Some(std::time::Duration::from_millis(10)));

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let object_path = "c:\\temp\\cube.obj";
    // let object_path = "c:\\temp\\teapot.obj";
    let cube = threed::Object::create_from_file(object_path.to_string()).unwrap();
    //   println!("{cube:?}");

    let mut prev = Instant::now();
    let mut count = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        count += 1;
        let now = Instant::now();
        let fps = 1. / (now - prev).as_secs_f32();

        if count > 100 {
            count = 0;
            println!("FPS: {fps}");
        }

        prev = now;

        let rot_x_mat = threed::create_x_rotation_matrix(-25.);
        let rot_y_mat = threed::create_y_rotation_matrix(50.);
        let rot_z_mat = threed::create_z_rotation_matrix(80.);
        let trans_mat = threed::create_translation_matrix(0., 0., 0.);

        let mut tris: Vec<(raster::Tri, threed::vec3)> = Vec::new();
        //  let mut new_obj = raster::Object::new(tris);

        //for i in 0..1 {
        //    let tri = &cube.tris[i];
        for tri in &cube.tris {
            let proc_tri = process_tri(&core, tri, &rot_z_mat, &rot_y_mat, &rot_x_mat, &trans_mat);

            match proc_tri {
                Some(tri) => tris.push(tri),
                None => (),
            }
        }

        for tri in tris {
            let light_dir = threed::vec3 {
                x: 1.,
                y: 2.,
                z: 3.,
            };

            let albedo_r = 190u32;
            let albedo_g = 255u32;
            let albedo_b = 136u32;

            //Packing goes BGRA
            let albedo = (albedo_b << 24) + (albedo_g << 16) + (albedo_r << 8);
            let colour = threed::calc_tri_illum(light_dir, tri.1, albedo);

            draw_triangle(&mut buffer, tri.0, colour);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
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

        tri.v2.x += 1.;
        tri.v2.x *= 0.5 * (WIDTH as f32);
        tri.v2.y += 1.;
        tri.v2.y *= 0.5 * (HEIGHT as f32);

        tri.v3.x += 1.;
        tri.v3.x *= 0.5 * (WIDTH as f32);
        tri.v3.y += 1.;
        tri.v3.y *= 0.5 * (HEIGHT as f32);

        let p1 = Point {
            x: tri.v1.x as u32,
            y: tri.v1.y as u32,
            z: tri.v1.z as i32,
        };

        let p2 = Point {
            x: tri.v2.x as u32,
            y: tri.v2.y as u32,
            z: tri.v2.z as i32,
        };

        let p3 = Point {
            x: tri.v3.x as u32,
            y: tri.v3.y as u32,
            z: tri.v3.z as i32,
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
