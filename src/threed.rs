use std::fs;
use std::io;
use std::ops::{Add, Sub};

use ndarray::arr2;
use ndarray::prelude::*;
use ndarray::Array;

use crate::colour::*;

use float_eq::{derive_float_eq, float_eq};

pub struct Transform {
    pub position: vec3,
    pub rotation: vec3,
}

//#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub tris: Vec<Tri>,
    pub transform: Transform,
    pub albedo: Colour,
}

impl Object {
    pub fn _new(name: String, tris: Vec<Tri>, transform: Transform, albedo: Colour) -> Self {
        Self {
            name,
            tris,
            transform,
            albedo,
        }
    }
}

pub struct Screen {
    pub width: i32,
    pub height: i32,
}

pub struct Camera {
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub yaw: f32,
    pub position: vec3,
    pub fwd_speed: f32,
    pub yaw_speed: f32,
}

impl Camera {
    pub fn create_view_matrix(&self) -> Array2<f32> {
        let rot_mat = create_y_rotation_matrix(self.yaw);

        let target = vec3 {
            x: 0.,
            y: 0.,
            z: 1.,
        };
        let mut target_vert = mult_vec3_mat4(target, &rot_mat);

        target_vert = target_vert + self.position;

        let up = vec3 {
            x: 0.,
            y: 1.,
            z: 0.,
        };

        let point_at = point_at(self.position, target_vert, up);

        let vm = quick_invert_mat4(point_at);
        vm
    }

    pub fn create_projection_matrix(&self, screen: Screen) -> Array<f32, Ix2> {
        let afq = self.calc_afq(&screen);

        let mut m = arr2(&[
            [0., 0., 0., 0.],
            [0., 0., 0., 0.],
            [0., 0., 0., 1.],
            [0., 0., 0., 0.],
        ]);

        m[[0, 0]] = afq.aspect_ratio * afq.fov;
        m[[1, 1]] = afq.fov;
        m[[2, 2]] = afq.q;
        m[[3, 2]] = -1. * afq.q * self.near_plane;

        m
    }

    fn calc_afq(&self, screen: &Screen) -> AFQ {
        let aspect_ratio = (screen.height as f32) / (screen.width as f32);

        let fov = 1. / ((self.fov / 2.).to_radians().tan());

        let q = self.far_plane / (self.far_plane - self.near_plane);
        AFQ {
            aspect_ratio,
            fov,
            q,
        }
    }

    pub fn move_forwards(&mut self) {
        let rot_rad = self.yaw.to_radians();
        let (sin, cos) = rot_rad.sin_cos();
        let new_fwd = self.position.z + self.fwd_speed * cos;
        let new_left = self.position.x - self.fwd_speed * sin;

        self.position.z = new_fwd;
        self.position.x = new_left;
    }

    pub fn move_backwards(&mut self) {
        let rot_rad = self.yaw.to_radians();
        let (sin, cos) = rot_rad.sin_cos();
        let new_fwd = self.position.z - self.fwd_speed * cos;
        let new_left = self.position.x + self.fwd_speed * sin;

        self.position.z = new_fwd;
        self.position.x = new_left;
    }

    pub fn move_left(&mut self) {
        let rot_rad = self.yaw.to_radians();
        let (sin, cos) = rot_rad.sin_cos();
        let new_fwd = self.position.z - self.fwd_speed * sin;
        let new_left = self.position.x - self.fwd_speed * cos;

        self.position.z = new_fwd;
        self.position.x = new_left;
    }

    pub fn move_right(&mut self) {
        let rot_rad = self.yaw.to_radians();
        let (sin, cos) = rot_rad.sin_cos();
        let new_fwd = self.position.z + self.fwd_speed * sin;
        let new_left = self.position.x + self.fwd_speed * cos;

        self.position.z = new_fwd;
        self.position.x = new_left;
    }
}

#[derive_float_eq(
    ulps_tol = "AFQUlps",
    ulps_tol_derive = "Clone, Copy, Debug, PartialEq",
    debug_ulps_diff = "AFQDebugUlpsDiff",
    debug_ulps_diff_derive = "Clone, Copy, Debug, PartialEq",
    all_tol = "f32"
)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct AFQ {
    aspect_ratio: f32,
    fov: f32,
    q: f32,
}

#[derive_float_eq(
    ulps_tol = "VertUlps",
    ulps_tol_derive = "Clone, Copy, Debug, PartialEq",
    debug_ulps_diff = "VertDebugUlpsDiff",
    debug_ulps_diff_derive = "Clone, Copy, Debug, PartialEq",
    all_tol = "f32"
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vert {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vert {
    pub fn from_string(s: String) -> Option<Vert> {
        //     println!("Vert from string: {s}");

        let chunks: Vec<&str> = s.split(" ").collect();

        if chunks.len() == 4 {
            if chunks[0] == "v" {
                let x: f32 = chunks[1].parse().unwrap();
                let y: f32 = chunks[2].parse().unwrap();
                let z: f32 = chunks[3].parse().unwrap();
                return Some(Vert { x, y, z });
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

impl Add for Vert {
    type Output = Vert;

    fn add(self, other: Vert) -> Vert {
        Vert {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vert {
    type Output = Vert;

    fn sub(self, other: Vert) -> Vert {
        Vert {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[derive(Debug)]
pub struct Tri {
    pub v1: Vert,
    pub v2: Vert,
    pub v3: Vert,
}

impl Object {
    fn face_from_string(s: String) -> Option<(usize, usize, usize)> {
        // println!("Face from string: {s}");
        let chunks: Vec<&str> = s.split(" ").collect();

        if chunks.len() == 4 {
            if chunks[0] == "f" {
                let v1_index: usize = chunks[1].parse().unwrap();
                let v2_index: usize = chunks[2].parse().unwrap();
                let v3_index: usize = chunks[3].parse().unwrap();
                Some((v1_index, v2_index, v3_index))
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn create_from_file(
        name: String,
        obj_path: String,
        transform: Transform,
        albedo: Colour,
    ) -> Result<Object, io::Error> {
        let content = fs::read_to_string(obj_path)?;
        //   println!("{content}");
        let lines: Vec<&str> = content.split("\n").collect();

        let mut verts: Vec<Vert> = Vec::new();
        let mut faces: Vec<(usize, usize, usize)> = Vec::new();

        for line in lines {
            let vert = Vert::from_string(line.to_string());
            match vert {
                Some(v) => verts.push(v), //Vertex was ok
                None => (),               //Not a valid vertex, just ignore
            }

            let face = Object::face_from_string(line.to_string());
            match face {
                Some(f) => faces.push(f), //Vertex was ok
                None => (),               //Not a valid face, just ignore
            }
        }

        let mut mesh: Vec<Tri> = Vec::new();
        for face in faces {
            let v1 = verts[face.0 - 1];
            let v2 = verts[face.1 - 1];
            let v3 = verts[face.2 - 1];

            mesh.push(Tri { v1, v2, v3 });
        }

        Ok(Object {
            name,
            tris: mesh,
            transform,
            albedo,
        })
    }
}

pub use Vert as vec3;

pub fn create_x_rotation_matrix(angle_deg: f32) -> Array2<f32> {
    let mut m = Array::<f32, _>::eye(4);

    let angle_rad = angle_deg.to_radians();
    let (sin, cos) = angle_rad.sin_cos();

    m[[1, 1]] = cos;
    m[[1, 2]] = sin;
    m[[2, 1]] = -sin;
    m[[2, 2]] = cos;
    m
}

pub fn create_y_rotation_matrix(angle_deg: f32) -> Array2<f32> {
    let mut m = Array::<f32, _>::eye(4);

    let angle_rad = angle_deg.to_radians();
    let (sin, cos) = angle_rad.sin_cos();

    m[[0, 0]] = cos;
    m[[0, 2]] = sin;
    m[[2, 0]] = -sin;
    m[[2, 2]] = cos;
    m
}

pub fn create_z_rotation_matrix(angle_deg: f32) -> Array2<f32> {
    let mut m = Array::<f32, _>::eye(4);

    let angle_rad = angle_deg.to_radians();
    let (sin, cos) = angle_rad.sin_cos();

    m[[0, 0]] = cos;
    m[[0, 1]] = sin;
    m[[1, 0]] = -sin;
    m[[1, 1]] = cos;
    m
}

pub fn create_translation_matrix(x: f32, y: f32, z: f32) -> Array2<f32> {
    let mut tm = Array::eye(4);
    tm[[3, 0]] = x;
    tm[[3, 1]] = y;
    tm[[3, 2]] = z;
    tm
}

pub(crate) fn point_at(pos: vec3, target: vec3, up: vec3) -> Array2<f32> {
    let new_forward = target - pos;
    let new_forward_norm = normalise_vec(&new_forward);

    let up_dot_fwd = dot_product(up, new_forward_norm);

    let a = vec3 {
        x: new_forward_norm.x * up_dot_fwd,
        y: new_forward_norm.y * up_dot_fwd,
        z: new_forward_norm.z * up_dot_fwd,
    };

    let new_up = up - a;
    let new_up_norm = normalise_vec(&new_up);

    let new_right = cross_product(new_up_norm, new_forward_norm);
    // let new_right_norm = normalise_vec(new_right);

    let mut vm: ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>> = Array::eye(4);
    vm[[0, 0]] = new_right.x;
    vm[[0, 1]] = new_right.y;
    vm[[0, 2]] = new_right.z;
    vm[[0, 3]] = 0.;

    vm[[1, 0]] = new_up_norm.x;
    vm[[1, 1]] = new_up_norm.y;
    vm[[1, 2]] = new_up_norm.z;
    vm[[1, 3]] = 0.;

    vm[[2, 0]] = new_forward_norm.x;
    vm[[2, 1]] = new_forward_norm.y;
    vm[[2, 2]] = new_forward_norm.z;
    vm[[2, 3]] = 0.;

    vm[[3, 0]] = pos.x;
    vm[[3, 1]] = pos.y;
    vm[[3, 2]] = pos.z;
    vm[[3, 3]] = 1.;

    vm
}

pub fn mult_vec3_mat4(vec: vec3, mat: &Array2<f32>) -> vec3 {
    let x = mat[[0, 0]] * vec.x + mat[[1, 0]] * vec.y + mat[[2, 0]] * vec.z + mat[[3, 0]];
    let y = mat[[0, 1]] * vec.x + mat[[1, 1]] * vec.y + mat[[2, 1]] * vec.z + mat[[3, 1]];
    let z = mat[[0, 2]] * vec.x + mat[[1, 2]] * vec.y + mat[[2, 2]] * vec.z + mat[[3, 2]];
    let w = mat[[0, 3]] * vec.x + mat[[1, 3]] * vec.y + mat[[2, 3]] * vec.z + mat[[3, 3]];

    if w == 0. {
        vec3 { x, y, z }
    } else {
        vec3 {
            x: x / w,
            y: y / w,
            z: z / w,
        }
    }
}

pub fn quick_invert_mat4(mat: Array2<f32>) -> Array2<f32> {
    let mut out: ArrayBase<ndarray::OwnedRepr<f32>, Dim<[usize; 2]>> = Array::eye(4);

    out[[0, 0]] = mat[[0, 0]];
    out[[0, 1]] = mat[[1, 0]];
    out[[0, 2]] = mat[[2, 0]];
    out[[0, 3]] = 0.;

    out[[1, 0]] = mat[[0, 1]];
    out[[1, 1]] = mat[[1, 1]];
    out[[1, 2]] = mat[[2, 1]];
    out[[1, 3]] = 0.;

    out[[2, 0]] = mat[[0, 2]];
    out[[2, 1]] = mat[[1, 2]];
    out[[2, 2]] = mat[[2, 2]];
    out[[2, 3]] = 0.;

    out[[3, 0]] =
        -(mat[[3, 0]] * out[[0, 0]] + mat[[3, 1]] * out[[1, 0]] + mat[[3, 2]] * out[[2, 0]]);
    out[[3, 1]] =
        -(mat[[3, 0]] * out[[0, 1]] + mat[[3, 1]] * out[[1, 1]] + mat[[3, 2]] * out[[2, 1]]);
    out[[3, 2]] =
        -(mat[[3, 0]] * out[[0, 2]] + mat[[3, 1]] * out[[1, 2]] + mat[[3, 2]] * out[[2, 2]]);
    out[[3, 3]] = 1.;

    out
}

pub(crate) fn normalise_vec(vec: &vec3) -> vec3 {
    let x = vec.x.powf(2.);
    let y = vec.y.powf(2.);
    let z = vec.z.powf(2.);

    let xyz = (x + y + z).sqrt();

    vec3 {
        x: vec.x / xyz,
        y: vec.y / xyz,
        z: vec.z / xyz,
    }
}

pub fn normal(tri: &Tri) -> vec3 {
    let a = tri.v2 - tri.v1;
    let b = tri.v3 - tri.v1;

    let x = (a.y * b.z) - (a.z * b.y);
    let y = (a.z * b.x) - (a.x * b.z);
    let z = (a.x * b.y) - (a.y * b.x);

    normalise_vec(&vec3 { x, y, z })
}

fn dot_product(v1: vec3, v2: vec3) -> f32 {
    (v1.x * v2.x) + (v1.y * v2.y) + (v1.z * v2.z)
}

pub(crate) fn cross_product(v1: vec3, v2: vec3) -> vec3 {
    let x = v1.y * v2.z - v1.z * v2.y;
    let y = v1.z * v2.x - v1.x * v2.z;
    let z = v1.x * v2.y - v1.y * v2.x;

    vec3 { x, y, z }
}

pub fn calc_tri_illum(light_dir: &vec3, tri_normal: &vec3, colour: Colour) -> Colour {
    let norm = normalise_vec(light_dir);
    let dp = dot_product(norm, *tri_normal);

    let factor = dp.max(0.01);
    colour.scale(factor)
}

#[cfg(test)]
mod tests {
    use crate::*;
    use float_eq::assert_float_eq;
    use ndarray::arr2;

    #[test]
    fn _test_calc_tri_illum() {
        // let expected = 961644u32;
        let expected = Colour::from_u32(961644);

        let light_dir = vec3 {
            x: 1.,
            y: 2.,
            z: 3.,
        };

        let tri_normal = vec3 {
            x: -2.,
            y: 4.,
            z: -1.,
        };

        let colour = Colour::from_u32(1234567);

        let actual = calc_tri_illum(&light_dir, &tri_normal, colour);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_calc_tri_illum2() {
        let expected = Colour::from_u32(2766878);

        let light_dir = vec3 {
            x: 0.,
            y: 10.,
            z: -10.,
        };

        let tri_normal = vec3 {
            x: 0.766045,
            y: -0.271654,
            z: -0.582563,
        };

        let colour = Colour::from_u32(1234567);

        let actual = calc_tri_illum(&light_dir, &tri_normal, colour);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_quick_invert() {
        let expected = arr2(&[
            [1., 4., 7., 0.],
            [2., 5., 8., 0.],
            [3., 6., 9., 0.],
            [14., 32., 50., 1.],
        ]);

        let in_mat = arr2(&[
            [1., 2., 3., 0.],
            [4., 5., 6., 0.],
            [7., 8., 9., 0.],
            [-1., -2., -3., 1.],
        ]);

        let result = quick_invert_mat4(in_mat);

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_create_translation_matrix() {
        let expected = arr2(&[
            [1., 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., 1., 0.],
            [1., 2., 3., 1.],
        ]);

        let result = create_translation_matrix(1., 2., 3.);

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_create_view_matrix() {
        let expected = arr2(&[
            [0.766044, 0., -0.642788, 0.],
            [0., 1., 0., 0.],
            [0.642788, 0., 0.766044, 0.],
            [-4.10324, -3., -1.7786, 1.],
        ]);

        let position = vec3 {
            x: 2.,
            y: 3.,
            z: 4.,
        };

        let yaw = 40.;

        let cam = Camera {
            far_plane: 0.,
            near_plane: 0.,
            fov: 60.,
            position,
            yaw,
            fwd_speed: 10.,
            yaw_speed: 1.,
        };

        let result = cam.create_view_matrix();

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_point_at() {
        let expected = arr2(&[
            [-0.40824, 0.816497, -0.40824, 0.],
            [-0.707108, 0., 0.707105, 0.],
            [0.57735, 0.57735, 0.57735, 0.],
            [1., 2., 3., 1.],
        ]);

        let pos = vec3 {
            x: 1.,
            y: 2.,
            z: 3.,
        };

        let target = vec3 {
            x: 4.,
            y: 5.,
            z: 6.,
        };

        let up = vec3 {
            x: 7.,
            y: 8.,
            z: 9.,
        };

        let result = point_at(pos, target, up);

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_cross_product() {
        let expected = vec3 {
            x: 0.5,
            y: 6.,
            z: -5.5,
        };

        let v1 = vec3 {
            x: 2.,
            y: 3.5,
            z: 4.,
        };

        let v2 = vec3 {
            x: 5.,
            y: 6.,
            z: 7.,
        };

        let result = cross_product(v1, v2);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_normalise_vec() {
        let expected = vec3 {
            x: 0.48,
            y: 0.5724,
            z: 0.6647,
        };

        let input = vec3 {
            x: 52.,
            y: 62.,
            z: 72.,
        };

        let result = normalise_vec(&input);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_normal() {
        let expected = vec3 {
            x: 0.935472,
            y: -0.311824,
            z: -0.166306,
        };

        let v1 = vec3 {
            x: 1.,
            y: -2.,
            z: 5.,
        };

        let v2 = vec3 {
            x: 2.,
            y: 9.,
            z: -10.,
        };

        let v3 = vec3 {
            x: 3.,
            y: 4.,
            z: 5.,
        };

        let tri = Tri { v1, v2, v3 };

        let result = normal(&tri);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_create_x_rotation_matrix() {
        let expected = arr2(&[
            [1f32, 0., 0., 0.],
            [0., 0.93969262f32, 0.342020150, 0.],
            [0., -0.34202015, 0.93969262, 0.],
            [0., 0., 0., 1.],
        ]);

        let result = create_x_rotation_matrix(20.);

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_create_y_rotation_matrix() {
        let expected = arr2(&[
            [0.93969262f32, 0., 0.34202015, 0.],
            [0., 1., 0., 0.],
            [-0.34202015, 0., 0.93969262, 0.],
            [0., 0., 0., 1.],
        ]);

        let result = create_y_rotation_matrix(20.);

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_create_z_rotation_matrix() {
        let expected = arr2(&[
            [0.93969262f32, 0.34202015, 0., 0.],
            [-0.34202015, 0.93969262, 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ]);

        let result = create_z_rotation_matrix(20.);

        assert_float_eq!(
            expected.into_raw_vec(),
            result.into_raw_vec(),
            abs_all <= 0.0001
        );
    }

    #[test]
    fn test_mult_vec_matrix_1() {
        let expected = vec3 {
            x: 52.,
            y: 62.,
            z: 72.,
        };

        let vec = vec3 {
            x: 2.,
            y: 3.,
            z: 4.,
        };

        let matrix = arr2(&[
            [1., 2., 3., 0.],
            [4., 5., 6., 0.],
            [7., 8., 9., 0.],
            [10., 11., 12., 0.],
        ]);

        let result = mult_vec3_mat4(vec, &matrix);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_mult_vec_matrix_2() {
        let expected = vec3 {
            x: 10.86,
            y: -13.2,
            z: 0.7,
        };

        let vec = vec3 {
            x: 0.2,
            y: -1.3,
            z: 0.9,
        };

        let matrix = arr2(&[
            [-1.2, 2., 3., 0.],
            [4., 5., 6., 0.],
            [7., -8., 9., 0.],
            [10., 0.1, -0.2, 0.],
        ]);

        let result = mult_vec3_mat4(vec, &matrix);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_mult_vec_matrix_3() {
        let expected = vec3 {
            x: 10.86,
            y: -13.2,
            z: 0.7,
        };

        let vec = vec3 {
            x: 0.2,
            y: -1.3,
            z: 0.9,
        };

        let matrix = arr2(&[
            [-1.2, 2., 3., 0.],
            [4., 5., 6., 0.],
            [7., -8., 9., 0.],
            [10., 0.1, -0.2, 0.],
        ]);

        let result = mult_vec3_mat4(vec, &matrix);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_mult_vec_matrix_4() {
        let expected = vec3 {
            x: -0.061859,
            y: -0.494872,
            z: 0.995338,
        };

        let vec = vec3 {
            x: -1.,
            y: -6.,
            z: 21.,
        };

        let matrix = arr2(&[
            [1.29904, 0., 0., 0.],
            [0., 1.73205, 0., 0.],
            [0., 0., 1.0001, 1.],
            [0., 0., -0.10001, 0.],
        ]);

        let result = mult_vec3_mat4(vec, &matrix);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }

    #[test]
    fn test_create_projection_matrix_1() {
        let expected = arr2(&[
            [1.2990382, 0., 0., 0.],
            [0., 1.7320509, 0., 0.],
            [0., 0., 1.0001, 1.],
            [0., 0., -0.10001, 0.],
        ]);

        let screen = Screen {
            width: 800,
            height: 600,
        };
        let camera = Camera {
            fov: 60.,
            near_plane: 0.1,
            far_plane: 1000.,
            position: vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            yaw: 0.,
            fwd_speed: 10.,
            yaw_speed: 1.,
        };

        let result = camera.create_projection_matrix(screen);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_create_projection_matrix_2() {
        let expected = arr2(&[
            [0.65161270, 0., 0., 0.],
            [0., 1.3032254, 0., 0.],
            [0., 0., 1.001001, 1.],
            [0., 0., -2.002002, 0.],
        ]);

        let screen = Screen {
            width: 900,
            height: 450,
        };
        let camera = Camera {
            fov: 75.,
            near_plane: 2.,
            far_plane: 2000.,
            position: vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            yaw: 0.,
            fwd_speed: 10.,
            yaw_speed: 1.,
        };

        let result = camera.create_projection_matrix(screen);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_calc_afq() {
        let expected = AFQ {
            aspect_ratio: 0.75,
            fov: 1.73205,
            q: 1.0001,
        };

        let screen = Screen {
            width: 800,
            height: 600,
        };
        let camera = Camera {
            fov: 60.,
            near_plane: 0.1,
            far_plane: 1000.,
            position: vec3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            yaw: 0.,
            fwd_speed: 10.,
            yaw_speed: 1.,
        };

        let result = camera.calc_afq(&screen);

        assert_float_eq!(expected, result, abs_all <= 0.0001);
    }
}
