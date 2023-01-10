use std::mem;

use crate::HEIGHT;
use crate::WIDTH;

#[derive(Debug)]
pub struct Tri {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
    pub z: f32,
}

fn two_d_to_1d(x: i32, y: i32) -> usize {
    (HEIGHT - (y as usize) - 1) * WIDTH + (x as usize)
}

/// This is an implementation of Bresenahms fast line drawing routine
pub fn draw_line(buffer: &mut Vec<u32>, x1: u32, y1: u32, x2: u32, y2: u32, colour: u32) {
    let mut x1 = x1 as i32;
    let mut y1 = y1 as i32;
    let x2 = x2 as i32;
    let y2 = y2 as i32;

    let sign_x: i32;
    let sign_y: i32;

    if x2 > x1 {
        sign_x = 1;
    } else {
        sign_x = -1;
    }

    if y2 > y1 {
        sign_y = 1;
    } else {
        sign_y = -1;
    }

    let dx = (x2 - x1).abs();
    let dy = -(y2 - y1).abs();

    let mut err = (dx + dy) as i32;
    let mut e2: i32;

    loop {
        let index = two_d_to_1d(x1, y1);
        buffer[index] = colour;

        if x1 == x2 && y1 == y2 {
            return;
        } else {
            e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x1 += sign_x;
            }

            if e2 <= dy {
                err += dx;
                y1 += sign_y;
            }
        }
    }
}

pub fn draw_horiz_line(buffer: &mut Vec<u32>, x1: u32, x2: u32, y: u32, colour: u32) {
    if check_bounds(x1, x2, y) {
        let y = y as usize;
        let y_offset = (HEIGHT - y - 1) * WIDTH;

        let range;
        //Note in the above range we must include the final value
        if x1 > x2 {
            range = x2 as usize..=x1 as usize;
        } else {
            range = x1 as usize..=x2 as usize;
        }

        // println!("Range: {range:?}");

        // This niaive way also seems to be fastest
        for i in range {
            let index = y_offset + i;

            if index < buffer.len() {
                buffer[y_offset + i] = colour;
            }
        }
    }

    //  buffer[range].fill(colour);
    // for i in &mut buffer[range] {
    //     *i = colour
    // }

    // for i in range {
    //     buffer[i] = colour;
    // }

    // Trying with map, it seems slower
    // let _dontcare: _ = (x1..x2)
    //     .map(|x| buffer[y_offset + (x as usize)] = colour)
    //     .collect::<()>();
}

/// Sort three points p1, p2, p3 such that the output is ordered by decreasing y
// Output is a tuple (pa, pb, pc) where
// pmax.y > pmid.y > pmin.y
fn sort_points_by_y(p1: Point, p2: Point, p3: Point) -> (Point, Point, Point) {
    let mut pmax = p1;
    let mut pmid = p2;
    let mut pmin = p3;

    if pmid.y > pmax.y {
        mem::swap(&mut pmax, &mut pmid);
    }

    if pmin.y > pmax.y {
        mem::swap(&mut pmax, &mut pmin);
    }
    // At this stage we have the largest point in pmax
    // Now figure out the mid and min
    if pmin.y > pmid.y {
        mem::swap(&mut pmid, &mut pmin);
    }

    (pmax, pmid, pmin)
}

pub fn draw_outlined_triangle(buffer: &mut Vec<u32>, tri: &Tri, colour: u32) {
    draw_line(buffer, tri.p1.x, tri.p1.y, tri.p2.x, tri.p2.y, colour);
    draw_line(buffer, tri.p2.x, tri.p2.y, tri.p3.x, tri.p3.y, colour);
    draw_line(buffer, tri.p3.x, tri.p3.y, tri.p1.x, tri.p1.y, colour);
}

/// Any triangle (p1, p2, p3) can be split into two further triangles, one with a flat bottom
/// and one with a flat top
/// Flat bottom: (p1, p2, p4)
/// Flat top: (p3, p2, p4)
///  +y
///   ^
///   |                       .  p1
///   |                  .       .     
///   |            .           .
///   |        .             .
///   |     p2--------------p4         
///   |         .        .
///   |           .   .
///   |            p3
///
/// /// (0,0)---------------------> +x
///
///
pub fn draw_filled_triangle(buffer: &mut Vec<u32>, tri: &Tri, colour: u32) {
    // println!("Drawing triangle: {tri:?}");

    // Goal is to calculate p4
    // Then draw the flat topped triangle and flat bottomed triangle

    // first sort the points so that p1.y > p2.y > p3.y

    let sorted_points = sort_points_by_y(tri.p1, tri.p2, tri.p3);
    // println!("Sorted points: {sorted_points:?}");

    // Now we need to find p4
    // Obviously it shares a y values with p2
    // And we can calculate the gradient p3-->p4
    // Then we can solve for p4.x

    let p4y = sorted_points.1.y;

    let num = (sorted_points.0.y as f32) - (sorted_points.2.y as f32);
    let denom = (sorted_points.0.x as f32) - (sorted_points.2.x as f32);

    let p4x;

    if denom == 0. {
        //The top and one of the bottom two points are in a vertical line, so the gradient is infinite
        //p4x has the same x value as the top point

        p4x = sorted_points.0.x;
    } else {
        let gradient_p3_p1 = num / denom;

        // y = mx + c
        // c = y - mx

        // Can use either p1 or p3 to calculate c, pick p1 for no good reason
        let c = (sorted_points.0.y as f32) - gradient_p3_p1 * (sorted_points.0.x as f32);

        // x = (y -c)/m

        p4x = (((p4y as f32) - c) / gradient_p3_p1).round() as u32;
    }

    draw_flat_bottom_triangle(buffer, sorted_points.0, sorted_points.1.x, p4x, p4y, colour);
    draw_flat_top_triangle(buffer, sorted_points.2, sorted_points.1.x, p4x, p4y, colour);
}

/// Draw a filled flat bottomed triangle by starting at the bottom
/// and drawing a horizontal line of decreasing width.
/// By definition p2.y == p3.y
/// And p1.y > p2.y
///  +y
///   ^
///   |            p1
///   |          /   \
///   |         /     \
///   |       p2-------p3
/// (0,0)---------------------> +x
///
fn draw_flat_bottom_triangle(
    buffer: &mut Vec<u32>,
    p1: Point,
    p2x: u32,
    p3x: u32,
    p23y: u32,
    colour: u32,
) {
    // println!("Drawing flat bottom triangle: p1:{p1:?}, p2x:{p2x}, p3x:{p3x}, p23y:{p23y}");

    // First calculate the inverse gradient of line p2 --> p1
    // Recall the gradient is Δy/Δx
    let mut num = (p2x as f32) - (p1.x as f32);
    let mut denom = (p23y as f32) - (p1.y as f32);
    let inverse_gradient_p2_p1 = num / denom;
    // The inverse gradient is more convenient when later calculating the horizontal line length

    // Then calculate the gradient of line p1 --> p3
    num = (p1.x as f32) - (p3x as f32);
    denom = (p1.y as f32) - (p23y as f32);
    let inverse_gradient_p1_p3 = num / denom;

    // The starting point is the bottom two points, p2 and p3
    let mut from = p2x as f32;
    let mut to = p3x as f32;

    // We know the triangle is flat bottom, so p1.y > p23y
    // Create the range of y values from p23y --> p1.y
    let range = (p23y)..(p1.y);

    // Loop over this range
    for y in range {
        // Drawing a horizontal line
        draw_horiz_line(buffer, from.round() as u32, to.round() as u32, y, colour);

        // Every iteration the horizontal line will get a bit shorter
        // as gradient_p2_p1 and gradient_p1_p3 are guaranteed to be opposite directions
        from += inverse_gradient_p2_p1;
        to += inverse_gradient_p1_p3;
    }
}

/// Draw a filled flat bottomed triangle by starting at the bottom
/// and drawing a horizontal line of decreasing width.
/// By definition p2.y == p3.y
/// And p1.y < p2.y
///  +y
///   ^
///   |       p2-------p3
///   |         \     /   
///   |          \   /     
///   |           p1
/// (0,0)---------------------> +x
///
fn draw_flat_top_triangle(
    buffer: &mut Vec<u32>,
    p1: Point,
    p2x: u32,
    p3x: u32,
    p23y: u32,
    colour: u32,
) {
    // println!("Drawing flat topped triangle: p1:{p1:?}, p2x:{p2x}, p3x:{p3x}, p23y:{p23y}");

    // First calculate the inverse gradient of line p2 --> p1
    // Recall the gradient is Δy/Δx
    let mut num = (p2x as f32) - (p1.x as f32);
    let mut denom = (p23y as f32) - (p1.y as f32);
    let inverse_gradient_p2_p1 = num / denom;
    // The inverse gradient is more convenient when later calculating the horizontal line length

    // Then calculate the gradient of line p1 --> p3
    num = (p1.x as f32) - (p3x as f32);
    denom = (p1.y as f32) - (p23y as f32);
    let inverse_gradient_p1_p3 = num / denom;

    // The starting point is the bottom points, p1
    let mut from = p1.x as f32;
    let mut to = p1.x as f32;

    // We know the triangle is flat top, so p1.y < p23y
    // Create the range of y values from p1.y --> p23y
    let range = (p1.y)..(p23y);

    // Loop over this range
    for y in range {
        // Drawing a horizontal line
        draw_horiz_line(buffer, from.round() as u32, to.round() as u32, y, colour);
        // Every iteration the horizontal line will get longer as it diverges from a single point of p1 --> p2 and p3
        // as gradient_p2_p1  and gradient_p1_p3 are guaranteed to be opposite signs
        from += inverse_gradient_p2_p1;
        to += inverse_gradient_p1_p3;
    }
}

fn check_bounds(from: u32, to: u32, y: u32) -> bool {
    if ((y as usize) < HEIGHT) & ((from as usize) < WIDTH) & ((to as usize) < WIDTH) {
        true
    } else {
        false
    }
}

#[test]
fn test_2d_to_1d_1() {
    let expected = HEIGHT * WIDTH - WIDTH;

    let result = two_d_to_1d(0, 0);

    assert_eq!(expected, result);
}

#[test]
fn test_sort_points_1() {
    let p1 = Point { x: 0, y: 0, z: 0. };
    let p2 = Point { x: 0, y: 1, z: 0. };
    let p3 = Point { x: 0, y: 2, z: 0. };

    let expected = (p3, p2, p1);

    let result = sort_points_by_y(p1, p2, p3);

    assert_eq!(expected, result);
}

#[test]
fn test_sort_points_2() {
    let p1 = Point {
        x: 0,
        y: 100,
        z: 0.,
    };
    let p2 = Point { x: 0, y: 50, z: 0. };
    let p3 = Point {
        x: 0,
        y: 200,
        z: 0.,
    };

    let expected = (p3, p1, p2);

    let result = sort_points_by_y(p1, p2, p3);

    assert_eq!(expected, result);
}
