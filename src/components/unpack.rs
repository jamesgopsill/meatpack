use crate::components::meat::{
    MeatPackCommand, MeatPackError, Pack, determine_command, is_signal_byte,
};

#[cfg(feature = "std")]
extern crate std;

/// A list of state the Unpacker struct can exist in.
#[derive(Debug)]
enum UnpackerInternalState {
    FirstCommandByte,
    SecondCommandByte,
    RightFullWidthByte,
    LeftFullWidthByte,
    Enabled,
    Disabled,
}

/// Reports whether we have unpacked some bytes or
/// reached a new line.
pub enum UnpackerState {
    Unpacked(usize),
    Line(usize),
}

/// A  struct for that unpacks bytes and emits
/// lines of gcode.
pub struct Unpacker {
    state: UnpackerInternalState,
    strip_whitespace: bool,
    held_back: u8,
}

impl Default for Unpacker {
    /// The default implementation of the unpacker.
    fn default() -> Self {
        Self {
            state: UnpackerInternalState::Disabled,
            strip_whitespace: false,
            held_back: 0,
        }
    }
}

impl Unpacker {
    /// Internal unpacking function.
    fn unpack_byte(
        &mut self,
        byte: u8,
        writer: &mut impl embedded_io::Write,
    ) -> Result<UnpackerState, MeatPackError> {
        // First check if it is a signal byte
        // and handle the scenarios.
        if is_signal_byte(byte) {
            match self.state {
                UnpackerInternalState::FirstCommandByte => {
                    self.state = UnpackerInternalState::SecondCommandByte;
                    return Ok(UnpackerState::Unpacked(0));
                }
                UnpackerInternalState::Disabled => {
                    self.state = UnpackerInternalState::FirstCommandByte;
                    return Ok(UnpackerState::Unpacked(0));
                }
                UnpackerInternalState::Enabled => {
                    self.state = UnpackerInternalState::FirstCommandByte;
                    return Ok(UnpackerState::Unpacked(0));
                }
                _ => {
                    return Err(MeatPackError::InvalidState);
                }
            }
        }

        // Handle non signal scenarios.
        match self.state {
            UnpackerInternalState::Disabled => {
                // If a normal new line.
                // Return the line for further processing.
                writer.write_all(&[byte])?;
                if byte == 10 {
                    Ok(UnpackerState::Line(1))
                } else {
                    Ok(UnpackerState::Unpacked(1))
                }
            }
            UnpackerInternalState::Enabled => {
                let (most, least) = byte.unpack(self.strip_whitespace);

                // most, least
                // Check if we need to wait for a
                // fullwidth byte.
                match (most, least) {
                    // \n\n packed byte. Just return one \n
                    (10, 10) => {
                        writer.write_all(&[10])?;
                        Ok(UnpackerState::Line(1))
                    }
                    // most is a full width byte
                    (0, 1..) => {
                        writer.write_all(&[least])?;
                        self.state = UnpackerInternalState::RightFullWidthByte;
                        if least == 10 {
                            Ok(UnpackerState::Line(1))
                        } else {
                            Ok(UnpackerState::Unpacked(1))
                        }
                    }
                    // least is a full width byte
                    (1.., 0) => {
                        // Note. need to wait for the next byte to then insert
                        // them in the right order.
                        self.state = UnpackerInternalState::LeftFullWidthByte;
                        self.held_back = most;
                        Ok(UnpackerState::Unpacked(0))
                    }
                    // Should be dealt with by the command bytes section.
                    (0, 0) => {
                        unreachable!();
                    }
                    // Two unpacked packable bytes.
                    (most, least) => {
                        writer.write_all(&[least, most])?;
                        if most == 10 {
                            Ok(UnpackerState::Line(2))
                        } else {
                            Ok(UnpackerState::Unpacked(2))
                        }
                    }
                }
            }
            UnpackerInternalState::SecondCommandByte => {
                let cmd = determine_command(byte)?;
                self.handle_command(cmd);
                Ok(UnpackerState::Unpacked(0))
            }
            UnpackerInternalState::FirstCommandByte => {
                self.state = UnpackerInternalState::RightFullWidthByte;
                writer.write_all(&[byte])?;
                Ok(UnpackerState::Unpacked(1))
            }
            UnpackerInternalState::RightFullWidthByte => {
                self.state = UnpackerInternalState::Enabled;
                writer.write_all(&[byte])?;
                if byte == 10 {
                    Ok(UnpackerState::Line(1))
                } else {
                    Ok(UnpackerState::Unpacked(1))
                }
            }
            UnpackerInternalState::LeftFullWidthByte => {
                self.state = UnpackerInternalState::Enabled;
                writer.write_all(&[byte, self.held_back])?;
                if self.held_back == 10 {
                    Ok(UnpackerState::Line(2))
                } else {
                    Ok(UnpackerState::Unpacked(2))
                }
            }
        }
    }

    /// Unpacks a line of `gcode` from the meatpack `reader`. Useful in
    /// `no_std` where space is limited and you want to operate on each
    /// line. Note that `writer` is not buffered so you need to decide whether
    /// you want to provide a buffered writer to the function.
    pub fn unpack_line(
        &mut self,
        reader: &mut impl embedded_io::BufRead,
        writer: &mut impl embedded_io::Write,
    ) -> Result<usize, MeatPackError> {
        let mut written: usize = 0;
        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            for (i, byte) in buf.iter().copied().enumerate() {
                match self.unpack_byte(byte, writer)? {
                    UnpackerState::Unpacked(s) => written += s,
                    UnpackerState::Line(s) => {
                        written += s;
                        let read = i + 1;
                        reader.consume(read);
                        writer.flush()?;
                        return Ok(written);
                    }
                };
            }
            let read = buf.len();
            reader.consume(read);
        }
        writer.flush()?;
        Ok(written)
    }

    /// Unpacks the `gcode` from the meatpack `reader` into the `writer`.
    /// Note that `writer` is not buffered so you need to decide whether
    /// you want to provide a buffered writer to the function. The function
    /// returns the bytes unpacked into `writer`.
    pub fn unpack(
        &mut self,
        reader: &mut impl embedded_io::BufRead,
        writer: &mut impl embedded_io::Write,
    ) -> Result<usize, MeatPackError> {
        let mut written: usize = 0;
        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            for byte in buf.iter().copied() {
                match self.unpack_byte(byte, writer)? {
                    UnpackerState::Unpacked(s) => written += s,
                    UnpackerState::Line(s) => written += s,
                };
            }
            let read = buf.len();
            reader.consume(read);
        }
        writer.flush()?;
        Ok(written)
    }

    /// Handles the command byte combinations that
    /// exist in the meatpack spec.
    fn handle_command(
        &mut self,
        cmd: MeatPackCommand,
    ) {
        match cmd {
            MeatPackCommand::PackingEnabled => {
                self.state = UnpackerInternalState::Enabled;
            }
            MeatPackCommand::PackingDisabled => {
                self.state = UnpackerInternalState::Disabled;
            }
            MeatPackCommand::ResetAll => {
                self.state = UnpackerInternalState::Disabled;
                self.strip_whitespace = false;
            }
            MeatPackCommand::QueryConfig => {}
            MeatPackCommand::NoSpacesEnabled => {
                self.strip_whitespace = true;
                self.state = UnpackerInternalState::Enabled;
            }
            MeatPackCommand::NoSpacesDisabled => {
                self.strip_whitespace = false;
                self.state = UnpackerInternalState::Enabled;
            }
            MeatPackCommand::SignalByte => {}
        }
    }
}
