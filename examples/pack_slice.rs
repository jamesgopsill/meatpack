use meatpack::Pack;

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

	// Create an instance of pack and set the internal buffer to 100 bytes.
	let mut packer = Pack::<100>::default();

	// Produce the header bytes for meatpack.
	println!("{:?}", packer.header());

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
			Ok(line) => {
				println!("{:?}", line);
			}
			Err(e) => {
				println!("{:?}", e);
				break;
			}
		}
	}
}
