use minifb::{Key, Scale, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::raster::{draw_horiz_line, draw_triangle, Point};

mod threed;

mod raster;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn main() {
    let _obj = threed::Object::create_from_file("c:\\temp\\cube.obj".to_string());

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

    //Main loop
    //Calculate x rot matrix
    //Calculate y rot matrix
    //Calculate z rot matrix
    //Calculate xyz trans matrix

    //Loop over all the triangles in the object
    //Do the mult_vec_matrix with the transform matrix
    //Do backface culling
    //For each point in the triangle do the mult_vec_matrix with the view matrix to get a new triangle

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
    let mut time = 0.0f32;

    //  let status_text = StatusText::new(WIDTH, HEIGHT, 2);

    let mut prev = Instant::now();
    let mut count = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let new_size = (window.get_size().0, window.get_size().1);
        if new_size != size {
            size = new_size;
            buffer.resize(size.0 * size.1, 0);
        }

        //  update_plasma(&mut buffer, time);
        //  draw_horiz_line(&mut buffer, 0, WIDTH as u32, 100);

        let p1 = Point { x: 50, y: 100 };
        let p2 = Point { x: 0, y: 0 };
        let p3 = Point { x: 100, y: 0 };
        let colour = 12345;

        draw_triangle(&mut buffer, p1, p2, p3, colour);

        let mut current = Instant::now();
        count += 1;

        if count == 1000 {
            count = 0;
            let delta_time = (current - prev).as_secs_f32();
            let frame_rate = 1. / delta_time;
            println!("{frame_rate:.1} FPS",);
        }

        prev = current;

        window
            .update_with_buffer(&buffer, new_size.0, new_size.1)
            .unwrap();

        time += 1.0 / 60.0;
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
