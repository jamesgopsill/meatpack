use no_std_io::io::{BufReader, Read};

use crate::core::{
    determine_command, is_linefeed_byte, is_signal_byte, unpack_byte, MeatPackCommand,
    MeatPackError,
};

/// An iterator over a reader that yields unpacked gcode lines.
pub struct UnpackedLines<'a, R> {
    // Keep track of meatpack config.
    unpacking_enabled: bool,
    no_spaces_enabled: bool,
    buffer_pos: u8,
    buffer: [u8; 128],
    #[cfg(feature = "std")]
    reader: &'a mut BufReader<R>,
    #[cfg(feature = "no_std")]
    reader: &'a mut BufReader<R, 128>,
}

impl<'a, R: Read> UnpackedLines<'a, R> {
    #[cfg(feature = "no_std")]
    pub fn new(reader: &'a mut BufReader<R, 128>) -> Self {
        Self {
            reader,
            unpacking_enabled: false,
            no_spaces_enabled: false,
            buffer_pos: 0,
            buffer: [0; 128],
        }
    }

    #[cfg(feature = "std")]
    pub fn new(reader: &'a mut BufReader<R>) -> Self {
        Self {
            reader,
            unpacking_enabled: false,
            no_spaces_enabled: false,
            buffer_pos: 0,
            buffer: [0; 128],
        }
    }

    /// Reads a single byte from the reader
    fn read_one_byte(&mut self) -> Result<u8, MeatPackError> {
        let mut byte: [u8; 1] = [0];
        match self.reader.read_exact(&mut byte) {
            Ok(_) => return Ok(byte[0]),
            Err(e) => return Err(MeatPackError::IOError(e)),
        }
    }

    /// Evaluates a MeatPackCommand
    fn evaluate_cmd(&mut self, cmd: MeatPackCommand) {
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
    fn handle_unpacking(&mut self, byte: u8) -> Result<(u8, u8), MeatPackError> {
        let (upper, lower) = unpack_byte(&byte, self.no_spaces_enabled)?;

        // If they are both unpacked characters.
        if upper != 0 && lower != 0 {
            return Ok((lower, upper));
        }

        // If lower contains 0b1111
        if lower == 0 {
            let fullwidth_byte = self.read_one_byte()?;
            return Ok((fullwidth_byte, upper));
        }

        // If upper contains 0b1111
        if upper == 0 {
            let fullwidth_byte = self.read_one_byte()?;
            return Ok((lower, fullwidth_byte));
        }

        // Should not get here
        return Err(MeatPackError::InvalidByte);
    }

    /// Clears and resets the buffer location
    fn clear_buffer(&mut self) {
        self.buffer.fill(0);
        self.buffer_pos = 0;
    }

    /// Pushed a byte to the buffer.
    fn push_buffer(&mut self, byte: u8) -> Result<(), MeatPackError> {
        if self.buffer_pos > 127 {
            return Err(MeatPackError::BufferFull);
        }
        self.buffer[self.buffer_pos as usize] = byte;
        self.buffer_pos += 1;
        return Ok(());
    }
}

impl<R: Read> Iterator for UnpackedLines<'_, R> {
    type Item = Result<[u8; 128], MeatPackError>;

    /// Implementation of the next function to yield lines of
    /// unpacked gcode.
    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(byte) = self.read_one_byte() {
            // Handle the command byte action
            match is_signal_byte(&byte) {
                true => {
                    // Read the next two bytes
                    let left = self.read_one_byte();
                    if left.is_err() {
                        return Some(Err(left.err().unwrap()));
                    }
                    let left = left.unwrap();
                    let right = self.read_one_byte();
                    if right.is_err() {
                        return Some(Err(right.err().unwrap()));
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
                            if is_linefeed_byte(&left) {
                                let line = self.buffer.clone();
                                self.clear_buffer();
                                if !is_linefeed_byte(&right) {
                                    if let Err(e) = self.push_buffer(right) {
                                        return Some(Err(e));
                                    };
                                }
                                return Some(Ok(line));
                            } else {
                                if let Err(e) = self.push_buffer(left) {
                                    return Some(Err(e));
                                };
                                if is_linefeed_byte(&right) {
                                    let line = self.buffer.clone();
                                    self.clear_buffer();
                                    return Some(Ok(line));
                                }
                                if let Err(e) = self.push_buffer(right) {
                                    return Some(Err(e));
                                };
                                continue;
                            }
                        }
                    }
                }
                false => match self.unpacking_enabled {
                    // Unpack the 2 x 4-bits packed in the 8-bit.
                    true => match self.handle_unpacking(byte) {
                        Ok((left, right)) => {
                            if is_linefeed_byte(&left) {
                                let line = self.buffer.clone();
                                self.clear_buffer();
                                if !is_linefeed_byte(&right) {
                                    if let Err(e) = self.push_buffer(right) {
                                        return Some(Err(e));
                                    };
                                }
                                return Some(Ok(line));
                            } else {
                                if let Err(e) = self.push_buffer(left) {
                                    return Some(Err(e));
                                };
                                if is_linefeed_byte(&right) {
                                    let line = self.buffer.clone();
                                    self.clear_buffer();
                                    return Some(Ok(line));
                                }
                                if let Err(e) = self.push_buffer(right) {
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
                            let line = self.buffer.clone();
                            self.clear_buffer();
                            return Some(Ok(line));
                        } else {
                            if let Err(e) = self.push_buffer(byte) {
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
            let line = self.buffer.clone();
            self.clear_buffer();
            return Some(Ok(line));
        }
        None
    }
}
