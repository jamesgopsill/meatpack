use std::process::exit;

use meatpack::Parser;

fn main() {
	/*
		let gcode = "M73 P0 R3
	M73 Q0 S3
	M201 X4000 Y4000 Z200 E2500
	M203 X300 Y300 Z40 E100
	M204 P4000 R1200 T4000
	";

		let mut parser = Parser::<64>::default();

		for b in gcode.as_bytes() {
			let res = parser.parse_byte(b);
			match res {
				Ok(meatpack::MeatPackOutput::WaitingForNextByte) => {
					println!("Waiting for next byte");
				}
				Ok(meatpack::MeatPackOutput::Line(line)) => {
					println!("{:?}", line);
					for byte in line {
						let c = char::from(*byte);
						print!("{}", c);
					}
				}
				Err(e) => {
					println!("{:?}", e);
				}
			}
		}
		*/

	let packed: [u8; 93] = [
		255, 255, 251, 255, 255, 247, 255, 255, 250, 59, 32, 10, 255, 255, 251, 127, 77, 243, 32,
		15, 80, 255, 32, 82, 195, 127, 77, 243, 32, 15, 81, 255, 32, 83, 195, 47, 77, 16, 239, 32,
		4, 0, 255, 32, 89, 4, 0, 255, 32, 90, 2, 240, 32, 43, 5, 192, 47, 77, 48, 239, 32, 3, 240,
		32, 63, 89, 0, 255, 32, 90, 4, 191, 32, 1, 192, 47, 77, 64, 255, 32, 80, 4, 0, 255, 32, 82,
		33, 0, 255, 32, 84, 4, 0,
	];

	let mut parser = Parser::<64>::default();

	for b in packed.iter() {
		let res = parser.parse_byte(b);
		match res {
			Ok(meatpack::MeatPackOutput::WaitingForNextByte) => {
				println!("Waiting for next byte");
			}
			Ok(meatpack::MeatPackOutput::Line(line)) => {
				println!("{:?}", line);
				for byte in line {
					let c = char::from(*byte);
					print!("{}", c);
				}
			}
			Err(e) => {
				println!("{:?}", e);
				exit(0);
			}
		}
	}
}
