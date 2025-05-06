use core::str::from_utf8;
use std::{string::String, vec::Vec};

use crate::{MEATPACK_HEADER, MeatPackResult, Packer, Unpacker};

#[test]
fn test_pack_unpack_strip_comments_false() {
    let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";
    let mut packer = Packer::<64>::new(false, false);
    let mut out: Vec<u8> = Vec::new();

    // This will store the entire packed version of the gcode.
    // Don't forget the header.
    out.extend(&MEATPACK_HEADER);

    // Feed in the bytes as you receive them and
    // the packer will return completed lines of
    // meatpacked gcode.
    for byte in gcode.as_bytes() {
        let packed = packer.pack(byte);
        match packed {
            Ok(MeatPackResult::Line(line)) => {
                out.extend(line);
            }
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Err(_) => panic!("Should not enter here"),
        }
    }

    // Now we create an unpacker to unpack the meatpacked data.
    let mut unpacker = Unpacker::<64>::default();

    let mut unpacked = String::new();

    // Imagine receiving the bytes from some I/O and we want
    // to construct gcode lines and deal with them as we form them.
    for byte in out.iter() {
        let res = unpacker.unpack(byte);
        match res {
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Ok(MeatPackResult::Line(line)) => {
                let s = from_utf8(line).unwrap();
                unpacked.push_str(s);
            }
            Err(_) => panic!("Should not enter here"),
        }
    }

    assert_eq!(gcode, unpacked.as_str())
}

#[test]
fn test_pack_unpack_strip_comments_true() {
    let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";
    let mut packer = Packer::<64>::default();
    let mut out: Vec<u8> = Vec::new();

    // This will store the entire packed version of the gcode.
    // Don't forget the header.
    out.extend(&MEATPACK_HEADER);

    // Feed in the bytes as you receive them and
    // the packer will return completed lines of
    // meatpacked gcode.
    for byte in gcode.as_bytes() {
        let packed = packer.pack(byte);
        match packed {
            Ok(MeatPackResult::Line(line)) => {
                out.extend(line);
            }
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Err(_) => panic!("Should not enter here"),
        }
    }

    // Now we create an unpacker to unpack the meatpacked data.
    let mut unpacker = Unpacker::<64>::default();

    let mut unpacked = String::new();

    // Imagine receiving the bytes from some I/O and we want
    // to construct gcode lines and deal with them as we form them.
    for byte in out.iter() {
        let res = unpacker.unpack(byte);
        match res {
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Ok(MeatPackResult::Line(line)) => {
                let s = from_utf8(line).unwrap();
                unpacked.push_str(s);
            }
            Err(_) => panic!("Should not enter here"),
        }
    }

    // Note. \x20 is used for the trailing space that would
    // remain after removing the comment. \x20 is needed to
    // avoid cargo fmt removing it when saving the file.
    let expected = "M73 P0 R3
M73 Q0 S3\x20
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

    assert_eq!(expected, unpacked.as_str())
}

#[cfg(feature = "alloc")]
#[test]
fn test_alloc_strip_comments_false() {
    use std::{env, fs};

    let mut path = env::current_dir().unwrap();
    path.push("test_files");
    path.push("box.gcode");
    let gcode = fs::read(path).unwrap();

    let mut meat: Vec<u8> = Vec::new();
    Packer::<128>::pack_slice(&gcode, &mut meat, false, false).unwrap();
    let mut unpacked: Vec<u8> = Vec::new();
    Unpacker::<128>::unpack_slice(&meat, &mut unpacked).unwrap();
    assert_eq!(gcode, unpacked)
}

#[cfg(feature = "alloc")]
#[test]
fn test_alloc_strip_comments_true_strip_whitespace_false() {
    let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

    let expected = "M73 P0 R3
M73 Q0 S3\x20
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

    let mut meat: Vec<u8> = Vec::new();
    Packer::<128>::pack_slice(gcode.as_bytes(), &mut meat, true, false).unwrap();
    let mut unpacked: Vec<u8> = Vec::new();
    Unpacker::<128>::unpack_slice(&meat, &mut unpacked).unwrap();
    let unpacked = String::from_utf8(unpacked).unwrap();
    assert_eq!(expected, unpacked)
}

#[cfg(feature = "alloc")]
#[test]
fn test_alloc_strip_comments_true_strip_whitespace_true() {
    let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

    let expected = "M73P0R3
M73Q0S3
M201X4000Y4000Z200E2500
M203X300Y300Z40E100
M204P4000R1200T4000
";

    let mut meat: Vec<u8> = Vec::new();
    Packer::<128>::pack_slice(gcode.as_bytes(), &mut meat, true, true).unwrap();
    let mut unpacked: Vec<u8> = Vec::new();
    Unpacker::<128>::unpack_slice(&meat, &mut unpacked).unwrap();
    let unpacked = String::from_utf8(unpacked).unwrap();
    assert_eq!(expected, unpacked)
}
