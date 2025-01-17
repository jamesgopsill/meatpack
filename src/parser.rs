use crate::meat::{determine_command, is_signal_byte, unpack_byte, MeatPackCommand, MeatPackError};

pub enum MeatPackOutput<'a> {
	WaitingForNextByte,
	Line(&'a [u8]),
}

#[derive(Debug)]
pub enum ParserState {
	FirstCommandByte,
	SecondCommandByte,
	RightFullWidthByte,
	LeftFullWidthByte,
	Enabled,
	Disabled,
}

pub struct Parser<const S: usize> {
	state: ParserState,
	no_spaces: bool,
	clear: bool,
	pos: usize,
	buffer: [u8; S],
}

impl<const S: usize> Default for Parser<S> {
	fn default() -> Self {
		Self {
			state: ParserState::Disabled,
			no_spaces: false,
			clear: false,
			pos: 0,
			buffer: [0u8; S],
		}
	}
}

impl<const S: usize> Parser<S> {
	pub fn parse_byte(
		&mut self,
		byte: &u8,
	) -> Result<MeatPackOutput, MeatPackError> {
		println!("{:?}", self.state);
		if self.clear {
			self.clear()
		}

		// First check if it is a signal byte and handle the
		// scenarios.
		if is_signal_byte(byte) {
			match self.state {
				ParserState::FirstCommandByte => {
					self.state = ParserState::SecondCommandByte;
					return Ok(MeatPackOutput::WaitingForNextByte);
				}
				ParserState::Disabled => {
					self.state = ParserState::FirstCommandByte;
					return Ok(MeatPackOutput::WaitingForNextByte);
				}
				ParserState::Enabled => {
					self.state = ParserState::FirstCommandByte;
					return Ok(MeatPackOutput::WaitingForNextByte);
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
			ParserState::Disabled => {
				// If a normal new line.
				// Return the line for further processing.
				self.push(byte)?;
				if *byte == 10 {
					self.clear = true; // clear buffer next time round.
					Ok(MeatPackOutput::Line(self.return_slice()))
				} else {
					Ok(MeatPackOutput::WaitingForNextByte)
				}
			}
			ParserState::Enabled => {
				let bytes = unpack_byte(byte, self.no_spaces)?;

				self.push(&bytes.1)?; // lower
				self.push(&bytes.0)?; // upper

				// Upper, lower
				// Check if we need to wait for a
				// fullwidth byte.
				match bytes {
					(0, 1..) => {
						self.state = ParserState::RightFullWidthByte;
					}
					(1.., 0) => {
						self.state = ParserState::LeftFullWidthByte;
					}
					(0, 0) => {
						return Err(MeatPackError::InvalidByte);
					}
					(_, _) => {}
				}

				// Check if we're ready to check the last byte
				// for a return value.
				match self.state {
					ParserState::Enabled => {
						if self.buffer[self.pos - 1] == 10 {
							self.clear = true; // clear buffer next time round.
							return Ok(MeatPackOutput::Line(self.return_slice()));
						}
						Ok(MeatPackOutput::WaitingForNextByte)
					}
					_ => Ok(MeatPackOutput::WaitingForNextByte),
				}
			}
			ParserState::SecondCommandByte => {
				let cmd = determine_command(byte)?;
				self.handle_command(cmd);
				Ok(MeatPackOutput::WaitingForNextByte)
			}
			ParserState::FirstCommandByte => {
				self.state = ParserState::RightFullWidthByte;
				self.push(byte)?;
				Ok(MeatPackOutput::WaitingForNextByte)
			}
			ParserState::RightFullWidthByte => {
				self.state = ParserState::Enabled;
				self.push(byte)?;
				Ok(MeatPackOutput::WaitingForNextByte)
			}
			ParserState::LeftFullWidthByte => {
				self.state = ParserState::Enabled;
				self.buffer[self.pos - 2] = *byte;
				if self.buffer[self.pos - 1] == 10 {
					self.clear = true; // clear buffer next time round.
					return Ok(MeatPackOutput::Line(self.return_slice()));
				}
				Ok(MeatPackOutput::WaitingForNextByte)
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
				self.state = ParserState::Enabled;
			}
			MeatPackCommand::PackingDisabled => {
				self.state = ParserState::Disabled;
			}
			MeatPackCommand::ResetAll => {
				self.state = ParserState::Disabled;
				self.no_spaces = false;
			}
			MeatPackCommand::QueryConfig => {}
			MeatPackCommand::NoSpacesEnabled => {
				self.no_spaces = true;
				self.state = ParserState::Enabled;
			}
			MeatPackCommand::NoSpacesDisabled => {
				self.no_spaces = false;
				self.state = ParserState::Enabled;
			}
			MeatPackCommand::SignalByte => {}
		}
	}
}
