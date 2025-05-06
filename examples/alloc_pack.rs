use std::{env, fs};

use meatpack::Packer;

fn main() {
    // Create the path to the gcode file
    let mut path = env::current_dir().unwrap();
    path.push("test_files");
    path.push("box.gcode");

    // Read the gcode into memory as we have the luxury
    // plenty of RAM on a PC.
    let gcode = fs::read(path).unwrap();

    println!("{:?}", &gcode[gcode.len() - 10..]);

    // Lets pack the gcode.
    let mut meat = Vec::new();

    // And demonstrate the difference in size
    Packer::<128>::pack_slice(&gcode, &mut meat, false, false).unwrap();
    let gl = gcode.len() as f32;
    let ml = meat.len() as f32;
    let ratio = ml / gl;

    println!("Gcode: {}, Meat: {}, {}", gl, ml, ratio);
}
