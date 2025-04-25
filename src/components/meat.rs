pub static SIGNAL_BYTE: u8 = 255;
pub static PACKING_ENABLED_BYTE: u8 = 251;
pub static LINEFEED_BYTE: u8 = 10;
pub static COMMENT_START_BYTE: u8 = 59;
pub static MEATPACK_HEADER: [u8; 3] = [SIGNAL_BYTE, SIGNAL_BYTE, PACKING_ENABLED_BYTE];

pub enum MeatPackResult<'a> {
    WaitingForNextByte,
    Line(&'a [u8]),
}

/// A set of possible error codes from the MeatPack crate.
#[derive(Debug)]
pub enum MeatPackError {
    InvalidByte,
    InvalidState,
    InvalidCommandByte,
    BufferFull,
    FullWidthByte,
    UnterminatedLine(usize),
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
        _ => Err(MeatPackError::InvalidCommandByte),
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
) -> Result<(u8, u8), MeatPackError> {
    // Process the 8-bit as two 4-bit values.
    // 4-bits still exist within a u8.
    let mut unpacked: (u8, u8) = (0, 0);
    // e.g. 0111_0010 >> 4 -> 0000_0111
    let upper = byte >> 4;
    let u = reverse_lookup(&upper, no_spaces)?;
    unpacked.0 = u;
    // e.g., 0111_0010 << 4 -> 0010_0000 -> 0000_0010
    let lower = byte << 4 >> 4;
    let u = reverse_lookup(&lower, no_spaces)?;
    unpacked.1 = u;
    Ok(unpacked)
}

/// Provides the lookup table for the 4-bit combinations and their 8-bit counterparts. `0b1011` has different intepretations depending on whether `no_spaces` has been enabled or disabled.
///
/// | 4-bit Decimal Code | 8-bit ASCII char (decimal code) |
/// | --- | --- |
/// | 0b0000 | 0 (48) |
/// | 0b0001 | 1 (49) |
/// | 0b0010 | 2 (50) |
/// | 0b0011 | 3 (51) |
/// | 0b0100 | 4 (52) |
/// | 0b0101 | 5 (53) |
/// | 0b0110 | 6 (54) |
/// | 0b0111 | 7 (55) |
/// | 0b1000 | 8 (56) |
/// | 0b1001 | 9 (57) |
/// | 0b1010 | . (46) |
/// | 0b1011 | Space (32) or E (69) |
/// | 0b1100 | \n (10) |
/// | 0b1101 | G (71) |
/// | 0b1110 | X (88) |
/// | 0b1111 | NUL (0) |
///
/// References
/// - <https://github.com/prusa3d/libbgcode/blob/main/src/LibBGCode/binarize/meatpack.cpp>
/// - <https://www.asciitable.com/>
pub const fn reverse_lookup(
    byte: &u8,
    no_spaces: bool,
) -> Result<u8, MeatPackError> {
    match byte {
        0b0000 => Ok(b'0'),
        0b0001 => Ok(b'1'),
        0b0010 => Ok(b'2'),
        0b0011 => Ok(b'3'),
        0b0100 => Ok(b'4'),
        0b0101 => Ok(b'5'),
        0b0110 => Ok(b'6'),
        0b0111 => Ok(b'7'),
        0b1000 => Ok(b'8'),
        0b1001 => Ok(b'9'),
        0b1010 => Ok(b'.'),
        0b1011 => {
            if no_spaces {
                Ok(b'E')
            } else {
                Ok(b' ')
            }
        }
        0b1100 => Ok(b'\n'),
        0b1101 => Ok(b'G'),
        0b1110 => Ok(b'X'),
        0b1111 => Ok(0),
        _ => Err(MeatPackError::InvalidByte),
    }
}

/// The forward lookup variant of the reverse lookup byte.
pub const fn forward_lookup(
    byte: &u8,
    no_spaces: bool,
) -> Result<u8, MeatPackError> {
    match byte {
        b'0' => Ok(0b0000),
        b'1' => Ok(0b0001),
        b'2' => Ok(0b0010),
        b'3' => Ok(0b0011),
        b'4' => Ok(0b0100),
        b'5' => Ok(0b0101),
        b'6' => Ok(0b0110),
        b'7' => Ok(0b0111),
        b'8' => Ok(0b1000),
        b'9' => Ok(0b1001),
        b'.' => Ok(0b1010),
        b'E' => {
            if no_spaces {
                Ok(0b1011)
            } else {
                Err(MeatPackError::FullWidthByte)
            }
        }
        b' ' => {
            if no_spaces {
                Err(MeatPackError::FullWidthByte)
            } else {
                Ok(0b1011)
            }
        }
        b'\n' => Ok(0b1100),
        b'G' => Ok(0b1101),
        b'X' => Ok(0b1110),
        _ => Err(MeatPackError::FullWidthByte),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unpack() {
        let packed: u8 = 0b1101_0001;
        let (upper, lower) = unpack_byte(&packed, false).unwrap();
        assert_eq!(upper, b'G');
        assert_eq!(lower, b'1');
    }

    #[test]
    fn test_pack_packable() {
        let packable: u8 = b'0';
        let packed = forward_lookup(&packable, false).unwrap();
        assert_eq!(packed, 0u8);
    }

    #[test]
    fn test_pack_unpackable() {
        let packable: u8 = b'T';
        let packed = forward_lookup(&packable, false);
        assert!(packed.is_err());
    }
}
