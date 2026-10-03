use meatpack::{Packer, Unpacker};

fn main() {
    let gcode = include_str!("../test_files/snippet.gcode");
    let packer = Packer::new(false, false);
    let mut packed: Vec<u8> = Vec::new();
    let _written = packer.pack(&mut gcode.as_bytes(), &mut packed).unwrap();
    println!("{gcode}");

    let mut reader: &[u8] = packed.as_slice();
    let unpacker = Unpacker::default();
    let mut unpacked: Vec<u8> = Vec::new();
    let _written = unpacker.unpack(&mut reader, &mut unpacked).unwrap();
    let s = String::from_utf8(unpacked).expect("Should be valid ASCII");
    println!("{s}");

    // Initiliase the packer with buffer size depending
    // on your application
    let mut reader: &[u8] = packed.as_slice();
    let mut unpacker = Unpacker::default();

    let mut line: Vec<u8> = Vec::new();
    loop {
        let written = unpacker.unpack_line(&mut reader, &mut line).unwrap();
        if written == 0 {
            break;
        }
        let s = String::from_utf8(line.clone()).expect("Should be valid ASCII");
        print!("[LINE] {s}");
        line.clear();
    }
}
