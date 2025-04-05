use core::str;

use meatpack::{MeatPackResult, Unpacker};

fn main() {
	// Some meatpacked gcode
	let packed: [u8; 93] = [
		255, 255, 251, 255, 255, 247, 255, 255, 250, 59, 32, 10, 255, 255, 251, 127, 77, 243, 32,
		15, 80, 255, 32, 82, 195, 127, 77, 243, 32, 15, 81, 255, 32, 83, 195, 47, 77, 16, 239, 32,
		4, 0, 255, 32, 89, 4, 0, 255, 32, 90, 2, 240, 32, 43, 5, 192, 47, 77, 48, 239, 32, 3, 240,
		32, 63, 89, 0, 255, 32, 90, 4, 191, 32, 1, 192, 47, 77, 64, 255, 32, 80, 4, 0, 255, 32, 82,
		33, 0, 255, 32, 84, 4, 0,
	];

	// Initiliase the packer with buffer size depending
	// on your application
	let mut unpacker = Unpacker::<64>::default();

	// Imagine receiving the bytes from some I/O and we want
	// to construct gcode lines and deal with them as we form them.
	for b in packed.iter() {
		let res = unpacker.unpack(b);
		match res {
			Ok(MeatPackResult::WaitingForNextByte) => {
				//println!("Waiting for next byte");
			}
			Ok(MeatPackResult::Line(line)) => {
				let line = str::from_utf8(line).unwrap();
				println!("{:?}", line);
			}
			Err(e) => {
				println!("{:?}", e);
				panic!();
			}
		}
	}
}
