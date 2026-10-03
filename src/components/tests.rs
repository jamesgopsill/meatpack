use std::{string::String, vec::Vec};

use crate::{Packer, Unpacker};

const SNIPPET: &str = include_str!("../../test_files/snippet.gcode");

#[test]
fn test_pack_unpack_strip_comments_false() {
    let packer = Packer::new(false, false);
    let mut packed: Vec<u8> = Vec::new();

    let _written = packer.pack(&mut SNIPPET.as_bytes(), &mut packed).unwrap();

    let unpacker = Unpacker::default();
    let mut unpacked: Vec<u8> = Vec::new();

    let _written = unpacker
        .unpack(&mut packed.as_slice(), &mut unpacked)
        .unwrap();

    let unpacked = String::from_utf8(unpacked).expect("Should be valid ASCII");

    assert_eq!(SNIPPET, unpacked)
}

#[test]
fn test_pack_unpack_strip_comments_true() {
    let packer = Packer::new(true, false);
    let mut packed: Vec<u8> = Vec::new();

    let _written = packer.pack(&mut SNIPPET.as_bytes(), &mut packed).unwrap();

    let unpacker = Unpacker::default();
    let mut unpacked: Vec<u8> = Vec::new();

    let _written = unpacker
        .unpack(&mut packed.as_slice(), &mut unpacked)
        .unwrap();

    let unpacked = String::from_utf8(unpacked).expect("Should be valid ASCII");

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
