//! Core functions for MeatPack.

#[cfg(feature = "no_std")]
use no_std_io::io::Error;
#[cfg(feature = "std")]
use std::io::Error;

/// A set of possible error codes from the MeatPack crate.
#[derive(Debug)]
pub enum MeatPackError {
    InvalidByte,
    InvalidCommandByte,
    BufferFull,
    IOError(Error),
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
pub fn determine_command(byte: &u8) -> Result<MeatPackCommand, MeatPackError> {
    match byte {
        246 => return Ok(MeatPackCommand::NoSpacesDisabled),
        247 => return Ok(MeatPackCommand::NoSpacesEnabled),
        248 => return Ok(MeatPackCommand::QueryConfig),
        249 => return Ok(MeatPackCommand::ResetAll),
        250 => return Ok(MeatPackCommand::PackingDisabled),
        251 => return Ok(MeatPackCommand::PackingEnabled),
        255 => return Ok(MeatPackCommand::SignalByte),
        _ => return Err(MeatPackError::InvalidCommandByte),
    }
}

/// Checks whether a `u8` is a signal byte `255`.
pub fn is_signal_byte(byte: &u8) -> bool {
    match byte {
        255 => return true,
        _ => return false,
    }
}

/// Checks whether a `u8` is a linefeed byte `10`.
pub fn is_linefeed_byte(byte: &u8) -> bool {
    match byte {
        10 => return true,
        _ => return false,
    }
}

/// Unpacks the 2 x 4-bit meatpack code packed into a u8.
pub fn unpack_byte(byte: &u8, no_spaces: bool) -> Result<(u8, u8), MeatPackError> {
    // Process the 8-bit as two 4-bit values.
    // 4-bits still exist within a u8.
    let mut unpacked: (u8, u8) = (0, 0);
    let upper = byte >> 4;
    let u = lookup_byte(&upper, no_spaces)?;
    unpacked.0 = u;
    let lower = byte << 4 >> 4;
    let u = lookup_byte(&lower, no_spaces)?;
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
pub fn lookup_byte(byte: &u8, no_spaces: bool) -> Result<u8, MeatPackError> {
    match byte {
        0b0000 => return Ok(48),
        0b0001 => return Ok(49),
        0b0010 => return Ok(50),
        0b0011 => return Ok(51),
        0b0100 => return Ok(52),
        0b0101 => return Ok(53),
        0b0110 => return Ok(54),
        0b0111 => return Ok(55),
        0b1000 => return Ok(56),
        0b1001 => return Ok(57),
        0b1010 => return Ok(46),
        0b1011 => {
            if no_spaces {
                return Ok(69);
            } else {
                return Ok(32);
            }
        }
        0b1100 => return Ok(10),
        0b1101 => return Ok(71),
        0b1110 => return Ok(88),
        0b1111 => return Ok(0),
        _ => return Err(MeatPackError::InvalidByte),
    }
}

#[cfg(feature = "std")]
/// Utility function that accepts a `u8` and prints the utf8 [0-255] chars which includes the ASCII table.
pub fn print_ascii(bytes: &[u8]) {
    for b in bytes {
        if *b != 0 {
            let c = char::try_from(*b).unwrap();
            print!("{}", c);
        }
    }
    println!();
}
