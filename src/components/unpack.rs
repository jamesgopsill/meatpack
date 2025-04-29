use crate::components::meat::{
    determine_command, is_signal_byte, MeatPackCommand, MeatPackError, MeatPackResult, Pack,
};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A list of state the Unpacker struct can exist in.
#[derive(Debug)]
pub enum UnpackerState {
    FirstCommandByte,
    SecondCommandByte,
    RightFullWidthByte,
    LeftFullWidthByte,
    Enabled,
    Disabled,
}

/// A  struct for that unpacks bytes and emits
/// lines of gcode.
pub struct Unpacker<const S: usize> {
    state: UnpackerState,
    no_spaces: bool,
    clear: bool,
    pos: usize,
    inner: [u8; S],
}

impl<const S: usize> Default for Unpacker<S> {
    /// The default implementation of the unpacker.
    fn default() -> Self {
        Self {
            state: UnpackerState::Disabled,
            no_spaces: false,
            clear: false,
            pos: 0,
            inner: [0u8; S],
        }
    }
}

impl<const S: usize> Unpacker<S> {
    /// Unpacks a single meatpacked byte checking on the
    /// history of the previously unpacked items. It returns
    /// detailing what it is waiting for next.
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
                    self.inner.fill(0);
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
                let (upper, lower) = byte.unpack(self.no_spaces)?;

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
                        if self.inner[self.pos - 1] == 10 && self.pos > 1 {
                            self.clear = true; // clear buffer next time round.
                            return Ok(MeatPackResult::Line(self.return_slice()));
                        }
                        // empty line
                        if self.inner[self.pos - 1] == 10 {
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
                self.inner[self.pos - 2] = *byte;
                if self.inner[self.pos - 1] == 10 {
                    self.clear = true; // clear buffer next time round.
                    return Ok(MeatPackResult::Line(self.return_slice()));
                }
                Ok(MeatPackResult::WaitingForNextByte)
            }
        }
    }

    /// Clears the internal buffer and resets the
    /// write position into the internal buffer.
    fn clear(&mut self) {
        self.inner.fill(0);
        self.pos = 0;
        self.clear = false;
    }

    /// Returns a slice of the filled elements in the buffer.
    fn return_slice(&mut self) -> &[u8] {
        &self.inner[0..self.pos]
    }

    /// Push a byte to the internal buffer.
    fn push(
        &mut self,
        byte: &u8,
    ) -> Result<(), MeatPackError> {
        if self.pos > S {
            return Err(MeatPackError::BufferFull);
        }
        self.inner[self.pos] = *byte;
        self.pos += 1;
        Ok(())
    }

    /// Handles the command byte combinations that
    /// exist in the meatpack spec.
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

    /// A utility function to check if any data remains
    /// in the internal buffer. We expect all meatpack
    /// lines to newline end.
    pub fn data_remains(&self) -> bool {
        if !self.clear || self.pos == 0 {
            return true;
        }
        false
    }

    /// A convenience function around unpacker that enables you
    /// to simply unpack meapacked data from a slice to a vec.
    #[cfg(feature = "alloc")]
    pub fn unpack_slice(
        in_buf: &[u8],
        out_buf: &mut Vec<u8>,
    ) -> Result<(), MeatPackError> {
        let mut unpacker = Unpacker::<S>::default();
        for b in in_buf {
            match unpacker.unpack(b) {
                Ok(MeatPackResult::Line(line)) => out_buf.extend(line),
                Ok(MeatPackResult::WaitingForNextByte) => {}
                Err(e) => return Err(e),
            }
        }
        // if the packer is in the state of clearing itself
        // on the next iteration then ignore as we hit a new line.
        // Otherwise we have an unterminated line with some
        // data possibly stuck in the buffer and we're expecting
        // to end with a terminated line so throw an err.
        if unpacker.data_remains() {
            return Err(MeatPackError::UnterminatedLine(unpacker.pos));
        }
        Ok(())
    }
}
