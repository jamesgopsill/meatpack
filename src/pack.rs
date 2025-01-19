use crate::meat::{
	forward_lookup, MeatPackError, MeatPackResult, COMMENT_START_BYTE, LINEFEED_BYTE,
	PACKING_ENABLED_BYTE, SIGNAL_BYTE,
};

/// A  struct for that packs bytes and emits
/// lines of meatpacked gcode. Stripping comments
/// is on by default and empty lines are omitted.
pub struct Packer<const S: usize> {
	lower: Option<u8>,
	fullwidth: Option<u8>,
	clear: bool,
	no_spaces: bool,
	strip_comments: bool,
	comment_flag: bool,
	pos: usize,
	buffer: [u8; S],
}

impl<const S: usize> Default for Packer<S> {
	fn default() -> Self {
		Self {
			lower: None,
			fullwidth: None,
			clear: false,
			no_spaces: false,
			strip_comments: true,
			comment_flag: false,
			pos: 0,
			buffer: [0u8; S],
		}
	}
}

impl<const S: usize> Packer<S> {
	/// Return a header that needs to be place at the start of
	/// a meatpacked gcode stream.
	pub fn header(&self) -> [u8; 3] {
		[SIGNAL_BYTE, SIGNAL_BYTE, PACKING_ENABLED_BYTE]
	}

	/// Pack a byte into the current line.
	pub fn pack(
		&mut self,
		b: &u8,
	) -> Result<MeatPackResult, MeatPackError> {
		if self.clear {
			self.clear()
		}
		if self.strip_comments {
			if *b == COMMENT_START_BYTE {
				self.comment_flag = true;
			}
			if *b == LINEFEED_BYTE {
				self.comment_flag = false;
			}
			if self.comment_flag {
				return Ok(MeatPackResult::WaitingForNextByte);
			}
		}

		match (self.lower, b) {
			// Special case requiring \n\n.
			(None, 10) => {
				let upper_and_lower = forward_lookup(&10, self.no_spaces).unwrap();
				let p = self.pack_bytes(upper_and_lower, upper_and_lower);
				self.push(p)?;
				self.clear = true;
				// Remove empty lines.
				if self.pos > 1 {
					Ok(MeatPackResult::Line(self.return_slice()))
				} else {
					Ok(MeatPackResult::WaitingForNextByte)
				}
			}
			// Start of a new byte to pack.
			(None, b) => match forward_lookup(b, self.no_spaces) {
				// Packable byte
				Ok(lower) => {
					self.lower = Some(lower);
					self.fullwidth = None;
					Ok(MeatPackResult::WaitingForNextByte)
				}
				// Fullwidth byte
				Err(_) => {
					self.lower = Some(0b1111);
					self.fullwidth = Some(*b);
					Ok(MeatPackResult::WaitingForNextByte)
				}
			},
			// fullwidth + \n
			(Some(0b1111), 10) => {
				let upper = forward_lookup(&10, self.no_spaces).unwrap();
				let p = self.pack_bytes(upper, 0b1111);
				self.push(p)?;
				self.push(self.fullwidth.unwrap())?;
				self.lower = None;
				self.fullwidth = None;
				self.clear = true;
				Ok(MeatPackResult::Line(self.return_slice()))
			}
			// Full width + some other b byte that is not a \n
			(Some(0b1111), b) => match forward_lookup(b, self.no_spaces) {
				// Packable byte
				Ok(upper) => {
					let p = self.pack_bytes(upper, 0b1111);
					self.push(p)?;
					self.push(self.fullwidth.unwrap())?;
					self.lower = None;
					self.fullwidth = None;
					Ok(MeatPackResult::WaitingForNextByte)
				}
				// Fullwidth byte
				Err(_) => {
					let p = self.pack_bytes(0b1111, 0b1111);
					self.push(p)?;
					self.push(self.fullwidth.unwrap())?;
					self.push(*b)?;
					self.lower = None;
					self.fullwidth = None;
					Ok(MeatPackResult::WaitingForNextByte)
				}
			},
			// Some packable lower byte with a \n upper.
			(Some(lower), 10) => {
				let upper = forward_lookup(b, self.no_spaces).unwrap();
				let p = self.pack_bytes(upper, lower);
				self.push(p)?;
				self.lower = None;
				self.fullwidth = None;
				self.clear = true;
				Ok(MeatPackResult::Line(self.return_slice()))
			}
			// Lower is packable + whatever b is but not a \n
			(Some(lower), b) => match forward_lookup(b, self.no_spaces) {
				// Packable byte
				Ok(upper) => {
					let p = self.pack_bytes(upper, lower);
					self.push(p)?;
					self.lower = None;
					self.fullwidth = None;
					Ok(MeatPackResult::WaitingForNextByte)
				}
				// Fullwidth byte
				Err(_) => {
					let p = self.pack_bytes(0b1111, lower);
					self.push(p)?;
					self.push(*b)?;
					self.lower = None;
					self.fullwidth = None;
					Ok(MeatPackResult::WaitingForNextByte)
				}
			},
		}
	}

	/// Returns a slice of the filled elements in the buffer.
	fn return_slice(&mut self) -> &[u8] {
		&self.buffer[0..self.pos]
	}

	/// Clears the buffer
	fn clear(&mut self) {
		self.buffer.fill(0);
		self.pos = 0;
		self.clear = false;
	}

	/// Adds a byte to the buffer.
	fn push(
		&mut self,
		byte: u8,
	) -> Result<(), MeatPackError> {
		if self.pos > S {
			return Err(MeatPackError::BufferFull);
		}
		self.buffer[self.pos] = byte;
		self.pos += 1;
		Ok(())
	}

	// Pack two 4-bit representations together.
	fn pack_bytes(
		&self,
		upper: u8,
		lower: u8,
	) -> u8 {
		let packed = upper << 4;
		packed ^ lower
	}
}
