use meatpack::{Pack, Unpack};

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000";

	let gcode_bytes = gcode.as_bytes();
	let mut packer = Pack::<100>::new(gcode_bytes);
	let mut packed: Vec<u8> = Vec::new();

	// Add the header.
	packed.extend(packer.header());

	// Pack the lines
	while let Some(line) = packer.pack_line() {
		match line {
			Ok(line) => {
				packed.extend(line);
			}
			Err(e) => println!("{:?}", e),
		}
	}

	// Now read the out vec.
	println!("{:?}", packed);

	let mut unpacked: Vec<u8> = Vec::new();
	let mut unpacker = Unpack::<100>::new(packed.as_slice());

	while let Some(line) = unpacker.unpack_line() {
		match line {
			Ok(line) => {
				unpacked.extend(line);
				unpacked.push(10); // linefeed byte
			}
			Err(e) => println!("{:?}", e),
		}
	}

	println!("{:?}", unpacked);

	// If in std.
	for byte in unpacked {
		let c = char::from(byte);
		print!("{}", c);
	}
	println!();
}
