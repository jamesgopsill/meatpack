use core::slice::Iter;

use crate::meat::{
	determine_command, is_linefeed_byte, is_signal_byte, unpack_byte, MeatPackCommand,
	MeatPackError,
};

/// A struct that can unpack meatpack packed gcode where S is the size of the output buffer.
pub struct Unpack<const S: usize> {
	unpacking_enabled: bool,
	no_spaces_enabled: bool,
	buffer_pos: usize,
	buffer: [u8; S],
}

impl<const S: usize> Default for Unpack<S> {
	fn default() -> Self {
		Self {
			unpacking_enabled: false,
			no_spaces_enabled: false,
			buffer_pos: 0,
			buffer: [0u8; S],
		}
	}
}

impl<const S: usize> Unpack<S> {
	/// Evaluates a MeatPackCommand
	fn evaluate_cmd(
		&mut self,
		cmd: MeatPackCommand,
	) {
		match cmd {
			MeatPackCommand::PackingEnabled => {
				self.unpacking_enabled = true;
			}
			MeatPackCommand::PackingDisabled => {
				self.unpacking_enabled = false;
			}
			MeatPackCommand::ResetAll => {
				self.unpacking_enabled = false;
				self.no_spaces_enabled = false;
			}
			MeatPackCommand::QueryConfig => {}
			MeatPackCommand::NoSpacesEnabled => {
				self.no_spaces_enabled = true;
			}
			MeatPackCommand::NoSpacesDisabled => {
				self.no_spaces_enabled = false;
			}
			MeatPackCommand::SignalByte => {}
		}
	}

	/// Handles the unpacking of a byte
	fn handle_unpacking(
		&mut self,
		byte: &u8,
		iter: &mut Iter<'_, u8>,
	) -> Result<(u8, u8), MeatPackError> {
		let (upper, lower) = unpack_byte(byte, self.no_spaces_enabled)?;

		// If they are both unpacked characters.
		if upper != 0 && lower != 0 {
			return Ok((lower, upper));
		}

		// If lower contains 0b1111
		if lower == 0 {
			if let Some(fullwidth_byte) = iter.next() {
				return Ok((*fullwidth_byte, upper));
			} else {
				return Err(MeatPackError::EndOfSlice);
			}
		}

		// If upper contains 0b1111
		if upper == 0 {
			if let Some(fullwidth_byte) = iter.next() {
				return Ok((lower, *fullwidth_byte));
			} else {
				return Err(MeatPackError::EndOfSlice);
			}
		}

		// Should not get here
		Err(MeatPackError::InvalidByte)
	}

	/// Clears and resets the buffer location
	fn clear_buffer(&mut self) {
		self.buffer.fill(0);
		self.buffer_pos = 0;
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

	/// Returns a slice of the filled elements in the buffer.
	fn filled_elements(&mut self) -> &[u8] {
		&self.buffer[0..self.buffer_pos]
	}

	/// Unpacks a gcode slice.
	pub fn unpack(
		&mut self,
		slice: &[u8],
	) -> Result<&[u8], MeatPackError> {
		self.clear_buffer();
		let mut iter = slice.iter();

		while let Some(byte) = iter.next() {
			// Handle the command byte action
			match is_signal_byte(byte) {
				true => {
					// Read the next two bytes
					let left = iter.next();
					if left.is_none() {
						return Err(MeatPackError::EndOfSlice);
					}
					let left = left.unwrap();

					let right = iter.next();
					if right.is_none() {
						return Err(MeatPackError::EndOfSlice);
					}
					let right = right.unwrap();

					match is_signal_byte(left) {
						true => {
							let cmd = determine_command(right);
							if cmd.is_err() {
								return Err(cmd.err().unwrap());
							}
							let cmd = cmd.unwrap();
							self.evaluate_cmd(cmd);
						}
						// Pass-through two;
						false => {
							// Could put some checks on these pass through bytes. I don't think they should contain \n, for example.
							self.push_buffer(left)?;
							self.push_buffer(right)?;
							continue;
						}
					}
				}
				false => match self.unpacking_enabled {
					// Unpack the 2 x 4-bits packed in the 8-bit.
					true => {
						let (left, right) = self.handle_unpacking(byte, &mut iter)?;
						self.push_buffer(&left)?;
						if is_linefeed_byte(&left) {
							continue;
						}
						self.push_buffer(&right)?;
						continue;
					}
					// Append the fullwidth character.
					false => {
						self.push_buffer(byte)?;
						continue;
					}
				},
			}
		}

		Ok(self.filled_elements())
	}
}

/*

/// A struct that can unpack meatpack packed gcode where S is the size of the output buffer.
pub struct Unpack<'a, const S: usize> {
	unpacking_enabled: bool,
	no_spaces_enabled: bool,
	buffer_pos: usize,
	buffer: [u8; S],
	//slice_pos: usize,
	//slice: &'a [u8],
}

impl<'a, const S: usize> Unpack<'a, S> {
	/// Create a new instance of the unpacker with an
	/// initial slice to unpack. N.b. The slice should end
	/// with a meatpack newline character (`0b1100`)
	pub fn new(slice: &'a [u8]) -> Self {
		Self {
			unpacking_enabled: false,
			no_spaces_enabled: false,
			buffer_pos: 0,
			buffer: [0; S],
			slice_pos: 0,
			slice,
		}
	}

	/// Add a slice to process.
	pub fn add_slice(
		&mut self,
		slice: &'a [u8],
	) {
		self.slice = slice;
		self.slice_pos = 0;
	}

	/// Read a byte from the slice
	fn read_byte(&mut self) -> Option<u8> {
		if self.slice_pos == self.slice.len() {
			return None;
		}
		let byte = self.slice[self.slice_pos];
		self.slice_pos += 1;
		Some(byte)
	}

	/// Evaluates a MeatPackCommand
	fn evaluate_cmd(
		&mut self,
		cmd: MeatPackCommand,
	) {
		match cmd {
			MeatPackCommand::PackingEnabled => {
				self.unpacking_enabled = true;
			}
			MeatPackCommand::PackingDisabled => {
				self.unpacking_enabled = false;
			}
			MeatPackCommand::ResetAll => {
				self.unpacking_enabled = false;
				self.no_spaces_enabled = false;
			}
			MeatPackCommand::QueryConfig => {}
			MeatPackCommand::NoSpacesEnabled => {
				self.no_spaces_enabled = true;
			}
			MeatPackCommand::NoSpacesDisabled => {
				self.no_spaces_enabled = false;
			}
			MeatPackCommand::SignalByte => {}
		}
	}

	/// Handles the unpacking of a byte
	fn handle_unpacking(
		&mut self,
		byte: &u8,
	) -> Result<(u8, u8), MeatPackError> {
		let (upper, lower) = unpack_byte(byte, self.no_spaces_enabled)?;

		// If they are both unpacked characters.
		if upper != 0 && lower != 0 {
			return Ok((lower, upper));
		}

		// If lower contains 0b1111
		if lower == 0 {
			if let Some(fullwidth_byte) = self.read_byte() {
				return Ok((fullwidth_byte, upper));
			} else {
				return Err(MeatPackError::EndOfSlice);
			}
		}

		// If upper contains 0b1111
		if upper == 0 {
			if let Some(fullwidth_byte) = self.read_byte() {
				return Ok((lower, fullwidth_byte));
			} else {
				return Err(MeatPackError::EndOfSlice);
			}
		}

		// Should not get here
		Err(MeatPackError::InvalidByte)
	}

	/// Clears and resets the buffer location
	fn clear_buffer(&mut self) {
		self.buffer.fill(0);
		self.buffer_pos = 0;
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

	/// Returns a slice of the filled elements in the buffer.
	fn filled_elements(&mut self) -> &[u8] {
		&self.buffer[0..self.buffer_pos]
	}

	/// Yields unpacked lines of gcode.
	///
	/// ```rust
	/// while let Some(line) = unpacker.unpack_line() {
	///     match line {
	///         Ok(line) => {/*Do something with the line.*/}
	///         Err(e) => {/*Handle the error in the line.*/}
	///     }
	/// }
	/// ```
	pub fn unpack_cmd(&mut self) -> Option<Result<&[u8], MeatPackError>> {
		self.clear_buffer();

		while let Some(byte) = self.read_byte() {
			// Handle the command byte action
			match is_signal_byte(&byte) {
				true => {
					// Read the next two bytes
					let left = self.read_byte();
					if left.is_none() {
						return Some(Err(MeatPackError::EndOfSlice));
					}
					let left = left.unwrap();

					let right = self.read_byte();
					if right.is_none() {
						return Some(Err(MeatPackError::EndOfSlice));
					}
					let right = right.unwrap();

					match is_signal_byte(&left) {
						true => {
							let cmd = determine_command(&right);
							if cmd.is_err() {
								return Some(Err(cmd.err().unwrap()));
							}
							let cmd = cmd.unwrap();
							self.evaluate_cmd(cmd);
						}
						// Pass-through two;
						false => {
							// Could put some checks on these pass through bytes. I don't think they should contain \n, for example.
							//
							if let Err(e) = self.push_buffer(&left) {
								return Some(Err(e));
							};
							if let Err(e) = self.push_buffer(&right) {
								return Some(Err(e));
							};
							continue;
						}
					}
				}
				false => match self.unpacking_enabled {
					// Unpack the 2 x 4-bits packed in the 8-bit.
					true => match self.handle_unpacking(&byte) {
						Ok((left, right)) => {
							if is_linefeed_byte(&left) {
								// Right should also be \n to meet the \n\n
								// expectation.
								// Otherwise we have invalid meatpack gcode.
								if is_linefeed_byte(&right) {
									return Some(Ok(self.filled_elements()));
								}
								return Some(Err(MeatPackError::InvalidByte));
							} else {
								if let Err(e) = self.push_buffer(&left) {
									return Some(Err(e));
								};
								if is_linefeed_byte(&right) {
									return Some(Ok(self.filled_elements()));
								}
								if let Err(e) = self.push_buffer(&right) {
									return Some(Err(e));
								};
								continue;
							}
						}
						Err(e) => return Some(Err(e)),
					},
					// Append the fullwidth character.
					false => {
						if is_linefeed_byte(&byte) {
							return Some(Ok(self.filled_elements()));
						} else {
							if let Err(e) = self.push_buffer(&byte) {
								return Some(Err(e));
							};
							continue;
						}
					}
				},
			}
		}
		// Empty if the buf is not empty at the end of reading the bytes.
		if self.buffer[0] > 0 {
			return Some(Ok(self.filled_elements()));
		}
		None
	}
}


*/
