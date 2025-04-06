use std::{env, fs};

use meatpack::{Packer, Unpacker};

fn main() {
	// Create the path to the gcode file
	let mut path = env::current_dir().unwrap();
	path.push("test_files");
	path.push("box.gcode");

	// Read the gcode into memory as we have the luxury
	// plenty of RAM on a PC.
	let gcode = fs::read(path).unwrap();

	// Lets pack the gcode.
	let mut meat = Vec::new();
	Packer::<128>::pack_slice(&gcode, &mut meat).unwrap();

	// Demonstrate the difference in size
	let gl = gcode.len() as f32;
	let ml = meat.len() as f32;
	let ratio = ml / gl;
	println!("Gcode: {}, Meat: {}, {}", gl, ml, ratio);

	// Now to unpack it as if we received a meatpacked file.
	let mut unpacked = Vec::new();
	Unpacker::<128>::unpack_slice(&meat, &mut unpacked).unwrap();

	// Lets just check that these are the same.
	println!("Original [0..20]:        {:?}", &gcode[0..20]);
	println!("Packed Unpacked [0..20]: {:?}", &unpacked[0..20]);

	println!(
		"Original [len-10..]:        {:?}",
		&gcode[gcode.len() - 10..]
	);
	println!(
		"Packed Unpacked [len-10..]: {:?}",
		&unpacked[unpacked.len() - 10..]
	);
}
