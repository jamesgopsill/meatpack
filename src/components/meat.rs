use thiserror::Error;

pub static SIGNAL_BYTE: u8 = 255;
pub static PACKING_ENABLED_BYTE: u8 = 251;
pub static ENABLE_NO_SPACES: u8 = 247;
pub static LINEFEED_BYTE: u8 = b'\n';
pub static COMMENT_START_BYTE: u8 = b';';
pub static FULLWIDTH_BYTE: u8 = 0b0000_1111;
pub static MEATPACK_HEADER: [u8; 3] = [SIGNAL_BYTE, SIGNAL_BYTE, PACKING_ENABLED_BYTE];
pub static NO_SPACES_COMMAND: [u8; 3] = [SIGNAL_BYTE, SIGNAL_BYTE, ENABLE_NO_SPACES];

/// The pack trait that provide the ability
/// to pack an item into a 4-bit meatpack
/// representation.
pub trait Pack {
    /// Will pack into a 4-bit representation or return
    /// an Err stating the item is a full width character.
    fn pack(
        &self,
        no_spaces: bool,
    ) -> Option<u8>;

    // Unpacks the item into two 8-bit values.
    fn unpack(
        &self,
        no_spaces: bool,
    ) -> (u8, u8);
}

/// Implementation of pack for a u8.
impl Pack for u8 {
    fn pack(
        &self,
        no_spaces: bool,
    ) -> Option<u8> {
        forward_lookup(self, no_spaces)
    }

    fn unpack(
        &self,
        no_spaces: bool,
    ) -> (u8, u8) {
        unpack_byte(self, no_spaces)
    }
}

/// Enable packing of items (u4s) into a meatpacked u8.
pub trait PackTuple {
    fn pack(&self) -> Result<u8, MeatPackError>;
}

impl PackTuple for (u8, u8) {
    fn pack(&self) -> Result<u8, MeatPackError> {
        if self.0 > 15u8 {
            return Err(MeatPackError::InvalidByte(self.0));
        }
        if self.1 > 15u8 {
            return Err(MeatPackError::InvalidByte(self.1));
        }
        let packed = self.0 << 4;
        Ok(packed ^ self.1)
    }
}

/// Used in the Packer and Unpacker to inform the
/// user whether a line has been omitted or more
/// bytes are required.
pub enum MeatPackResult<'a> {
    WaitingForNextByte,
    Line(&'a [u8]),
}

/// A set of possible error codes from the MeatPack crate.
#[derive(Debug, Error)]
pub enum MeatPackError {
    #[error("Invalid byte: Recevied: {0}")]
    InvalidByte(u8),
    #[error("The packer is in an invalid state.")]
    InvalidState,
    #[error("Invalid command byte. Received: {0}.")]
    InvalidCommandByte(u8),
    #[error("The buffer is full.")]
    BufferFull,
    #[error("Unterminated line. {0} bytes remain in the buffer.")]
    UnterminatedLine(usize),
    #[error("Empty Buffer")]
    EmptyBuffer,
    #[error(r"Unterminated buffer. Expected the in buffer to terminate with a \n.")]
    UnterminatedBuffer,
}

/// An enum detailing all the available Meatpack commands.
#[derive(Debug)]
pub enum MeatPackCommand {
    PackingEnabled,
    PackingDisabled,
    ResetAll,
    QueryConfig,
    NoSpacesEnabled,
    NoSpacesDisabled,
    SignalByte,
}

/// This function checks whether a `u8` conforms to one of the reserved
/// command bytes.
///
/// | 8-bit Decimal Code | Command |
/// | --- | --- |
/// | 246 | No Spaces Disabled |
/// | 247 | No Spaces Enabled |
/// | 248 | Query Config |
/// | 249 | Reset All |
/// | 250 | Packing Disabled |
/// | 251 | Packing Enabled |
/// | 255 | Signal Byte |
///
/// References
/// - <https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp>
pub const fn determine_command(byte: &u8) -> Result<MeatPackCommand, MeatPackError> {
    match byte {
        246 => Ok(MeatPackCommand::NoSpacesDisabled),
        247 => Ok(MeatPackCommand::NoSpacesEnabled),
        248 => Ok(MeatPackCommand::QueryConfig),
        249 => Ok(MeatPackCommand::ResetAll),
        250 => Ok(MeatPackCommand::PackingDisabled),
        251 => Ok(MeatPackCommand::PackingEnabled),
        255 => Ok(MeatPackCommand::SignalByte),
        b => Err(MeatPackError::InvalidCommandByte(*b)),
    }
}

/// Checks whether a `u8` is a signal byte `255`.
pub const fn is_signal_byte(byte: &u8) -> bool {
    matches!(byte, 255)
}

/// Unpacks the 2 x 4-bit meatpack code packed into a u8.
pub fn unpack_byte(
    byte: &u8,
    no_spaces: bool,
) -> (u8, u8) {
    // Process the 8-bit as two 4-bit values.
    // 4-bits still exist within a u8.
    let mut unpacked: (u8, u8) = (0, 0);
    // Take the 4 most significant bits.
    // e.g. 0111_0010 >> 4 -> 0000_0111
    let most = byte >> 4;
    let u = reverse_lookup(&most, no_spaces).unwrap();
    unpacked.0 = u;
    // Take the 4 least significant bits.
    // e.g., 0111_0010 << 4 -> 0010_0000 -> 0000_0010
    let least = byte << 4 >> 4;
    let u = reverse_lookup(&least, no_spaces).unwrap();
    unpacked.1 = u;
    unpacked
}

/// Provides the lookup table for the 4-bit combinations and their 8-bit counterparts. `0b1011` has different intepretations depending on whether `no_spaces` has been enabled or disabled.
///
/// | 4-bit Decimal Code | 8-bit ASCII char (decimal code) |
/// | --- | --- |
/// | 0b0000_0000 | 0 (48) |
/// | 0b0000_0001 | 1 (49) |
/// | 0b0000_0010 | 2 (50) |
/// | 0b0000_0011 | 3 (51) |
/// | 0b0000_0100 | 4 (52) |
/// | 0b0000_0101 | 5 (53) |
/// | 0b0000_0110 | 6 (54) |
/// | 0b0000_0111 | 7 (55) |
/// | 0b0000_1000 | 8 (56) |
/// | 0b0000_1001 | 9 (57) |
/// | 0b0000_1010 | . (46) |
/// | 0b0000_1011 | Space (32) or E (69) |
/// | 0b0000_1100 | \n (10) |
/// | 0b0000_1101 | G (71) |
/// | 0b0000_1110 | X (88) |
/// | 0b0000_1111 | NUL (0) |
///
/// References
/// - <https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp>
/// - <https://www.asciitable.com/>
pub const fn reverse_lookup(
    byte: &u8,
    no_spaces: bool,
) -> Option<u8> {
    match byte {
        0b0000_0000 => Some(b'0'),
        0b0000_0001 => Some(b'1'),
        0b0000_0010 => Some(b'2'),
        0b0000_0011 => Some(b'3'),
        0b0000_0100 => Some(b'4'),
        0b0000_0101 => Some(b'5'),
        0b0000_0110 => Some(b'6'),
        0b0000_0111 => Some(b'7'),
        0b0000_1000 => Some(b'8'),
        0b0000_1001 => Some(b'9'),
        0b0000_1010 => Some(b'.'),
        0b0000_1011 => {
            if no_spaces {
                Some(b'E')
            } else {
                Some(b' ')
            }
        }
        0b0000_1100 => Some(b'\n'),
        0b0000_1101 => Some(b'G'),
        0b0000_1110 => Some(b'X'),
        0b0000_1111 => Some(0),
        _ => None,
    }
}

/// The forward lookup variant of the reverse lookup byte.
pub const fn forward_lookup(
    byte: &u8,
    no_spaces: bool,
) -> Option<u8> {
    match byte {
        b'0' => Some(0b0000_0000),
        b'1' => Some(0b0000_0001),
        b'2' => Some(0b0000_0010),
        b'3' => Some(0b0000_0011),
        b'4' => Some(0b0000_0100),
        b'5' => Some(0b0000_0101),
        b'6' => Some(0b0000_0110),
        b'7' => Some(0b0000_0111),
        b'8' => Some(0b0000_1000),
        b'9' => Some(0b0000_1001),
        b'.' => Some(0b0000_1010),
        b'E' => {
            if no_spaces {
                Some(0b0000_1011)
            } else {
                None
            }
        }
        b' ' => {
            if no_spaces {
                None
            } else {
                Some(0b0000_1011)
            }
        }
        b'\n' => Some(0b0000_1100),
        b'G' => Some(0b0000_1101),
        b'X' => Some(0b0000_1110),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unpack() {
        let packed: u8 = 0b1101_0001;
        let (most, least) = packed.unpack(false);
        assert_eq!(most, b'G');
        assert_eq!(least, b'1');
    }

    #[test]
    fn test_pack_packable() {
        let packable: u8 = b'0';
        let packed = packable.pack(false).unwrap();
        assert_eq!(packed, 0u8);
    }

    #[test]
    fn test_pack_unpackable() {
        let unpackable: u8 = b'T';
        let packed = unpackable.pack(false);
        assert!(packed.is_none());
    }
}
