use meatpack::{Pack, Unpack};

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

	// ## Pack ##
	// Pack a set of lines
	let gcode_bytes = gcode.as_bytes();
	let mut packer = Pack::<100>::new(gcode_bytes);
	let mut packed: Vec<u8> = vec![];

	// Produce the header bytes for meatpack.
	packed.extend(packer.header());

	// Iterating through a slice of commands and yielding packed bytes.
	while let Some(line) = packer.pack_next_cmd() {
		match line {
			Ok(line) => packed.extend(line),
			Err(e) => {
				println!("{:?}", e);
				break;
			}
		}
	}

	println!("{:?}", packed);

	// ## Unpack ##
	let mut unpacker = Unpack::<100>::default();
	let delimiter: u8 = 0b1100;
	let mut start: usize = 0;
	for i in 0..packed.len() {
		let b = packed[i] >> 4;
		if b == delimiter {
			let slice = &packed[start..=i];
			let cmd = unpacker.unpack(slice);
			match cmd {
				Ok(cmd) => {
					// If in std.
					for byte in cmd {
						let c = char::from(*byte);
						print!("{}", c);
					}
				}
				Err(e) => println!("{:?}", e),
			}
			start = i + 1;
		}
	}
}

/*

println!("{}", gcode);

let gcode_bytes = gcode.as_bytes();
let mut packer = Pack::<100>::new(gcode_bytes);
let mut packed: Vec<u8> = Vec::new();

// Add the header.
packed.extend(packer.header());

// Pack the lines
while let Some(line) = packer.pack_next_cmd() {
	match line {
		Ok(line) => packed.extend(line),
		Err(e) => println!("{:?}", e),
	}
}

// Now read the packed vec.
println!("{:?}", packed);

let mut unpacked: Vec<u8> = Vec::new();
let mut unpacker = Unpack::<100>::new(packed.as_slice());

while let Some(line) = unpacker.unpack_cmd() {
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
*/
