mod threed;

use ndarray::prelude::*;

fn main() {
    println!("Hello, world!");

    let obj = threed::Object::create_from_file("c:\\temp\\cube.obj".to_string());

    // match obj {
    //     Ok(obj) => println!("ok"),//print!("Object loaded: {obj:#?}"),
    //     Err(_) => (),
    // }

    let pm = threed::calc_proj_matrix();
  //  println!("{}", pm);

    let vm = threed::calc_view_matrix();
  //  println!("{}", vm);

    let tm = threed::calc_trans_matrix(0.0, 0.0, 0.0);
    println!("Translation matrix:{}", tm);

    let pt = array![
        5.,6.,7.,
    ]; 
   // println!("{}", pt);
    
    let tt = threed::mult_vec_matrix(pt, &tm);
    
   // println!("{}", tt);

    //Loop over all the triangles in the object
    //Do the mult_vec_matrix with the transform matrix
    //Do backface culling
    //For each point in the triangle do the mult_vec_matrix with the view matrix to get a new triangle



    for tri in obj.unwrap().tris  {
        //let new_v1 = tri.v1;
        let v1_trans = threed::mult_vec_matrix(array![tri.v1.x, tri.v1.y, tri.v1.z], &tm);
        let v2_trans = threed::mult_vec_matrix(array![tri.v2.x, tri.v2.y, tri.v2.z], &tm);
        let v3_trans = threed::mult_vec_matrix(array![tri.v3.x, tri.v3.y, tri.v3.z], &tm);
    
        println!("{}", v1_trans);

        let v1_view = threed::mult_vec_matrix(v1_trans, &vm);
        let v2_view = threed::mult_vec_matrix(v2_trans, &vm);
        let v3_view = threed::mult_vec_matrix(v3_trans, &vm);



    }

   // let tris = obj.as_ref().unwrap().tris;


}
