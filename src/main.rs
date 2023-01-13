// To Do
// [done] Move tests to separate files
// [done] Mouse object rotation
// [done] Mouse object translation
// [done] On screen text
// [done] Camera controls
// Mouse object selection
// Movable light source
// Orthographic camera
// Objectg colour change in real-time
// Alpha blending
// Move to EGUI?
// Add objects are runtime
// Object scaling
// Textures!

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};
use raster::draw_outlined_triangle;
use std::env;
use std::time::Instant;
use threed::*;

use crate::raster::{draw_filled_triangle, Point};

use crate::colour::*;

mod threed;

mod raster;

mod colour;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const NUM_PIXELS: usize = HEIGHT * WIDTH;

struct Stats {
    frame_rate: f32,
    trans_and_proj_time: f32,
    raster_time: f32,
    present_time: f32,
    vis_tris: usize,
}

struct Core {
    view_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    proj_mat: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<[usize; 2]>>,
    camera: Camera,
    light_dir: vec3,
    window: Window,
    pixel_buffer: Vec<u32>,
    objects: Vec<Object>,
    should_shutdown: bool,
    mouse_button_held: MouseButtonHeld,
    selected_object: usize,
    prev_mouse_pos: Option<(f32, f32)>,
    wireframe_enabled: bool,
    help_enabled: bool,
    stats_enabled: bool,
    stats: Stats,
}

struct _MousePos {
    x: f32,
    y: f32,
}

enum MouseButtonHeld {
    None,
    _Left,
    Middle,
    Right,
}

fn init() -> Core {
    let screen = Screen {
        width: 800,
        height: 600,
    };

    let cam_pos = vec3 {
        x: 0.,
        y: 5.,
        z: -20.,
    };

    let camera = Camera {
        fov: 60.,
        near_plane: 0.1,
        far_plane: 1000.,
        position: cam_pos,
        yaw: 0.,
    };

    let proj_mat = camera.create_projection_matrix(screen);

    let view_mat = camera.create_view_matrix();

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

    let pixel_buffer: Vec<u32> = vec![0; NUM_PIXELS];

    let mut objects = vec![
        init_cube(),
        init_teapot(0., 0., -8.),
        // init_spaceship(-5., 2., 5.),
    ];

    let floor = init_checkerboard_floor();

    for obj in floor {
        objects.push(obj);
    }

    let stats = Stats {
        frame_rate: 0.,
        trans_and_proj_time: 0.,
        raster_time: 0.,
        present_time: 0.,
        vis_tris: 0,
    };

    Core {
        view_mat,
        proj_mat,
        camera,
        light_dir,
        window,
        pixel_buffer,
        objects,
        should_shutdown: false,
        mouse_button_held: MouseButtonHeld::None,
        selected_object: 1,
        prev_mouse_pos: None,
        wireframe_enabled: false,
        help_enabled: false,
        stats_enabled: true,
        stats,
    }
}

fn main() {
    let mut core = init();
    main_loop(&mut core);
    if core.should_shutdown {
        return;
    }
}

fn handle_keys(core: &mut Core) {
    if core.window.is_key_down(Key::Escape) {
        core.should_shutdown = true;
    }

    if core.window.is_key_pressed(Key::L, KeyRepeat::No) {
        core.wireframe_enabled = !core.wireframe_enabled;
    }

    if core.window.is_key_pressed(Key::H, KeyRepeat::No) {
        core.help_enabled = !core.help_enabled;
    }

    if core.window.is_key_pressed(Key::P, KeyRepeat::No) {
        core.stats_enabled = !core.stats_enabled;
    }

    if core.window.is_key_pressed(Key::W, KeyRepeat::Yes) {
        core.camera.position.z = core.camera.position.z + 1.;
        core.view_mat = core.camera.create_view_matrix();
    }

    if core.window.is_key_pressed(Key::S, KeyRepeat::Yes) {
        core.camera.position.z = core.camera.position.z - 1.;
        core.view_mat = core.camera.create_view_matrix();
    }

    if core.window.is_key_pressed(Key::A, KeyRepeat::Yes) {
        core.camera.position.x = core.camera.position.x - 1.;
        core.view_mat = core.camera.create_view_matrix();
    }

    if core.window.is_key_pressed(Key::D, KeyRepeat::Yes) {
        core.camera.position.x = core.camera.position.x + 1.;
        core.view_mat = core.camera.create_view_matrix();
    }

    if core.window.is_key_pressed(Key::Left, KeyRepeat::Yes) {
        core.camera.yaw = core.camera.yaw + 5.;
        core.view_mat = core.camera.create_view_matrix();
    }

    if core.window.is_key_pressed(Key::Right, KeyRepeat::Yes) {
        core.camera.yaw = core.camera.yaw - 5.;
        core.view_mat = core.camera.create_view_matrix();
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
    let mut _rot_y = 0f32;

    let font_weight = FontWeight::Regular;
    let raster_height = RasterHeight::Size20;
    let fill_colour = Colour::new(59, 59, 59);

    loop {
        handle_keys(core);
        handle_mouse(core);
        if core.should_shutdown {
            return;
        }

        core.pixel_buffer[0..NUM_PIXELS].fill(fill_colour.as_0rgb());

        let now = Instant::now();
        let delta_time = (now - prev).as_secs_f32();
        prev = now;
        core.stats.frame_rate = 1. / delta_time;

        // let degrees_per_second = 36.;
        //rot_y += delta_time * degrees_per_second;

        let mut tris: Vec<(raster::Tri, vec3, Colour)> = Vec::new();

        //Start of Transform and project
        let trans_and_proj_time_start = Instant::now();

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

        let trans_and_proj_time_end = Instant::now();

        core.stats.trans_and_proj_time =
            (trans_and_proj_time_end - trans_and_proj_time_start).as_secs_f32();
        //End of Transform and Project

        //Start of Raster
        let raster_time_start = Instant::now();

        for index in 0..tris.len() {
            let tri = &tris[indices[index]];

            let colour = calc_tri_illum(&core.light_dir, &tri.1, tri.2);
            if core.wireframe_enabled {
                draw_outlined_triangle(&mut core.pixel_buffer, &tri.0, colour.as_0rgb());
            } else {
                draw_filled_triangle(&mut core.pixel_buffer, &tri.0, colour.as_0rgb());
            }
        }

        let raster_time_end = Instant::now();

        core.stats.raster_time = (raster_time_end - raster_time_start).as_secs_f32();
        //End of Raster

        core.stats.vis_tris = tris.len();

        if core.stats_enabled {
            draw_stats(core, font_weight, raster_height);
        };
        draw_help(core, font_weight, raster_height);

        //Start of Present
        let present_time_start = Instant::now();

        core.window
            .update_with_buffer(&core.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();

        let present_time_end = Instant::now();
        core.stats.present_time = (present_time_end - present_time_start).as_secs_f32();
        //End of Present
    }
}

fn draw_stats(core: &mut Core, font_weight: FontWeight, raster_height: RasterHeight) {
    let x_pos = 520;
    let frame_rate = core.stats.frame_rate;
    let msg = format!("Frame Rate       {frame_rate:.0} FPS");
    draw_string(msg.as_str(), x_pos, 0, font_weight, raster_height, core);

    let trans_and_proj_time_ms = core.stats.trans_and_proj_time * 1000.;
    let msg = format!("Trans. & Proj      {trans_and_proj_time_ms:.0} ms");
    draw_string(
        msg.as_str(),
        x_pos,
        raster_height as u32,
        font_weight,
        raster_height,
        core,
    );

    let raster_time_ms = core.stats.raster_time * 1000.;
    let msg = format!("Raster             {raster_time_ms:.0} ms");
    draw_string(
        msg.as_str(),
        x_pos,
        2 * raster_height as u32,
        font_weight,
        raster_height,
        core,
    );

    let present_time_ms = core.stats.present_time * 1000.;
    let msg = format!("Present            {present_time_ms:.0} us");
    draw_string(
        msg.as_str(),
        x_pos,
        3 * raster_height as u32,
        font_weight,
        raster_height,
        core,
    );

    let vis_tris = core.stats.vis_tris;
    let msg = format!("Visible tris.   {vis_tris}");
    draw_string(
        msg.as_str(),
        x_pos,
        4 * raster_height as u32,
        font_weight,
        raster_height,
        core,
    );
}

fn draw_help(core: &mut Core, font_weight: FontWeight, raster_height: RasterHeight) {
    let x_pos = 0;

    let msg: Vec<&str>;

    if core.help_enabled {
        msg = vec![
            "LMB   Select object",
            "RMB   Rotate object",
            "MMB   Pan object (XZ) plane",
            "Wheel Translate object (Y axis)",
            "-------------------------------",
            "W     Move Forwards",
            "A     Move Left",
            "S     Move Backwards",
            "D     Move Right",
            "<-    Yaw CCW",
            "->    Yaw CW",
            "-------------------------------",
            "H     Toggle Help",
            "L     Toggle Wireframe Mode",
            "P     Toggle Stats",
            "B     Toggle Back Face Culling",
        ];
    } else {
        msg = vec!["Press H to toggle Help"];
    }

    let mut i = 0;
    for msg in msg {
        draw_string(
            msg,
            x_pos,
            i * raster_height as u32,
            font_weight,
            raster_height,
            core,
        );
        i += 1;
    }
}

fn draw_string(
    msg: &str,
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
                let index = char_i * char_raster.width()
                    + col_i
                    + row_i * WIDTH
                    + (x as usize)
                    + (y as usize * WIDTH);

                let mut curr_pixel = Colour::from_u32(core.pixel_buffer[index]);

                curr_pixel.add_intensity(*intensity);

                core.pixel_buffer[index] = curr_pixel.as_0rgb();
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

fn _init_spaceship(x: f32, y: f32, z: f32) -> Object {
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
