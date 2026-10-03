use crate::components::{
    emit::Emit,
    meat::{Command, Error, Pack, determine_command, is_signal_byte},
};

/// A list of state the Unpacker struct can exist in.
#[derive(Debug)]
enum UnpackerState {
    FirstCommandByte,
    SecondCommandByte,
    RightFullWidthByte,
    LeftFullWidthByte,
    Enabled,
    Disabled,
}

/// A  struct for that unpacks bytes and emits
/// lines of gcode.
pub struct Unpacker {
    state: UnpackerState,
    strip_whitespace: bool,
    held_back: u8,
}

impl Default for Unpacker {
    /// The default implementation of the unpacker.
    fn default() -> Self {
        Self {
            state: UnpackerState::Disabled,
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
    ) -> Result<Emit, Error> {
        // First check if it is a signal byte
        // and handle the scenarios.
        if is_signal_byte(byte) {
            match self.state {
                UnpackerState::FirstCommandByte => {
                    self.state = UnpackerState::SecondCommandByte;
                    return Ok(Emit::NONE);
                }
                UnpackerState::Disabled => {
                    self.state = UnpackerState::FirstCommandByte;
                    return Ok(Emit::NONE);
                }
                UnpackerState::Enabled => {
                    self.state = UnpackerState::FirstCommandByte;
                    return Ok(Emit::NONE);
                }
                _ => {
                    return Err(Error::InvalidState);
                }
            }
        }

        // Handle non signal scenarios.
        match self.state {
            UnpackerState::Disabled => {
                // If a normal new line.
                // Return the line for further processing.
                Ok(Emit::one(byte))
            }
            UnpackerState::Enabled => {
                let (most, least) = byte.unpack(self.strip_whitespace);

                // most, least
                // Check if we need to wait for a
                // fullwidth byte.
                match (most, least) {
                    // \n\n packed byte. Just return one \n
                    (10, 10) => Ok(Emit::one(10)),
                    // most is a full width byte
                    (0, 1..) => {
                        self.state = UnpackerState::RightFullWidthByte;
                        Ok(Emit::one(least))
                    }
                    // least is a full width byte
                    (1.., 0) => {
                        // Note. need to wait for the next byte to then insert
                        // them in the right order.
                        self.state = UnpackerState::LeftFullWidthByte;
                        self.held_back = most;
                        Ok(Emit::NONE)
                    }
                    // Should be dealt with by the command bytes section.
                    (0, 0) => {
                        unreachable!();
                    }
                    // Two unpacked packable bytes.
                    (most, least) => Ok(Emit::two(least, most)),
                }
            }
            UnpackerState::SecondCommandByte => {
                let cmd = determine_command(byte)?;
                self.handle_command(cmd);
                Ok(Emit::NONE)
            }
            UnpackerState::FirstCommandByte => {
                self.state = UnpackerState::RightFullWidthByte;
                Ok(Emit::one(byte))
            }
            UnpackerState::RightFullWidthByte => {
                self.state = UnpackerState::Enabled;
                Ok(Emit::one(byte))
            }
            UnpackerState::LeftFullWidthByte => {
                self.state = UnpackerState::Enabled;
                Ok(Emit::two(byte, self.held_back))
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
    ) -> Result<usize, Error> {
        let mut written: usize = 0;
        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            for (i, byte) in buf.iter().copied().enumerate() {
                let emitted = self.unpack_byte(byte)?;
                let emitted = emitted.as_slice();
                writer.write_all(emitted)?;
                // Did we emit the end of a line?
                if emitted.last().is_some_and(|&b| b == b'\n') {
                    written += emitted.len();
                    let read = i + 1;
                    reader.consume(read);
                    writer.flush()?;
                    return Ok(written);
                }
            }
            let read = buf.len();
            reader.consume(read);
        }
        writer.flush()?;
        Ok(written)
    }

    /// Unpacks a line of `gcode` from the meatpack `reader`. Useful in
    /// `no_std` where space is limited and you want to operate on each
    /// line. Note that `writer` is not buffered so you need to decide whether
    /// you want to provide a buffered writer to the function.
    pub async fn unpack_line_async(
        &mut self,
        reader: &mut impl embedded_io_async::BufRead,
        writer: &mut impl embedded_io_async::Write,
    ) -> Result<usize, Error> {
        let mut written: usize = 0;
        loop {
            let buf = reader.fill_buf().await?;
            if buf.is_empty() {
                break;
            }
            for (i, byte) in buf.iter().copied().enumerate() {
                let emitted = self.unpack_byte(byte)?;
                let emitted = emitted.as_slice();
                writer.write_all(emitted).await?;
                // Did we emit the end of a line?
                if emitted.last().is_some_and(|&b| b == b'\n') {
                    written += emitted.len();
                    let read = i + 1;
                    reader.consume(read);
                    writer.flush().await?;
                    return Ok(written);
                }
            }
            let read = buf.len();
            reader.consume(read);
        }
        writer.flush().await?;
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
    ) -> Result<usize, Error> {
        let mut written: usize = 0;
        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            for byte in buf.iter().copied() {
                let emitted = self.unpack_byte(byte)?;
                let emitted = emitted.as_slice();
                writer.write_all(emitted)?;
                written += emitted.len()
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
    pub async fn unpack_async(
        &mut self,
        reader: &mut impl embedded_io_async::BufRead,
        writer: &mut impl embedded_io_async::Write,
    ) -> Result<usize, Error> {
        let mut written: usize = 0;
        loop {
            let buf = reader.fill_buf().await?;
            if buf.is_empty() {
                break;
            }
            for byte in buf.iter().copied() {
                let emitted = self.unpack_byte(byte)?;
                let emitted = emitted.as_slice();
                writer.write_all(emitted).await?;
                written += emitted.len()
            }
            let read = buf.len();
            reader.consume(read);
        }
        writer.flush().await?;
        Ok(written)
    }

    /// Handles the command byte combinations that
    /// exist in the meatpack spec.
    fn handle_command(
        &mut self,
        cmd: Command,
    ) {
        match cmd {
            Command::PackingEnabled => {
                self.state = UnpackerState::Enabled;
            }
            Command::PackingDisabled => {
                self.state = UnpackerState::Disabled;
            }
            Command::ResetAll => {
                self.state = UnpackerState::Disabled;
                self.strip_whitespace = false;
            }
            Command::QueryConfig => {}
            Command::NoSpacesEnabled => {
                self.strip_whitespace = true;
                self.state = UnpackerState::Enabled;
            }
            Command::NoSpacesDisabled => {
                self.strip_whitespace = false;
                self.state = UnpackerState::Enabled;
            }
            Command::SignalByte => {}
        }
    }
}
