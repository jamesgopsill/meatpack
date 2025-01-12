use crate::{
	arrwriter::ArrWriter,
	meat::{
		forward_lookup, MeatPackError, COMMENT_START_BYTE, LINEFEED_BYTE, PACKING_ENABLED_BYTE,
		SIGNAL_BYTE,
	},
};

/// Packs a single gcode command (i.e., line).
/// Returns on the first line feed it encounters.
/// Errors if no LF encountered at the end of the slice.
/// Strips all comments.
pub fn pack_cmd<'a, const S: usize>(
	input: &'a [u8],
	output: &'a mut [u8; S],
) -> Result<(usize, usize), MeatPackError> {
	if input[input.len() - 1] != LINEFEED_BYTE {
		return Err(MeatPackError::LineFeedMissing);
	}
	let mut read: usize = 0;
	let mut written: usize = 0;
	let mut iter = input.iter();
	let mut writer = ArrWriter::new(output);
	let mut comment_flag = false;

	// Now read in the data from inner.
	while let Some(byte_one) = iter.next() {
		read += 1;
		if *byte_one == COMMENT_START_BYTE {
			comment_flag = true;
		}
		// Check if the byte is \n and the lower is not populated
		// so we add a \n\n packed byte.
		if *byte_one == LINEFEED_BYTE {
			writer.push(&204)?;
			written += 1;
			return Ok((read, written));
		}

		if comment_flag {
			// Ignore the rest of the line and
			// waiting to hit \n with the clause above.
			continue;
		}

		let mut linefeed_flag = false;
		let byte_two = iter.next();
		read += 1;
		if byte_two.is_none() {
			return Err(MeatPackError::EmptySlice);
		}
		let byte_two = byte_two.unwrap();
		if *byte_two == LINEFEED_BYTE {
			linefeed_flag = true;
		}

		let packed_one = forward_lookup(byte_one, false);
		let packed_two = forward_lookup(byte_two, false);

		match (packed_one, packed_two) {
			// Both packable
			(Ok(lower), Ok(upper)) => {
				let packed = upper << 4;
				let packed = packed ^ lower;
				writer.push(&packed)?;
				written += 1;
			}
			(Ok(lower), Err(_)) => {
				let upper: u8 = 0b1111;
				let packed = upper << 4;
				let packed = packed ^ lower;
				writer.push(&packed)?;
				written += 1;
				writer.push(byte_two)?;
				written += 1;
			}
			(Err(_), Ok(upper)) => {
				let lower: u8 = 0b1111;
				let packed = upper << 4;
				let packed = packed ^ lower;
				writer.push(&packed)?;
				written += 1;
				writer.push(byte_one)?;
				written += 1;
			}
			// Both not packable so needs a signal byte in front
			(Err(_), Err(_)) => {
				writer.push(&SIGNAL_BYTE)?;
				written += 1;
				writer.push(byte_one)?;
				written += 1;
				writer.push(byte_two)?;
				written += 1;
			}
		}

		if linefeed_flag {
			return Ok((read, written));
		}
	}
	Err(MeatPackError::LineFeedMissing)
}

/// A utility struct for packing a long series of gcode commands
/// that wraps around the core `pack_cmd` function.
pub struct Pack<'a, const S: usize> {
	pos: usize,
	slice: &'a [u8],
	buffer: [u8; S],
}

impl<'a, const S: usize> Pack<'a, S> {
	/// Create a new instance of Pack.
	pub fn new(slice: &'a [u8]) -> Self {
		Self {
			pos: 0,
			slice,
			buffer: [0u8; S],
		}
	}

	/// Return a header that needs to be place at the start of
	/// a meatpacked gcode stream.
	pub fn header(&self) -> [u8; 3] {
		[SIGNAL_BYTE, SIGNAL_BYTE, PACKING_ENABLED_BYTE]
	}

	/// Packs a line of gcode from the provided slice.
	pub fn pack_next_cmd(&mut self) -> Option<Result<&[u8], MeatPackError>> {
		let buf = &mut self.buffer;
		let slice = &self.slice[self.pos..];
		if slice.is_empty() {
			return None;
		}
		let packed_cmd = pack_cmd(slice, buf);
		match packed_cmd {
			Ok((read, written)) => {
				self.pos += read;
				let out = &buf[0..written];
				if written > 0 {
					return Some(Ok(out));
				}
				None
			}
			Err(e) => Some(Err(e)),
		}
	}
}
