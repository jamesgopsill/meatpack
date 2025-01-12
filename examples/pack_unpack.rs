use meatpack::{is_meatpack_newline, Pack, Unpack};

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

	// ## Pack ##
	// Create an instance of pack and set the internal buffer to 100 bytes.
	let mut packer = Pack::<100>::default();
	let mut packed: Vec<u8> = vec![];

	// Produce the header bytes for meatpack.
	packed.extend(packer.header());

	let mut start = 0;
	let gbytes = gcode.as_bytes();
	for (i, b) in gbytes.iter().enumerate() {
		if *b != 10 {
			continue;
		}
		// ASCII LF
		let slice = &gbytes[start..(i + 1)];
		start = i + 1;
		match packer.pack(slice) {
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
	let mut start: usize = 0;

	// Unpacking lines one by one
	for i in 0..packed.len() {
		if !is_meatpack_newline(&packed[i]) {
			continue;
		}

		// meatpack new line found.
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
