use crate::components::meat::{
    COMMENT_START_BYTE, FULLWIDTH_BYTE, LINEFEED_BYTE, MeatPackError, Pack, PackTuple,
    forward_lookup,
};

use crate::{MEATPACK_HEADER, NO_SPACES_COMMAND};

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
}
