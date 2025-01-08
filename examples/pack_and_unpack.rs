use meatpack::{core::print_ascii, Packer, Unpacker};
use no_std_io::io::{BufReader, BufWriter};

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000";

	println!("{}", gcode);
	// Packing
	let reader = BufReader::new(gcode.as_bytes());
	// Provide the writer a Vec<u8> to fill.
	let writer = BufWriter::new(Vec::<u8>::new());
	let mut packer = Packer::<128, 64, _, _>::new(reader, writer);
	match packer.pack() {
		Ok(_) => {
			// Get access to the underlying inner again to
			// read back the contents.
			let out = packer.writer.into_inner().unwrap();
			println!("{:?}", &out);
			// Unpacking
			let reader = BufReader::new(out.as_slice());
			let mut unpacker = Unpacker::<64, 128, _>::new(reader);
			while let Some(line) = unpacker.unpack_line() {
				match line {
					Ok(line) => {
						#[cfg(feature = "std")]
						print_ascii(line);
						#[cfg(feature = "no_std")]
						println!("{:?}", &line);
					}
					Err(e) => println!("{:?}", e),
				}
			}
		}
		Err(e) => println!("{:?}", e),
	}
}
