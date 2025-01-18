# Meatpack

A pure Rust implementation of Scott Mudge's [MeatPack](https://github.com/scottmudge/OctoPrint-MeatPack) algorithm. The crate works in both `std` and `no_std` environments. A CLI and bindings for other languages are in the pipeline. The library is configurable allowing you to set it up according to your embedded system resource constraints.

# Support

Please consider supporting the crate by:

- Downloading and using the crate.
- Raising issues and improvements on the GitHub repo.
- Recommending the crate to others.
- ⭐ the crate on GitHub.
- Sponsoring the [maintainer](https://github.com/sponsors/jamesgopsill).

## The Algorithm

Gcode uses a restricted alphabet and are mostly made up of numbers, decimal point, a few letters ('G', 'M', 'E', etc.), and other utilitiy characters (newline, space, etc.). Scott's histographic analysis of a dozen or so gcode files found that **~93%** of all gcode use the same **15 characters**. Gcode is stored as a sequence of `u8` ascii characters that can fit a 256-character alphabet.

**MeatPack** takes advantage of this histographic property and uses a lookup table that takes the 15 most popular characters and matches them to a 4-bit representation (capable of representing 16 characters). This 4-bit representations are packed into 8-bit/1-byte characters. This effectively doubles the data density of a gcode file.

The 16th character (`0b1111`) of the 4-bit representation is reserved for telling the unpacker that it should expect a full-width character in the following byte.

The packer reorders some characters if the full width character is surrounded by packable characters. This is to avoid 4 bits being wasted telling the packer that only one full width character is coming up.

If the lower 4-bits contains `0b1111` then we unpack the full width character followed by the upper 4-bit character. If the upper 4-bit contains `0b1111` then we unpack the lower 4-bit character followed by the full width character. This minor reordering allows slightly more data to the be packed at the cost of a little complexity.

### Example

Take the following "G1" command.

`G1 X113.214 Y91.45 E1.3154`

which consists of 27 bytes (paranthesis indicate 1 byte),

`(G) (1) ( ) (X) (1) (1) (3) (.) (2) (1) (4) ( ) (Y) (9) (1) (.) (4) (5) ( ) (E) (1) (.) (3) (1) (5) (4) (\n)`

Applying the packing algorithm we can pack the command into 16 bytes as follows:

`(1G) (X ) (11) (.33) (12) ( 4) (9#) (Y) (.1) (54) (# ) (E) (.1) (13) (45) (\n)`

The characters are ordered in bit order (upper, lower) and would be unpacked lower then upper. With "Whitespace Removal" active, we can further reduce this to 13 bytes.

`(1G) (1X) (31) (2.) (41) (9#) (Y) (.1) (54) (1E) (3.) (51) (\n4)`

The packed command is now less than **half** the size of the original command.


## Command Patterns

**MeatPack** provides a communication/control layer identified by 2 255 (`OxFF`) signal bytes followed by a command byte. `0xFF` is virtually never found in gcode, so it is can be considered a reserved character. The following command bytes exist:

| Byte (`u8`) Value | Command |
|---|---|
| 246 | Disable No Spaces |
| 247 | Enable No Spaces |
| 248 | Query Config |
| 249 | Reset All |
| 250 | Disable Packing |
| 251 | Enable Packing |
| 255 | Signal Byte |

# Examples

```Rust
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
```


# References

- <https://github.com/scottmudge/OctoPrint-MeatPack>
- <https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp>
- <https://github.com/scottmudge/Prusa-Firmware-MeatPack/blob/MK3_sm_MeatPack/Firmware/meatpack.cpp>
