mod threed;

use ndarray::{arr1, arr2, Array1};

fn main() {
    println!("Hello, world!");

    let obj = threed::Object::create_from_file("c:\\temp\\cube.obj".to_string());

    match obj {
        Ok(obj) => print!("Object loaded: {obj:#?}"),
        Err(_) => (),
    }

    let vector = arr1(&[1, 2, 3, 0]);

    let matrix = arr2(&[[2, 1, 1, 0], [1, -1, 1, 0], [1, 1, 0, 0], [1, 1, 1, 0]]);

    //  let new_vector: Array1<_> = scalar * vector;
    //println!("{}", new_vector);

    //   let new_matrix = matrix.dot(&new_vector);
    let new_matrix = vector.dot(&matrix);
    println!("{}", new_matrix);
}
