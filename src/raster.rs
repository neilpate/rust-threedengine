use crate::HEIGHT;
use crate::WIDTH;

pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub fn draw_horiz_line(buffer: &mut Vec<u32>, x1: u32, x2: u32, y: u32, colour: u32) {
    for xi in x1..x2 {
        buffer[(y as usize) * WIDTH + (xi as usize)] = colour;
    }
}

pub fn draw_triangle(buffer: &mut Vec<u32>, p1: Point, p2: Point, p3: Point, colour: u32) {
    draw_flat_bottom_triangle(buffer, p1, p2, p3, colour);
}

fn draw_flat_bottom_triangle(buffer: &mut Vec<u32>, p1: Point, p2: Point, p3: Point, colour: u32) {
    let mut num = (p2.x as f32) - (p1.x as f32);
    let mut denom = (p2.y as f32) - (p1.y as f32);
    let invslope1 = num / denom;

    num = ((p3.x as f32) - (p1.x as f32));
    denom = ((p3.y as f32) - (p1.y as f32));
    let invslope2 = num / denom;

    let mut curr_x1 = p2.x as f32;
    let mut curr_x2 = p3.x as f32;

    let range;
    if p2.y > p1.y {
        range = (p1.y)..(p2.y);
    } else {
        range = (p2.y)..(p1.y);
    }

    for y in range {
        draw_horiz_line(buffer, curr_x1 as u32, curr_x2 as u32, y, colour);
        curr_x1 += invslope1;
        curr_x2 += invslope2;
    }
}
