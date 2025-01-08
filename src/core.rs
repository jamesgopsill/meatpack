//! Core functions for MeatPack.
use no_std_io::io::Error;

pub static SIGNAL_BYTE: u8 = 255;
pub static ENABLE_PACKING_BYTE: u8 = 251;
pub static LINEFEED_BYTE: u8 = 10;

/// A set of possible error codes from the MeatPack crate.
#[derive(Debug)]
pub enum MeatPackError {
	IntoInnerError,
	InvalidByte,
	InvalidCommandByte,
	BufferFull,
	FullWidthByte,
	LineTooSmallToPack,
	IOError(Error),
}

/// An enum detailing all the available Meatpack commands.
#[derive(Debug)]
pub enum MeatPackCommand {
	PackingEnabled,
	PackingDisabled,
	ResetAll,
	QueryConfig,
	NoSpacesEnabled,
	NoSpacesDisabled,
	SignalByte,
}

/// This function checks whether a `u8` conforms to one of the reserved
/// command bytes.
///
/// | 8-bit Decimal Code | Command |
/// | --- | --- |
/// | 246 | No Spaces Disabled |
/// | 247 | No Spaces Enabled |
/// | 248 | Query Config |
/// | 249 | Reset All |
/// | 250 | Packing Disabled |
/// | 251 | Packing Enabled |
/// | 255 | Signal Byte |
///
/// References
/// - <https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp>
pub fn determine_command(byte: &u8) -> Result<MeatPackCommand, MeatPackError> {
	match byte {
		246 => Ok(MeatPackCommand::NoSpacesDisabled),
		247 => Ok(MeatPackCommand::NoSpacesEnabled),
		248 => Ok(MeatPackCommand::QueryConfig),
		249 => Ok(MeatPackCommand::ResetAll),
		250 => Ok(MeatPackCommand::PackingDisabled),
		251 => Ok(MeatPackCommand::PackingEnabled),
		255 => Ok(MeatPackCommand::SignalByte),
		_ => Err(MeatPackError::InvalidCommandByte),
	}
}

/// Checks whether a `u8` is a signal byte `255`.
pub fn is_signal_byte(byte: &u8) -> bool {
	matches!(byte, 255)
}

/// Checks whether a `u8` is a linefeed byte `10`.
pub fn is_linefeed_byte(byte: &u8) -> bool {
	matches!(byte, 10)
}

/// Unpacks the 2 x 4-bit meatpack code packed into a u8.
pub fn unpack_byte(
	byte: &u8,
	no_spaces: bool,
) -> Result<(u8, u8), MeatPackError> {
	// Process the 8-bit as two 4-bit values.
	// 4-bits still exist within a u8.
	let mut unpacked: (u8, u8) = (0, 0);
	let upper = byte >> 4;
	let u = reverse_lookup(&upper, no_spaces)?;
	unpacked.0 = u;
	let lower = byte << 4 >> 4;
	let u = reverse_lookup(&lower, no_spaces)?;
	unpacked.1 = u;
	Ok(unpacked)
}

/// Provides the lookup table for the 4-bit combinations and their 8-bit counterparts. `0b1011` has different intepretations depending on whether `no_spaces` has been enabled or disabled.
///
/// | 4-bit Decimal Code | 8-bit ASCII char (decimal code) |
/// | --- | --- |
/// | 0b0000 | 0 (48) |
/// | 0b0001 | 1 (49) |
/// | 0b0010 | 2 (50) |
/// | 0b0011 | 3 (51) |
/// | 0b0100 | 4 (52) |
/// | 0b0101 | 5 (53) |
/// | 0b0110 | 6 (54) |
/// | 0b0111 | 7 (55) |
/// | 0b1000 | 8 (56) |
/// | 0b1001 | 9 (57) |
/// | 0b1010 | . (46) |
/// | 0b1011 | Space (32) or E (69) |
/// | 0b1100 | \n (10) |
/// | 0b1101 | G (71) |
/// | 0b1110 | X (88) |
/// | 0b1111 | NUL (0) |
///
/// References
/// - <https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp>
/// - <https://www.asciitable.com/>
pub fn reverse_lookup(
	byte: &u8,
	no_spaces: bool,
) -> Result<u8, MeatPackError> {
	match byte {
		0b0000 => Ok(48),
		0b0001 => Ok(49),
		0b0010 => Ok(50),
		0b0011 => Ok(51),
		0b0100 => Ok(52),
		0b0101 => Ok(53),
		0b0110 => Ok(54),
		0b0111 => Ok(55),
		0b1000 => Ok(56),
		0b1001 => Ok(57),
		0b1010 => Ok(46),
		0b1011 => {
			if no_spaces {
				Ok(69)
			} else {
				Ok(32)
			}
		}
		0b1100 => Ok(10),
		0b1101 => Ok(71),
		0b1110 => Ok(88),
		0b1111 => Ok(0),
		_ => Err(MeatPackError::InvalidByte),
	}
}

/// The forward lookup variant of the reverse lookup byte.
pub fn forward_lookup(
	byte: &u8,
	no_spaces: bool,
) -> Result<u8, MeatPackError> {
	match byte {
		48 => Ok(0b0000),
		49 => Ok(0b0001),
		50 => Ok(0b0010),
		51 => Ok(0b0011),
		52 => Ok(0b0100),
		53 => Ok(0b0101),
		54 => Ok(0b0110),
		55 => Ok(0b0111),
		56 => Ok(0b1000),
		57 => Ok(0b1001),
		46 => Ok(0b1010),
		69 => {
			if no_spaces {
				Ok(0b1011)
			} else {
				Err(MeatPackError::FullWidthByte)
			}
		}
		32 => {
			if no_spaces {
				Err(MeatPackError::FullWidthByte)
			} else {
				Ok(0b1011)
			}
		}
		10 => Ok(0b1100),
		71 => Ok(0b1101),
		88 => Ok(0b1110),
		_ => Err(MeatPackError::FullWidthByte),
	}
}

#[cfg(feature = "std")]
/// Utility function that accepts a `u8` and prints the utf8 [0-255] chars which includes the ASCII table.
pub fn print_ascii(bytes: &[u8]) {
	for b in bytes {
		if *b != 0 {
			let c = char::from(*b);
			print!("{}", c);
		}
	}
	println!();
}
