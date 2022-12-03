use std::fs;
use std::io;

use ndarray::arr2;
use ndarray::prelude::*;
use ndarray::Array;

pub struct Screen {
    width : i32,
    height : i32,
}

pub struct Camera {
    fov : f32,
    near_plane : f32,
    far_plane : f32,
}


//#[derive(Debug)]
#[derive(Debug, Clone, Copy)]
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

pub fn calc_proj_matrix() -> Array2<f32> {
     let mut pm = Array::<f32, _>::zeros((4, 4).f());
     pm[[0,0]] = 1.29904;
     pm[[1,1]] = 1.73205;
     pm[[2,2]] = 1.0001;
     pm[[2,3]] = 1.0;
     pm[[3,2]] = -0.10001;
     pm        
}

pub fn Create_Projection_Matrix() -> Array<f32, Ix2> {
    let pm = arr2(&[
        [1.29904, 0.0, 0.0, 0.0],
        [0.0, 1.73205, 0.0, 0.0],
        [0.0, 0.0, 1.0001, 1.0],
        [0.0, 0.0, -0.10001, 0.0],
    ]);
    pm
}
pub fn calc_view_matrix() -> Array2<f32> {
    let mut vm = Array::<f32, _>::eye(4);
    vm[[3,2]] = 10.0;
    vm        
}

#[test]
fn test1() {
    assert_eq!(1, 1);
}

#[test]
fn test_create_projection_matrix() {
    let expected = arr2(&[
        [1.29904, 0.0, 0.0, 0.0],
        [0.0, 1.73205, 0.0, 0.0],
        [0.0, 0.0, 1.0001, 1.0],
        [0.0, 0.0, -0.10001, 0.0],
    ]);
    let result = Create_Projection_Matrix();

    assert_eq!(expected, result);
}

pub fn calc_trans_matrix(x:f32, y:f32,z:f32) -> Array2<f32> {
    let mut tm = Array::eye(4);
    tm[[3,0]] = x;
    tm[[3,1]] = y;
    tm[[3,2]] = z;
    tm

}

pub fn mult_vec_matrix(v_in:Array1<f32>, m:&Array2<f32>) -> Array1<f32> {
    let mut v_out = Array::<f32, _>::zeros((3).f());
    v_out[[0]] = m[[0,0]] * v_in[[0]] + m[[1,0]] * v_in[[1]] + m[[2,0]] * v_in[[2]] + m[[3,0]]; 
    v_out[[1]] = m[[0,1]] * v_in[[0]] + m[[1,1]] * v_in[[1]] + m[[2,1]] * v_in[[2]] + m[[3,1]]; 
    v_out[[2]] = m[[0,2]] * v_in[[0]] + m[[1,2]] * v_in[[1]] + m[[2,2]] * v_in[[2]] + m[[3,2]]; 
    v_out        
}