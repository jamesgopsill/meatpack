use std::process;

use meatpack::{MeatPackResult, Packer, Unpacker};

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";
	let mut packer = Packer::<64>::default();
	let mut out: Vec<u8> = vec![];

	out.extend(packer.header());

	for byte in gcode.as_bytes() {
		let packed = packer.pack(byte);
		match packed {
			Ok(MeatPackResult::Line(line)) => {
				println!("{:?}", line);
				out.extend(line);
			}
			Ok(MeatPackResult::WaitingForNextByte) => {}
			Err(e) => println!("{:?}", e),
		}
	}

	println!("{:?}", out);

	let mut unpacker = Unpacker::<64>::default();
	for byte in out {
		let res = unpacker.unpack(&byte);
		match res {
			Ok(MeatPackResult::WaitingForNextByte) => {}
			Ok(MeatPackResult::Line(line)) => {
				// If in std.
				for byte in line {
					let c = char::from(*byte);
					print!("{}", c);
				}
			}
			Err(e) => {
				println!("{:?}", e);
				process::exit(0)
			}
		}
	}
}
