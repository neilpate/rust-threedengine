// To Do
// Mouse object selection
// [done] Mouse object rotation
// [done] Mouse object translation
// Movable light source
// Orthographic camera
// Objectg colour change in real-time
// Camera controls
// Alpha blending
// Move to EGUI?
// Add objects are runtime
// Object scaling
// On screen text
// Textures!

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use std::env;
use std::time::Instant;
use threed::*;

use crate::raster::{draw_triangle, Point};

use crate::colour::*;

mod threed;

mod raster;

mod colour;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const NUM_PIXELS: usize = HEIGHT * WIDTH;

struct Core {
    view_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    proj_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    _cam_pos: vec3,
    light_dir: vec3,
    window: Window,
    pixel_buffer: Vec<u32>,
    objects: Vec<Object>,
    shutdown: bool,
    mouse_button_held: MouseButtonHeld,
    selected_object: usize,
    prev_mouse_pos: Option<(f32, f32)>,
}

struct MousePos {
    x: f32,
    y: f32,
}

enum MouseButtonHeld {
    None,
    Left,
    Middle,
    Right,
}

fn init() -> Core {
    let screen = Screen {
        width: 800,
        height: 600,
    };
    let camera = Camera {
        fov: 60.,
        near_plane: 0.1,
        far_plane: 1000.,
    };

    let proj_mat = create_projection_matrix(screen, camera);

    let cam_pos = vec3 {
        x: 0.,
        y: 5.,
        z: -20.,
    };

    let view_mat = create_view_matrix(0., cam_pos);

    let light_dir = vec3 {
        x: 0.,
        y: 10.,
        z: -10.,
    };

    let window = Window::new(
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

    let mut pixel_buffer: Vec<u32> = vec![0; NUM_PIXELS];

    let mut objects = vec![
        init_cube(),
        init_teapot(0., 0., -8.),
        // init_spaceship(-5., 2., 5.),
    ];

    let floor = init_checkerboard_floor();

    for obj in floor {
        objects.push(obj);
    }
    Core {
        view_mat,
        proj_mat,
        _cam_pos: cam_pos,
        light_dir,
        window,
        pixel_buffer,
        objects,
        shutdown: false,
        mouse_button_held: MouseButtonHeld::None,
        selected_object: 1,
        prev_mouse_pos: None,
    }
}

fn main() {
    let mut core = init();
    main_loop(&mut core);
    shutdown(core);
}

fn shutdown(core: Core) {}

fn handle_keys(core: &mut Core) {
    let keys = core.window.get_keys_pressed(KeyRepeat::Yes);

    if core.window.is_key_down(Key::Escape) {
        core.shutdown = true;
    }
}

fn handle_mouse(core: &mut Core) {
    if core.window.get_mouse_down(MouseButton::Middle) {
        core.mouse_button_held = MouseButtonHeld::Middle;
    } else if core.window.get_mouse_down(MouseButton::Right) {
        core.mouse_button_held = MouseButtonHeld::Right;
    } else {
        core.mouse_button_held = MouseButtonHeld::None;
    }

    match core.mouse_button_held {
        MouseButtonHeld::Middle => match core.prev_mouse_pos {
            Some(prev_pos) => {
                let curr = core.window.get_mouse_pos(MouseMode::Clamp).unwrap();
                let delta_x = prev_pos.0 - curr.0;
                let delta_y = prev_pos.1 - curr.1;

                core.objects[core.selected_object].transform.position.x -= delta_x / 3.;
                core.objects[core.selected_object].transform.position.z += delta_y / 3.;
                core.prev_mouse_pos = core.window.get_mouse_pos(MouseMode::Clamp);
            }
            None => {
                core.prev_mouse_pos = core.window.get_mouse_pos(MouseMode::Clamp);
            }
        },

        MouseButtonHeld::Right => match core.prev_mouse_pos {
            Some(prev_pos) => {
                let curr = core.window.get_mouse_pos(MouseMode::Clamp).unwrap();
                let delta_x = prev_pos.0 - curr.0;
                let delta_y = prev_pos.1 - curr.1;

                core.objects[core.selected_object].transform.rotation.y -= delta_x;
                core.objects[core.selected_object].transform.rotation.z += delta_y;
                core.prev_mouse_pos = core.window.get_mouse_pos(MouseMode::Clamp);
            }
            None => {
                core.prev_mouse_pos = core.window.get_mouse_pos(MouseMode::Clamp);
            }
        },
        _ => core.prev_mouse_pos = None,
    }

    let delta_y = core.window.get_scroll_wheel();

    match delta_y {
        Some(val) => core.objects[core.selected_object].transform.position.y += val.1 / 20.,
        None => {}
    };
}

fn main_loop(core: &mut Core) {
    let mut prev = Instant::now();
    let mut count = 0;
    let mut rot_y = 0f32;

    let font_weight = FontWeight::Regular;
    let raster_height = RasterHeight::Size16;
    // let buffer_height = raster_height.val();
    let char_width = get_raster_width(font_weight, raster_height);
    //let buffer_width = char_width * msg.chars().count();

    loop {
        handle_keys(core);
        handle_mouse(core);
        if core.shutdown {
            return;
        }
        let fill_colour = Colour::new(59, 59, 59);

        core.pixel_buffer[0..NUM_PIXELS].fill(fill_colour.as_0rgb());

        let now = Instant::now();
        let delta_time = (now - prev).as_secs_f32();
        prev = now;
        let fps = 1. / delta_time;

        let degrees_per_second = 36.;

        rot_y += delta_time * degrees_per_second;

        let mut tris: Vec<(raster::Tri, vec3, Colour)> = Vec::new();

        for object in &core.objects {
            let rot_x_mat = create_x_rotation_matrix(object.transform.rotation.x);
            let rot_y_mat = create_y_rotation_matrix(object.transform.rotation.y);
            let rot_z_mat = create_z_rotation_matrix(object.transform.rotation.z);
            let trans_mat = create_translation_matrix(
                object.transform.position.x,
                object.transform.position.y,
                object.transform.position.z,
            );

            for tri in &object.tris {
                let proc_tri = process_tri(
                    &core,
                    tri,
                    &rot_z_mat,
                    &rot_y_mat,
                    &rot_x_mat,
                    &trans_mat,
                    object.albedo,
                );

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

            let colour = calc_tri_illum(&core.light_dir, &tri.1, tri.2);

            //  if (index == 4) | (index == 5) {
            draw_triangle(&mut core.pixel_buffer, &tri.0, colour.as_0rgb());
            //  }
        }

        let vis_tris = tris.len();

        draw_stats(fps, vis_tris, font_weight, raster_height, core);

        core.window
            .update_with_buffer(&core.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();

        count += 1;
        if count > 100 {
            count = 0;
            println!("FPS: {fps:.0}");
            let vis_tris = tris.len();
            println!("Visible tris: {vis_tris}");
        }
    }
}

fn draw_stats(
    fps: f32,
    vis_tris: usize,
    font_weight: FontWeight,
    raster_height: RasterHeight,
    core: &mut Core,
) {
    let stats_x_pos = 520;
    let mut msg = format!("Frame Rate       {fps:.0} FPS");
    draw_string(msg, stats_x_pos, 0, font_weight, raster_height, core);
    let mut msg = format!("Trans. & Proj           ");
    draw_string(
        msg,
        stats_x_pos,
        raster_height as u32,
        font_weight,
        raster_height,
        core,
    );
    let mut msg = format!("Raster                  ");
    draw_string(
        msg,
        stats_x_pos,
        2 * raster_height as u32,
        font_weight,
        raster_height,
        core,
    );
    let mut msg = format!("Present                 ");
    draw_string(
        msg,
        stats_x_pos,
        3 * raster_height as u32,
        font_weight,
        raster_height,
        core,
    );
    let mut msg = format!("Visible tris.   {vis_tris}    ");
    draw_string(
        msg,
        stats_x_pos,
        4 * raster_height as u32,
        font_weight,
        raster_height,
        core,
    );
}

fn draw_string(
    msg: String,
    x: u32,
    y: u32,
    font_weight: FontWeight,
    raster_height: RasterHeight,
    core: &mut Core,
) {
    for (char_i, char) in msg.chars().enumerate() {
        let char_raster = get_raster(char, font_weight, raster_height).expect("unknown char");
        for (row_i, row) in char_raster.raster().iter().enumerate() {
            for (col_i, intensity) in row.iter().enumerate() {
                let (r, g, b) = (*intensity as u32, *intensity as u32, *intensity as u32);
                let (r, g, b) = (255 - r, 255 - g, 255 - b);
                let rgb_32 = /*0 << 24 | */r << 16 | g << 8 | b;

                let index = char_i * char_raster.width()
                    + col_i
                    + row_i * WIDTH
                    + (x as usize)
                    + (y as usize * WIDTH);

                core.pixel_buffer[index] = rgb_32;
            }
        }
    }
}

fn model_path(model_name: String) -> String {
    let curr_dir = env::current_dir().unwrap();
    let mut path = curr_dir.join("Resource\\Models");
    path = path.join(model_name);
    let path_str = path.into_os_string().to_str().unwrap().to_string();
    println!("Model path: {path_str}");
    path_str
}

fn init_checkerboard_floor() -> Vec<Object> {
    let mut objs = Vec::new();

    let model_path = model_path("Plane 1m.obj".to_string());

    let rotation = vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };

    let num = 20;
    let num_div2 = (num as f32) / 2.;

    for z in 0..num {
        let z_f32 = z as f32;
        for x in 0..num {
            let x_f32 = x as f32;
            let position = vec3 {
                x: x_f32 - num_div2,
                y: 0.,
                z: z_f32 - num_div2,
            };

            let transform = Transform { position, rotation };

            let colour = (x_f32 + z_f32) % 2.;

            let mut albedo = Colour::new(255, 255, 255);
            if colour == 0. {
            } else {
                albedo.r = 0;
                albedo.g = 0;
                albedo.b = 0;
            }
            let obj =
                Object::create_from_file("cube".to_string(), model_path.clone(), transform, albedo)
                    .unwrap();
            objs.push(obj)
        }
    }

    objs
}
// let transform = Transform { position, rotation };
// // Object::create_from_file("cube".to_string(), path.to_string(), transform, albedo).unwrap()

fn init_cube() -> Object {
    let model_path = model_path("cube.obj".to_string());

    let position = vec3 {
        x: 3.,
        y: 3.,
        z: 3.,
    };
    let rotation = vec3 {
        x: 45.,
        y: 45.,
        z: 45.,
    };
    let transform = Transform { position, rotation };
    let albedo = Colour::new(42, 170, 255);
    // Object::create_from_file("cube".to_string(), path.to_string(), transform, albedo).unwrap()
    Object::create_from_file("cube".to_string(), model_path, transform, albedo).unwrap()
}

fn init_teapot(x: f32, y: f32, z: f32) -> Object {
    let model_path = model_path("teapot.obj".to_string());

    let position = vec3 { x, y, z };
    let rotation = vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let transform = Transform { position, rotation };
    let albedo = Colour::new(1, 204, 3);
    Object::create_from_file("teapot".to_string(), model_path, transform, albedo).unwrap()
}

fn init_spaceship(x: f32, y: f32, z: f32) -> Object {
    let model_path = model_path("spaceship.obj".to_string());

    let position = vec3 { x, y, z };
    let rotation = vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let transform = Transform { position, rotation };
    let albedo = Colour::new(1, 204, 3);
    Object::create_from_file("teapot".to_string(), model_path, transform, albedo).unwrap()
}

fn process_tri(
    core: &Core,
    tri: &Tri,
    rot_z_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_y_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_x_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    trans_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    albedo: Colour,
) -> Option<(raster::Tri, Vert, Colour)> {
    let mut tri = transform_tri(tri, rot_z_mat, rot_y_mat, rot_x_mat, trans_mat);

    let normal = normal(&tri);

    if normal.z <= 0. {
        tri.v1 = mult_vec3_mat4(tri.v1, &core.view_mat);
        tri.v1 = mult_vec3_mat4(tri.v1, &core.proj_mat);

        tri.v2 = mult_vec3_mat4(tri.v2, &core.view_mat);
        tri.v2 = mult_vec3_mat4(tri.v2, &core.proj_mat);

        tri.v3 = mult_vec3_mat4(tri.v3, &core.view_mat);
        tri.v3 = mult_vec3_mat4(tri.v3, &core.proj_mat);

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

        Some((raster::Tri { p1, p2, p3 }, normal, albedo))
    } else {
        None
    }
}

fn transform_tri(
    tri: &Tri,
    rot_z_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_y_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    rot_x_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    trans_mat: &ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
) -> Tri {
    let mut v1 = mult_vec3_mat4(tri.v1, rot_z_mat);
    v1 = mult_vec3_mat4(v1, rot_y_mat);
    v1 = mult_vec3_mat4(v1, rot_x_mat);
    v1 = mult_vec3_mat4(v1, trans_mat);
    let mut v2 = mult_vec3_mat4(tri.v2, rot_z_mat);
    v2 = mult_vec3_mat4(v2, rot_y_mat);
    v2 = mult_vec3_mat4(v2, rot_x_mat);
    v2 = mult_vec3_mat4(v2, trans_mat);
    let mut v3 = mult_vec3_mat4(tri.v3, rot_z_mat);
    v3 = mult_vec3_mat4(v3, rot_y_mat);
    v3 = mult_vec3_mat4(v3, rot_x_mat);
    v3 = mult_vec3_mat4(v3, trans_mat);
    Tri { v1, v2, v3 }
}
