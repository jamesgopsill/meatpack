use core::slice::Iter;

use crate::meat::{
	forward_lookup, MeatPackError, LINEFEED_BYTE, PACKING_ENABLED_BYTE, SIGNAL_BYTE,
};

pub struct Pack<'a, const S: usize> {
	buffer_pos: usize,
	buffer: [u8; S],
	iter: Iter<'a, u8>,
}

impl<'a, const S: usize> Pack<'a, S> {
	pub fn new(slice: &'a [u8]) -> Self {
		let iter = slice.iter();
		Self {
			buffer_pos: 0,
			buffer: [0; S],
			iter,
		}
	}

	/// Pushed a byte to the buffer.
	fn push_buffer(
		&mut self,
		byte: &u8,
	) -> Result<(), MeatPackError> {
		if self.buffer_pos > S {
			return Err(MeatPackError::BufferFull);
		}
		self.buffer[self.buffer_pos] = *byte;
		self.buffer_pos += 1;
		Ok(())
	}

	/// Clears and resets the buffer location
	fn clear_buffer(&mut self) {
		self.buffer.fill(0);
		self.buffer_pos = 0;
	}

	/// Returns a slice of the filled elements in the buffer.
	fn filled_elements(&mut self) -> &[u8] {
		&self.buffer[0..self.buffer_pos]
	}

	pub fn header(&self) -> [u8; 3] {
		[SIGNAL_BYTE, SIGNAL_BYTE, PACKING_ENABLED_BYTE]
	}

	pub fn pack_line(&mut self) -> Option<Result<&[u8], MeatPackError>> {
		self.clear_buffer();

		// Now read in the data from inner.
		while let Some(byte_one) = self.iter.next() {
			// Check if the byte is \n and the lower is not populated
			// so we add a \n\n packed byte.
			if *byte_one == LINEFEED_BYTE {
				if let Err(e) = self.push_buffer(&204) {
					return Some(Err(e));
				};
				return Some(Ok(self.filled_elements()));
			}

			let mut linefeed_flag = false;
			let byte_two = self.iter.next();
			if byte_two.is_none() {
				return Some(Err(MeatPackError::EmptySlice));
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
					if let Err(e) = self.push_buffer(&packed) {
						return Some(Err(e));
					};
				}
				(Ok(lower), Err(_)) => {
					let upper: u8 = 0b1111;
					let packed = upper << 4;
					let packed = packed ^ lower;
					if let Err(e) = self.push_buffer(&packed) {
						return Some(Err(e));
					};
					if let Err(e) = self.push_buffer(byte_two) {
						return Some(Err(e));
					};
				}
				(Err(_), Ok(upper)) => {
					let lower: u8 = 0b1111;
					let packed = upper << 4;
					let packed = packed ^ lower;
					if let Err(e) = self.push_buffer(&packed) {
						return Some(Err(e));
					};
					if let Err(e) = self.push_buffer(byte_one) {
						return Some(Err(e));
					};
				}
				// Both not packable so needs a signal byte in front
				(Err(_), Err(_)) => {
					if let Err(e) = self.push_buffer(&SIGNAL_BYTE) {
						return Some(Err(e));
					};
					if let Err(e) = self.push_buffer(byte_one) {
						return Some(Err(e));
					};
					if let Err(e) = self.push_buffer(byte_two) {
						return Some(Err(e));
					};
				}
			}

			if linefeed_flag {
				return Some(Ok(self.filled_elements()));
			}
		}

		// Empty if the buf is not empty at the end of reading the bytes.
		if self.buffer[0] > 0 {
			return Some(Ok(self.filled_elements()));
		}
		None
	}
}
