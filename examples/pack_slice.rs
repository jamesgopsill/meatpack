use meatpack::{pack_cmd, Pack};

fn main() {
	let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

	// Pack a set of lines
	let gcode_bytes = gcode.as_bytes();
	let mut packer = Pack::<100>::new(gcode_bytes);

	// Produce the header bytes for meatpack.
	println!("{:?}", packer.header());

	// Iterating through a slice of commands and yielding packed bytes.
	while let Some(line) = packer.pack_next_cmd() {
		match line {
			Ok(line) => {
				println!("{:?}", line);
			}
			Err(e) => {
				println!("{:?}", e);
				break;
			}
		}
	}

	// Or using the function directly for per line packing.
	let mut start = 0;
	let mut i = 0;
	let gbytes = gcode.as_bytes();
	for b in gbytes {
		i += 1;
		if *b == 10 {
			let input = &gbytes[start..i];
			start = i;
			let mut out = [0u8; 100];
			let res = pack_cmd(input, &mut out);
			match res {
				Ok((_, written)) => {
					println!("{:?}", &out[0..written]);
				}
				Err(e) => {
					println!("{:?}", e);
					break;
				}
			}
		}
	}
}
