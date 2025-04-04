use std::{env, fs, str};

use meatpack::{Packer, Unpacker};

fn main() {
	// Create the path to the gcode file
	let mut path = env::current_dir().unwrap();
	path.push("test_files");
	path.push("box.gcode");

	let gcode = fs::read(path).unwrap();
	let gcode_slice = &gcode[0..15];
	let mut meat = Vec::new();
	Packer::<128>::pack_slice(gcode_slice, &mut meat).unwrap();
	let gl = gcode_slice.len() as f32;
	let ml = meat.len() as f32;
	let ratio = ml / gl;

	let mut unpacked = Vec::new();
	Unpacker::<128>::unpack_slice(&meat, &mut unpacked).unwrap();

	println!("GCODE SNIPPET");
	println!("{:?}", gcode_slice);
	println!("{}", str::from_utf8(gcode_slice).unwrap());

	println!("MEAT");
	println!("{:?}", meat);
	println!("Gcode: {}, Meat: {}, {}", gl, ml, ratio);

	println!("UNPACKED");
	println!("{:?}", unpacked);
	println!("{}", str::from_utf8(&unpacked).unwrap());

	let mut meat = Vec::new();
	Packer::<128>::pack_slice(&gcode, &mut meat).unwrap();
	let gl = gcode.len() as f32;
	let ml = meat.len() as f32;
	let ratio = ml / gl;

	let mut unpacked = Vec::new();
	Unpacker::<128>::unpack_slice(&meat, &mut unpacked).unwrap();
	println!("####");
	println!(
		"Gcode: {}, Meat: {}, Unpacked: {}, Ratio: {}",
		gl,
		ml,
		unpacked.len(),
		ratio
	);

	// TODO: solve the diff
	//assert_eq!(gcode, unpacked);
	println!("{:?}", &gcode[0..20]);
	println!("{:?}", &unpacked[0..20]);
}
