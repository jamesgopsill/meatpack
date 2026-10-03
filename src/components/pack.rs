use crate::components::meat::{
    COMMENT_START_BYTE, Error, FULLWIDTH_BYTE, LINEFEED_BYTE, Pack, PackTuple, forward_lookup,
};

use crate::{MEATPACK_HEADER, NO_SPACES_COMMAND};

/// `Packer` manages the packing of `gcode` into meatpacked `gcode`.
pub struct Packer {
    least: Option<u8>,
    fullwidth: Option<u8>,
    strip_whitespace: bool,
    strip_comments: bool,
    comment_flag: bool,
    buf: [u8; 3],
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
            buf: [0u8; 3],
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
            buf: [0u8; 3],
        }
    }

    fn return_slice(
        &mut self,
        s: &[u8],
    ) -> &[u8] {
        self.buf[..s.len()].copy_from_slice(s);
        &self.buf[..s.len()]
    }

    fn return_empty_slice(&self) -> &[u8] {
        static EMPTY: [u8; 0] = [];
        &EMPTY
    }

    /// This is an internal function the works out what to do
    /// when receiving a byte.
    fn pack_byte(
        &mut self,
        byte: u8,
    ) -> Result<&[u8], Error> {
        // Ignore whitespace if we have been instructed to do so.
        if self.strip_whitespace && matches!(byte, b' ' | b'\t') {
            return Ok(self.return_empty_slice());
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
                return Ok(self.return_empty_slice());
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
                self.least = None;
                self.fullwidth = None;
                // OPTION: strip empty lines?
                Ok(self.return_slice(&[packed_byte]))
            }
            // Start of a new byte to pack.
            (None, b) => match b.pack(self.strip_whitespace) {
                // Packable byte
                Some(least) => {
                    self.least = Some(least);
                    self.fullwidth = None;
                    Ok(self.return_empty_slice())
                }
                // Fullwidth byte
                None => {
                    self.least = Some(0b1111);
                    self.fullwidth = Some(b);
                    Ok(self.return_empty_slice())
                }
            },
            // fullwidth + \n
            (Some(0b1111), b'\n') => {
                let most = b'\n'
                    .pack(self.strip_whitespace)
                    .expect(r"Expected \n to return 0b0000_1100");
                let packed_byte = (most, FULLWIDTH_BYTE).pack()?;
                self.least = None;
                let fullwidth = self.fullwidth.take().unwrap();
                Ok(self.return_slice(&[packed_byte, fullwidth]))
            }
            // Full width + some other b byte that is not a \n
            (Some(0b1111), b) => match forward_lookup(b, self.strip_whitespace) {
                // Packable byte
                Some(most) => {
                    let packed_byte = (most, FULLWIDTH_BYTE).pack()?;
                    self.least = None;
                    let fullwidth = self.fullwidth.take().unwrap();
                    Ok(self.return_slice(&[packed_byte, fullwidth]))
                }
                // Fullwidth byte
                None => {
                    // Equivalent to a SIGNAL BYTE but keeping the function for
                    // readability.
                    let packed_byte = (FULLWIDTH_BYTE, FULLWIDTH_BYTE).pack()?;
                    self.least = None;
                    let fullwidth = self.fullwidth.take().unwrap();
                    Ok(self.return_slice(&[packed_byte, fullwidth, b]))
                }
            },
            // Some packable least byte with a \n most.
            (Some(least), b'\n') => {
                let most = byte
                    .pack(self.strip_whitespace)
                    .expect("Should be packable.");
                let packed_byte = (most, least).pack()?;
                self.least = None;
                self.fullwidth = None;
                Ok(self.return_slice(&[packed_byte]))
            }
            // least is packable + whatever b is but not a \n
            (Some(least), b) => match b.pack(self.strip_whitespace) {
                // Packable byte
                Some(most) => {
                    let packed_byte = (most, least).pack()?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(self.return_slice(&[packed_byte]))
                }
                // Fullwidth byte
                None => {
                    let packed_byte = (FULLWIDTH_BYTE, least).pack()?;
                    self.least = None;
                    self.fullwidth = None;
                    Ok(self.return_slice(&[packed_byte, b]))
                }
            },
        }
    }

    /// Packs a `gcode` read stream to meatpacked `gcode` write stream returning
    /// the number of bytes packed into the writer. The writer is not buffered so
    /// it is recommended to use a `BufWriter` or `buffered-io` on `std` and `no_std`,
    /// respectively.
    pub fn pack(
        mut self,
        reader: &mut impl embedded_io::BufRead,
        writer: &mut impl embedded_io::Write,
    ) -> Result<usize, Error> {
        let mut written = 0;

        writer.write_all(MEATPACK_HEADER.as_slice())?;
        written += MEATPACK_HEADER.len();
        if self.strip_whitespace {
            writer.write_all(NO_SPACES_COMMAND.as_slice())?;
            written += NO_SPACES_COMMAND.len();
        }

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }
            for &byte in buf {
                let emitted = self.pack_byte(byte)?;
                writer.write_all(emitted)?;
                written += emitted.len();
            }
            let read = buf.len();
            reader.consume(read);
        }

        writer.flush()?;

        Ok(written)
    }

    /// Packs a `gcode` read stream to meatpacked `gcode` write stream returning
    /// the number of bytes packed into the writer. The writer is not buffered so
    /// it is recommended to use a `BufWriter` or `buffered-io` on `std` and `no_std`,
    /// respectively.
    pub async fn pack_async(
        mut self,
        reader: &mut impl embedded_io_async::BufRead,
        writer: &mut impl embedded_io_async::Write,
    ) -> Result<usize, Error> {
        let mut written = 0;

        writer.write_all(MEATPACK_HEADER.as_slice()).await?;
        written += MEATPACK_HEADER.len();
        if self.strip_whitespace {
            writer.write_all(NO_SPACES_COMMAND.as_slice()).await?;
            written += NO_SPACES_COMMAND.len();
        }

        loop {
            let buf = reader.fill_buf().await?;
            if buf.is_empty() {
                break;
            }
            for &byte in buf {
                let emitted = self.pack_byte(byte)?;
                writer.write_all(emitted).await?;
                written += emitted.len();
            }
            let read = buf.len();
            reader.consume(read);
        }

        writer.flush().await?;

        Ok(written)
    }
}
