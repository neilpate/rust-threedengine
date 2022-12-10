use std::fs;
use std::io;

use approx::assert_abs_diff_eq;
use ndarray::arr2;
use ndarray::prelude::*;
use ndarray::Array;


use float_eq::{assert_float_eq, float_eq, derive_float_eq};

pub struct Screen {
     pub width : i32,
     pub height : i32,
}

pub struct Camera {
    pub fov : f32,
    pub near_plane : f32,
    pub far_plane : f32,
}

#[derive_float_eq(
    ulps_tol = "AFQUlps", 
    ulps_tol_derive = "Clone, Copy, Debug, PartialEq",
    debug_ulps_diff = "AFQDebugUlpsDiff",
    debug_ulps_diff_derive = "Clone, Copy, Debug, PartialEq",
    all_tol = "f32"
)]
#[derive(Debug, PartialEq, Clone, Copy)]
struct AFQ {
    aspect_ratio : f32,
    fov : f32,
    q : f32,
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
    pub z: f32
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

#[derive(Debug)]
pub struct Tri {
    pub v1: Vert,
    pub v2: Vert,
    pub v3: Vert
}
#[derive(Debug)]
pub struct Object {
    pub tris : Vec<Tri>
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

    pub fn create_from_file(obj_path: String) -> Result<Object, io::Error> {
        let content = fs::read_to_string(obj_path)?;
        //   println!("{content}");
        let lines = content.split("\r\n");

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

        Ok(Object { tris: mesh })
    }
    

    }

use Vert as vec3; 


fn calc_afq(screen: &Screen, camera:&Camera) -> AFQ{
    let aspect_ratio = (screen.height as f32) / (screen.width as f32);

    let fov = 1./((camera.fov / 2.).to_radians().tan());

    let q = camera.far_plane / (camera.far_plane - camera.near_plane);
    AFQ {aspect_ratio, fov, q}
}

pub fn create_projection_matrix(screen: Screen, camera:Camera) -> Array<f32, Ix2> {
    
    let afq = calc_afq(&screen, &camera);

    let mut m = arr2(&[
        [0., 0., 0., 0.],
        [0., 0., 0., 0.],
        [0., 0., 0., 1.],
        [0., 0., 0., 0.],
    ]);

    m[[0,0]] = afq.aspect_ratio * afq.fov;
    m[[1,1]] = afq.fov;
    m[[2,2]] = afq.q;
    m[[3,2]] = -1.* afq.q * camera.near_plane;

    m
}
pub fn create_y_rotation_matrix(angle_deg : f32) -> Array2<f32> {
    let mut m = Array::<f32, _>::eye(4);
   
    let angle_rad = angle_deg.to_radians();
    let (sin, cos)  = angle_rad.sin_cos();
   
    m[[0,0]] = cos;
    m[[0,2]] = sin;
    m[[2,0]] = -sin;
    m[[2,2]] = cos;
    m        
}

pub fn calc_trans_matrix(x:f32, y:f32,z:f32) -> Array2<f32> {
    let mut tm = Array::eye(4);
    tm[[3,0]] = x;
    tm[[3,1]] = y;
    tm[[3,2]] = z;
    tm

}

pub fn mult_vec3_mat4(vec:vec3, mat:&Array2<f32>) -> vec3 {
    let x = mat[[0,0]] * vec.x + mat[[1,0]] * vec.y + mat[[2,0]] * vec.z + mat[[3,0]]; 
    let y = mat[[0,1]] * vec.x + mat[[1,1]] * vec.y + mat[[2,1]] * vec.z + mat[[3,1]]; 
    let z = mat[[0,2]] * vec.x + mat[[1,2]] * vec.y + mat[[2,2]] * vec.z + mat[[3,2]]; 
    vec3 {x, y, z}       
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

    assert_float_eq!(expected.into_raw_vec(), result.into_raw_vec(), abs_all <= 0.0001);    //Don't like this conversion to vec just to
    //assert_abs_diff_eq!(expected, result);                                                                                            
}
    
#[test]
fn test_mult_vec_matrix_1() {
    let expected = vec3 {x: 52., y: 62., z: 72.};
    
    let vec = vec3{ x: 2., y:  3., z: 4.};
    
    let matrix = arr2(&[
        [1., 2., 3., 0.],
        [4., 5., 6., 0.],
        [7., 8., 9., 0.],     
        [10., 11., 12., 0.],
        ]);
        
        let result = mult_vec3_mat4(vec, &matrix );
        
        assert_float_eq!(expected, result, abs_all <= 0.0001);
    } 

#[test]
fn test_mult_vec_matrix_2() {
    let expected = vec3{ x: 10.86, y: -13.2, z: 0.7};
    
    let vec = vec3{ x: 0.2, y: -1.3, z: 0.9};
    
    let matrix = arr2(&[
        [-1.2, 2., 3., 0.],
        [4., 5., 6., 0.],
        [7., -8., 9., 0.],     
        [10., 0.1, -0.2, 0.],
        ]);
        
        let result = mult_vec3_mat4(vec, &matrix );
        
        assert_float_eq!(expected, result, abs_all <= 0.0001); 
    } 
    
    #[test]
    fn test_mult_vec_matrix_3() {
        let expected = vec3 { x: 10.86, y: -13.2, z: 0.7};
        
        let vec = vec3 { x: 0.2, y: -1.3, z: 0.9};
        
        let matrix = arr2(&[
            [-1.2, 2., 3., 0.],
            [4., 5., 6., 0.],
            [7., -8., 9., 0.],     
            [10., 0.1, -0.2, 0.],
        ]);
        
        let result = mult_vec3_mat4(vec, &matrix );
        
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

    let screen = Screen { width : 800, height :600};
    let camera = Camera {fov: 60., near_plane : 0.1, far_plane : 1000.};

    let result = create_projection_matrix(screen, camera);

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

    let screen = Screen { width : 900, height :450};
    let camera = Camera {fov: 75., near_plane : 2., far_plane : 2000.};

    let result = create_projection_matrix(screen, camera);

    assert_eq!(expected, result);
}

#[test]
fn test_calc_afq(){
    let expected = AFQ { aspect_ratio : 0.75, fov : 1.73205, q : 1.0001};

    let screen = Screen { width : 800, height :600};
    let camera = Camera {fov: 60., near_plane : 0.1, far_plane : 1000.};

    let result = calc_afq(&screen, &camera);

  //  assert_eq!(expected, result);
  assert_float_eq!(expected, result, abs_all <= 0.0001);
}
