use crate::components::meat::{
    COMMENT_START_BYTE, FULLWIDTH_BYTE, LINEFEED_BYTE, MeatPackError, Pack, PackTuple,
    forward_lookup,
};

use crate::{MEATPACK_HEADER, NO_SPACES_COMMAND};

/*
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A  struct for that packs bytes and emits
/// lines of meatpacked gcode. Stripping comments
/// is on by default and empty lines are omitted.
pub struct Packer<const S: usize> {
    least: Option<u8>,
    fullwidth: Option<u8>,
    clear: bool,
    strip_whitespace: bool,
    strip_comments: bool,
    comment_flag: bool,
    pos: usize,
    inner: [u8; S],
}

impl<const S: usize> Default for Packer<S> {
    /// The default implementation of a Packer.
    fn default() -> Self {
        Self {
            least: None,
            fullwidth: None,
            clear: false,
            strip_whitespace: false,
            strip_comments: true,
            comment_flag: false,
            pos: 0,
            inner: [0u8; S],
        }
    }
}

impl<const S: usize> Packer<S> {
    /// Create a new instance of the packer
    pub fn new(
        strip_comments: bool,
        strip_whitespace: bool,
    ) -> Self {
        Self {
            least: None,
            fullwidth: None,
            clear: false,
            strip_whitespace,
            strip_comments,
            comment_flag: false,
            pos: 0,
            inner: [0u8; S],
        }
    }

    /// Pack a byte into the current line.
    pub fn pack<'a>(
        &'a mut self,
        b: u8,
    ) -> Result<MeatPackResult<'a>, MeatPackError> {
        // Cleat the buffer if we have been instructed to do so.
        if self.clear {
            self.clear()
        }
        // Ignore whitespace if we have been instructed to do so.
        if self.strip_whitespace && matches!(b, b' ' | b'\t') {
            return Ok(MeatPackResult::WaitingForNextByte);
        }
        // Check if strip comments is active and ignore
        if self.strip_comments {
            if b == COMMENT_START_BYTE {
                self.comment_flag = true;
            }
            if b == LINEFEED_BYTE {
                self.comment_flag = false;
            }
            if self.comment_flag {
                return Ok(MeatPackResult::WaitingForNextByte);
            }
        }

        // Match on the possible two bytes we have. One that is intended for the least and most significant ends of a u8.
        match (self.least, b) {
            // Special case requiring \n\n.
            (None, b'\n') => {
                let most = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expect \n to return 0b0000_1100");
                let least = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expect \n to return 0b0000_1100");
                let packed_byte = (most, least)
                    .pack()
                    .expect("Should pack as we have provided to two known packables.");
                self.push(packed_byte)?;
                self.clear = true;
                // Remove empty lines.
                if self.pos > 1 {
                    Ok(MeatPackResult::Line(self.return_slice()))
                } else {
                    Ok(MeatPackResult::WaitingForNextByte)
                }
            }
            // Start of a new byte to pack.
            (None, b) => match b.pack(self.strip_whitespace) {
                // Packable byte
                Some(least) => {
                    self.least = Some(least);
                    self.fullwidth = None;
                    Ok(MeatPackResult::WaitingForNextByte)
                }
                // Fullwidth byte
                None => {
                    self.least = Some(0b1111);
                    self.fullwidth = Some(b);
                    Ok(MeatPackResult::WaitingForNextByte)
                }
            },
            // fullwidth + \n
            (Some(0b1111), b'\n') => {
                let most = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expected \n to return 0b0000_1100");
                let packed_byte = (most, FULLWIDTH_BYTE)
                    .pack()
                    .expect("Should pack as we have provided to two packed chars.");
                self.push(packed_byte)?;
                self.push(self.fullwidth.unwrap())?;
                self.least = None;
                self.fullwidth = None;
                self.clear = true;
                Ok(MeatPackResult::Line(self.return_slice()))
            }
            // Full width + some other b byte that is not a \n
            (Some(0b1111), b) => match forward_lookup(b, self.strip_whitespace) {
                // Packable byte
                Some(most) => {
                    let packed_byte = (most, FULLWIDTH_BYTE)
                        .pack()
                        .expect("Should pack as we have provided to two packed chars.");
                    self.push(packed_byte)?;
                    self.push(self.fullwidth.unwrap())?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(MeatPackResult::WaitingForNextByte)
                }
                // Fullwidth byte
                None => {
                    // Equivalent to a SIGNAL BYTE but keeping the function for
                    // readability.
                    let packed_byte = (FULLWIDTH_BYTE, FULLWIDTH_BYTE)
                        .pack()
                        .expect("Should pack as we have provided to two packed chars.");
                    self.push(packed_byte)?;
                    self.push(self.fullwidth.unwrap())?;
                    self.push(b)?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(MeatPackResult::WaitingForNextByte)
                }
            },
            // Some packable least byte with a \n most.
            (Some(least), b'\n') => {
                let most = b.pack(self.strip_whitespace).expect("Should be packable.");
                let packed_bytes = (most, least).pack().expect("Should be packable.");
                self.push(packed_bytes)?;
                self.least = None;
                self.fullwidth = None;
                self.clear = true;
                Ok(MeatPackResult::Line(self.return_slice()))
            }
            // least is packable + whatever b is but not a \n
            (Some(least), b) => match b.pack(self.strip_whitespace) {
                // Packable byte
                Some(most) => {
                    let packed_byte = (most, least).pack().expect("Should be packable.");
                    self.push(packed_byte)?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(MeatPackResult::WaitingForNextByte)
                }
                // Fullwidth byte
                None => {
                    let packed_byte = (FULLWIDTH_BYTE, least).pack().expect("Should be packable.");
                    self.push(packed_byte)?;
                    self.push(b)?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(MeatPackResult::WaitingForNextByte)
                }
            },
        }
    }

    /// Returns a slice of the filled elements in the inner.
    fn return_slice(&mut self) -> &[u8] {
        &self.inner[0..self.pos]
    }

    /// Clears the inner
    fn clear(&mut self) {
        self.inner.fill(0);
        self.pos = 0;
        self.clear = false;
    }

    /// Adds a byte to the inner.
    fn push(
        &mut self,
        byte: u8,
    ) -> Result<(), MeatPackError> {
        if self.pos > S {
            return Err(MeatPackError::BufferFull);
        }
        self.inner[self.pos] = byte;
        self.pos += 1;
        Ok(())
    }

    /// A utility function to check if any data remains
    /// in the internal inner. We expect all meatpack
    /// lines to newline end.
    pub fn data_remains(&self) -> bool {
        if !self.clear || self.pos == 0 {
            return true;
        }
        false
    }

    /// A convenience function for those with alloc available to them.
    /// It wraps around packer and packs a slice of bytes into a vec.
    #[cfg(feature = "alloc")]
    pub fn pack_slice(
        in_buf: &[u8],
        out_buf: &mut Vec<u8>,
        strip_comments: bool,
        strip_whitespace: bool,
    ) -> Result<(), MeatPackError> {
        use super::meat::NO_SPACES_COMMAND;

        if in_buf.is_empty() {
            return Err(MeatPackError::EmptyBuffer);
        }

        if in_buf.last().unwrap() != &b'\n' {
            return Err(MeatPackError::UnterminatedBuffer);
        }

        out_buf.extend(MEATPACK_HEADER.as_slice());
        if strip_whitespace {
            out_buf.extend(NO_SPACES_COMMAND.as_slice());
        }

        let mut packer = Packer::<S>::new(strip_comments, strip_whitespace);

        for &b in in_buf.iter() {
            match packer.pack(b) {
                Ok(MeatPackResult::Line(line)) => out_buf.extend(line),
                Ok(MeatPackResult::WaitingForNextByte) => {}
                Err(e) => return Err(e),
            }
        }
        // if the packer is in the state of clearing itself
        // on the next iteration then ignore as we hit a new line.
        // Otherwise we have an unterminated line with some
        // data possibly stuck in the inner and we're expecting
        // to end with a terminated line so throw an err.
        if packer.data_remains() {
            return Err(MeatPackError::UnterminatedLine(packer.pos));
        }

        Ok(())
    }
}
*/

#[cfg(feature = "std")]
extern crate std;

pub enum PackerState {
    NewLine(usize),
    Packed(usize),
    Pending,
}

pub struct Packer {
    least: Option<u8>,
    fullwidth: Option<u8>,
    strip_whitespace: bool,
    strip_comments: bool,
    comment_flag: bool,
}

impl Default for Packer {
    /// The default implementation of a Packer.
    fn default() -> Self {
        Self {
            least: None,
            fullwidth: None,
            strip_whitespace: false,
            strip_comments: true,
            comment_flag: false,
        }
    }
}

impl Packer {
    /// Create a new instance of the packer
    pub fn new(
        strip_comments: bool,
        strip_whitespace: bool,
    ) -> Self {
        Self {
            least: None,
            fullwidth: None,
            strip_whitespace,
            strip_comments,
            comment_flag: false,
        }
    }

    fn pack_byte(
        &mut self,
        byte: u8,
        writer: &mut impl embedded_io::Write,
    ) -> Result<PackerState, MeatPackError> {
        // Ignore whitespace if we have been instructed to do so.
        if self.strip_whitespace && matches!(byte, b' ' | b'\t') {
            return Ok(PackerState::Pending);
        }
        // Check if strip comments is active and ignore
        if self.strip_comments {
            if byte == COMMENT_START_BYTE {
                self.comment_flag = true;
            }
            if byte == LINEFEED_BYTE {
                self.comment_flag = false;
            }
            if self.comment_flag {
                return Ok(PackerState::Pending);
            }
        }

        // Match on the possible two bytes we have.
        // One that is intended for the least and most significant ends of a u8.
        match (self.least, byte) {
            // Special case requiring \n\n.
            (None, b'\n') => {
                let most = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expect \n to return 0b0000_1100");
                let least = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expect \n to return 0b0000_1100");
                let packed_byte = (most, least).pack()?;
                writer.write(&[packed_byte])?;
                self.least = None;
                self.fullwidth = None;
                // Should we remove empty lines?
                Ok(PackerState::NewLine(1))
            }
            // Start of a new byte to pack.
            (None, b) => match b.pack(self.strip_whitespace) {
                // Packable byte
                Some(least) => {
                    self.least = Some(least);
                    self.fullwidth = None;
                    Ok(PackerState::Pending)
                }
                // Fullwidth byte
                None => {
                    self.least = Some(0b1111);
                    self.fullwidth = Some(b);
                    Ok(PackerState::Pending)
                }
            },
            // fullwidth + \n
            (Some(0b1111), b'\n') => {
                let most = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expected \n to return 0b0000_1100");
                let packed_byte = (most, FULLWIDTH_BYTE).pack()?;
                writer.write(&[packed_byte, self.fullwidth.unwrap()])?;
                self.least = None;
                self.fullwidth = None;
                Ok(PackerState::NewLine(2))
            }
            // Full width + some other b byte that is not a \n
            (Some(0b1111), b) => match forward_lookup(b, self.strip_whitespace) {
                // Packable byte
                Some(most) => {
                    let packed_byte = (most, FULLWIDTH_BYTE)
                        .pack()
                        .expect("Should pack as we have provided to two packed chars.");
                    writer.write(&[packed_byte, self.fullwidth.unwrap()])?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(PackerState::Packed(2))
                }
                // Fullwidth byte
                None => {
                    // Equivalent to a SIGNAL BYTE but keeping the function for
                    // readability.
                    let packed_byte = (FULLWIDTH_BYTE, FULLWIDTH_BYTE).pack()?;
                    writer.write(&[packed_byte, self.fullwidth.unwrap(), b])?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(PackerState::Packed(3))
                }
            },
            // Some packable least byte with a \n most.
            (Some(least), b'\n') => {
                let most = byte
                    .pack(self.strip_whitespace)
                    .expect("Should be packable.");
                let packed_byte = (most, least).pack()?;
                writer.write(&[packed_byte])?;
                self.least = None;
                self.fullwidth = None;
                Ok(PackerState::NewLine(1))
            }
            // least is packable + whatever b is but not a \n
            (Some(least), b) => match b.pack(self.strip_whitespace) {
                // Packable byte
                Some(most) => {
                    let packed_byte = (most, least).pack()?;
                    writer.write(&[packed_byte])?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(PackerState::Packed(1))
                }
                // Fullwidth byte
                None => {
                    let packed_byte = (FULLWIDTH_BYTE, least).pack()?;
                    writer.write(&[packed_byte, b])?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(PackerState::Packed(2))
                }
            },
        }
    }

    pub fn pack(
        &mut self,
        reader: &mut impl embedded_io::BufRead,
        writer: &mut impl embedded_io::Write,
    ) -> Result<usize, MeatPackError> {
        let mut written = 0;

        writer.write(MEATPACK_HEADER.as_slice())?;
        written += MEATPACK_HEADER.len();
        if self.strip_whitespace {
            writer.write(NO_SPACES_COMMAND.as_slice())?;
            written += NO_SPACES_COMMAND.len();
        }

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            for byte in buf.iter().copied() {
                match self.pack_byte(byte, writer)? {
                    PackerState::NewLine(size) => written += size,
                    PackerState::Packed(size) => written += size,
                    PackerState::Pending => {}
                }
            }
            let read = buf.len();
            reader.consume(read);
        }

        writer.flush()?;

        Ok(written)
    }

    #[cfg(feature = "std")]
    pub fn pack_std(
        &mut self,
        reader: &mut impl std::io::BufRead,
        writer: &mut impl std::io::Write,
    ) -> Result<usize, MeatPackError> {
        use embedded_io_adapters::std::FromStd;

        let mut reader = FromStd::new(reader);
        let mut writer = FromStd::new(writer);
        self.pack(&mut reader, &mut writer)
    }
}
