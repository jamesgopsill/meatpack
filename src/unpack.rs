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
