use meatpack::{MeatPackResult, Packer};

fn main() {
	// Some example gcode.
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

	// Initiliase the packer with buffer size depending
	// on your application
	let mut packer = Packer::<64>::default();

	// Feed in the bytes as you receive them and
	// the packer will return completed lines of
	// meatpacked gcode to send onwards.
	for byte in gcode.as_bytes() {
		let packed = packer.pack(byte);
		match packed {
			Ok(MeatPackResult::Line(line)) => {
				println!("{:?}", line);
			}
			Ok(MeatPackResult::WaitingForNextByte) => {}
			Err(e) => {
				println!("{:?}", e);
				panic!()
			}
		}
	}
}
