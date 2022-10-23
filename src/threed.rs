use std::fs;
use std::io;
//#[derive(Debug)]
#[derive(Debug, Clone, Copy)]
pub struct Vert {
    x: f32,
    y: f32,
    z: f32
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
                return Some(Vert { x, y, z})
            }
            else {
            return None;
            }
        }
        else{
            return None;
        }
    }
}

#[derive(Debug)]
pub struct Tri {
    v1: Vert,
    v2: Vert,
    v3: Vert
}
#[derive(Debug)]
pub struct Object {
    tris : Vec<Tri>
}

impl Object {

fn face_from_string(s: String) -> Option<(usize, usize, usize)>{
   // println!("Face from string: {s}");
    let chunks: Vec<&str> = s.split(" ").collect();


    if chunks.len() == 4 {

        if chunks[0] == "f" {

            let v1_index: usize = chunks[1].parse().unwrap();
            let v2_index: usize = chunks[2].parse().unwrap();
            let v3_index: usize = chunks[3].parse().unwrap();
           Some( (v1_index, v2_index, v3_index))
        }
        else{
            return None
        }
    }
    else{
        return None
    }
}

    pub fn create_from_file(obj_path : String) -> Result<Object, io::Error> {
        
        let content = fs::read_to_string(obj_path)?;
     //   println!("{content}");
        let lines = content.split("\r\n"); 

        let mut verts: Vec<Vert> = Vec::new();
        let mut faces: Vec<(usize, usize, usize)> = Vec::new();
         
        for line in lines {
            let vert = Vert::from_string(line.to_string());
            match vert {
                Some(v) => verts.push(v), //Vertex was ok
                None => (),   //Not a valid vertex, just ignore
            }

            let face = Object::face_from_string(line.to_string());
            match face {
                Some(f) => faces.push(f), //Vertex was ok
                None => (),   //Not a valid face, just ignore
            }
        }
        
        let mut mesh: Vec<Tri> = Vec::new();
        for face in faces {
            let v1 = verts[face.0 -1];
            let v2 = verts[face.1 -1];
            let v3 = verts[face.2 -1];

            mesh.push(Tri{v1,v2,v3});

        }
        
        Ok(Object { tris: mesh })

    }
    
}

