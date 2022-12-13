use std::mem;

use crate::HEIGHT;
use crate::WIDTH;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub fn draw_horiz_line(buffer: &mut Vec<u32>, x1: u32, x2: u32, y: u32, colour: u32) {
    for xi in x1..x2 {
        buffer[(y as usize) * WIDTH + (xi as usize)] = colour;
    }
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
pub fn draw_triangle(buffer: &mut Vec<u32>, p1: Point, p2: Point, p3: Point, colour: u32) {
    // Calculate p4
    let p4 = Point { x: 0, y: 0 };

    // first sort the points so that p1.y > p2.y > p3.y

    draw_flat_bottom_triangle(buffer, p1, p2.x, p4.x, p2.y, colour);
    draw_flat_top_triangle(buffer, p3, p2.x, p4.x, p2.y, colour);
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
        draw_horiz_line(buffer, from as u32, to as u32, y, colour);

        // Every iteration the horizontal line will get a bit shorter
        // as gradient_p2_p1 is +ve
        // and gradient_p1_p3 is -ve
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
    let mut from = p2x as f32;
    let mut to = p3x as f32;

    // We know the triangle is flat top, so p1.y < p23y
    // Create the range of y values from p1.y --> p23y
    let range = (p1.y)..(p23y);

    // Loop over this range
    for y in range {
        // Drawing a horizontal line
        draw_horiz_line(buffer, from as u32, to as u32, y, colour);

        // Every iteration the horizontal line will get a bit shorter
        // as gradient_p2_p1 is +ve
        // and gradient_p1_p3 is -ve
        from += inverse_gradient_p2_p1;
        to += inverse_gradient_p1_p3;
    }
}

#[test]
fn test_sort_points_1() {
    let p1 = Point { x: 0, y: 0 };
    let p2 = Point { x: 0, y: 1 };
    let p3 = Point { x: 0, y: 2 };

    let expected = (p3, p2, p1);

    let result = sort_points_by_y(p1, p2, p3);

    assert_eq!(expected, result);
}

#[test]
fn test_sort_points_2() {
    let p1 = Point { x: 0, y: 100 };
    let p2 = Point { x: 0, y: 50 };
    let p3 = Point { x: 0, y: 200 };

    let expected = (p3, p1, p2);

    let result = sort_points_by_y(p1, p2, p3);

    assert_eq!(expected, result);
}
