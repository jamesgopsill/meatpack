use std::{env, fs};

use meatpack::Packer;

fn main() {
	// Create the path to the gcode file
	let mut path = env::current_dir().unwrap();
	path.push("test_files");
	path.push("box.gcode");

	let gcode = fs::read(path).unwrap();
	let mut meat = Vec::new();
	Packer::<128>::pack_slice(&gcode, &mut meat).unwrap();
	let gl = gcode.len() as f32;
	let ml = meat.len() as f32;
	let ratio = ml / gl;
	println!("Gcode: {}, Meat: {}, {}", gl, ml, ratio);
}
