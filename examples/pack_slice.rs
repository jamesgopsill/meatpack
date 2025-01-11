use meatpack::Pack;

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000";

	let gcode_bytes = gcode.as_bytes();
	let mut packer = Pack::<100>::new(gcode_bytes);

	// Produce the header bytes for meatpack.
	println!("{:?}", packer.header());

	// Pack lines individually
	while let Some(line) = packer.pack_line() {
		match line {
			Ok(line) => {
				println!("{:?}", line);
			}
			Err(e) => println!("{:?}", e),
		}
	}
}
