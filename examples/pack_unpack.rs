use meatpack::{Packer, Unpacker};

fn main() {
    let gcode = include_str!("../test_files/snippet.gcode");

    println!("## IN ##");
    println!("{gcode}");
    println!("####");

    // Initiliase the packer with buffer size depending
    // on your application
    let packer = Packer::new(false, false);
    let mut meat: Vec<u8> = vec![];

    // Feed in the bytes as you receive them and
    // the packer will return completed lines of
    // meatpacked gcode.
    let written = packer.pack(&mut gcode.as_bytes(), &mut meat).unwrap();
    println!("Gcode: {} Meat: {}", gcode.len(), written);

    println!("## OUT ##");

    // Now we create an unpacker to unpack the meatpacked data.
    let unpacker = Unpacker::default();

    // Imagine receiving the bytes from some I/O and we want
    // to construct gcode lines and deal with them as we form them.
    let mut out: Vec<u8> = Vec::new();
    let written = unpacker.unpack(&mut meat.as_slice(), &mut out).unwrap();

    assert_eq!(written, out.len());
    assert_eq!(written, gcode.len());

    let unpacked_gcode = String::from_utf8(out).expect("Should be valid ASCII");

    println!("{unpacked_gcode}");
    assert_eq!(gcode, unpacked_gcode);

    println!("####");
}
