mod threed;

fn main() {
    println!("Hello, world!");

let obj = threed::Object::create_from_file("c:\\temp\\cube.obj".to_string());

match obj {
    Ok(obj) => print!("Object loaded: {obj:#?}"),
    Err(_) => ()
}

}
