use crate::meat::{
	determine_command, is_signal_byte, unpack_byte, MeatPackCommand, MeatPackError, MeatPackResult,
};

#[derive(Debug)]
pub enum UnpackerState {
	FirstCommandByte,
	SecondCommandByte,
	RightFullWidthByte,
	LeftFullWidthByte,
	Enabled,
	Disabled,
}

pub struct Unpacker<const S: usize> {
	state: UnpackerState,
	no_spaces: bool,
	clear: bool,
	pos: usize,
	buffer: [u8; S],
}

impl<const S: usize> Default for Unpacker<S> {
	fn default() -> Self {
		Self {
			state: UnpackerState::Disabled,
			no_spaces: false,
			clear: false,
			pos: 0,
			buffer: [0u8; S],
		}
	}
}

impl<const S: usize> Unpacker<S> {
	pub fn unpack(
		&mut self,
		byte: &u8,
	) -> Result<MeatPackResult, MeatPackError> {
		if self.clear {
			self.clear()
		}

		// First check if it is a signal byte
		// and handle the scenarios.
		if is_signal_byte(byte) {
			match self.state {
				UnpackerState::FirstCommandByte => {
					self.state = UnpackerState::SecondCommandByte;
					return Ok(MeatPackResult::WaitingForNextByte);
				}
				UnpackerState::Disabled => {
					self.state = UnpackerState::FirstCommandByte;
					return Ok(MeatPackResult::WaitingForNextByte);
				}
				UnpackerState::Enabled => {
					self.state = UnpackerState::FirstCommandByte;
					return Ok(MeatPackResult::WaitingForNextByte);
				}
				_ => {
					self.pos = 0;
					self.buffer.fill(0);
					return Err(MeatPackError::InvalidState);
				}
			}
		}

		// Handle non signal scenarios.
		match self.state {
			UnpackerState::Disabled => {
				// If a normal new line.
				// Return the line for further processing.
				self.push(byte)?;
				if *byte == 10 {
					self.clear = true; // clear buffer next time round.
					Ok(MeatPackResult::Line(self.return_slice()))
				} else {
					Ok(MeatPackResult::WaitingForNextByte)
				}
			}
			UnpackerState::Enabled => {
				let (upper, lower) = unpack_byte(byte, self.no_spaces)?;

				// Upper, lower
				// Check if we need to wait for a
				// fullwidth byte.
				match (upper, lower) {
					// \n\n packed byte. Just return one \n
					(10, 10) => {
						self.push(&10)?;
					}
					// upper is a full width byte
					(0, 1..) => {
						self.push(&lower)?;
						self.state = UnpackerState::RightFullWidthByte;
					}
					// lower is a full width byte
					(1.., 0) => {
						self.push(&lower)?;
						self.push(&upper)?;
						self.state = UnpackerState::LeftFullWidthByte;
					}
					// Should be dealt with by the command bytes section.
					(0, 0) => {
						return Err(MeatPackError::InvalidByte);
					}
					// Two unpacked packable bytes.
					(upper, lower) => {
						self.push(&lower)?;
						self.push(&upper)?;
					}
				}

				// Check if we're ready to check the last byte
				// for a return value.
				match self.state {
					UnpackerState::Enabled => {
						if self.buffer[self.pos - 1] == 10 && self.pos > 1 {
							self.clear = true; // clear buffer next time round.
							return Ok(MeatPackResult::Line(self.return_slice()));
						}
						// empty line
						if self.buffer[self.pos - 1] == 10 {
							self.clear = true; // clear buffer next time round.
						}
						Ok(MeatPackResult::WaitingForNextByte)
					}
					_ => Ok(MeatPackResult::WaitingForNextByte),
				}
			}
			UnpackerState::SecondCommandByte => {
				let cmd = determine_command(byte)?;
				self.handle_command(cmd);
				Ok(MeatPackResult::WaitingForNextByte)
			}
			UnpackerState::FirstCommandByte => {
				self.state = UnpackerState::RightFullWidthByte;
				self.push(byte)?;
				Ok(MeatPackResult::WaitingForNextByte)
			}
			UnpackerState::RightFullWidthByte => {
				self.state = UnpackerState::Enabled;
				self.push(byte)?;
				Ok(MeatPackResult::WaitingForNextByte)
			}
			UnpackerState::LeftFullWidthByte => {
				self.state = UnpackerState::Enabled;
				self.buffer[self.pos - 2] = *byte;
				if self.buffer[self.pos - 1] == 10 {
					self.clear = true; // clear buffer next time round.
					return Ok(MeatPackResult::Line(self.return_slice()));
				}
				Ok(MeatPackResult::WaitingForNextByte)
			}
		}
	}

	fn clear(&mut self) {
		self.buffer.fill(0);
		self.pos = 0;
		self.clear = false;
	}

	/// Returns a slice of the filled elements in the buffer.
	fn return_slice(&mut self) -> &[u8] {
		&self.buffer[0..self.pos]
	}

	fn push(
		&mut self,
		byte: &u8,
	) -> Result<(), MeatPackError> {
		if self.pos > S {
			return Err(MeatPackError::BufferFull);
		}
		self.buffer[self.pos] = *byte;
		self.pos += 1;
		Ok(())
	}

	fn handle_command(
		&mut self,
		cmd: MeatPackCommand,
	) {
		match cmd {
			MeatPackCommand::PackingEnabled => {
				self.state = UnpackerState::Enabled;
			}
			MeatPackCommand::PackingDisabled => {
				self.state = UnpackerState::Disabled;
			}
			MeatPackCommand::ResetAll => {
				self.state = UnpackerState::Disabled;
				self.no_spaces = false;
			}
			MeatPackCommand::QueryConfig => {}
			MeatPackCommand::NoSpacesEnabled => {
				self.no_spaces = true;
				self.state = UnpackerState::Enabled;
			}
			MeatPackCommand::NoSpacesDisabled => {
				self.no_spaces = false;
				self.state = UnpackerState::Enabled;
			}
			MeatPackCommand::SignalByte => {}
		}
	}
}
