use meatpack::Packer;

fn main() {
    // Some example gcode.
    let gcode = include_bytes!("../test_files/snippet.gcode");
    let mut packer = Packer::default();
    let mut out: Vec<u8> = Vec::new();
    let written = packer.pack(&mut gcode.as_slice(), &mut out).unwrap();
    println!("Gcode: {} Meat: {}", gcode.len(), written);
}
