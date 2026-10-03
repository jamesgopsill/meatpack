use meatpack::Unpacker;

fn main() {
    // Some meatpacked gcode
    let packed: [u8; 93] = [
        255, 255, 251, 255, 255, 247, 255, 255, 250, 59, 32, 10, 255, 255, 251, 127, 77, 243, 32,
        15, 80, 255, 32, 82, 195, 127, 77, 243, 32, 15, 81, 255, 32, 83, 195, 47, 77, 16, 239, 32,
        4, 0, 255, 32, 89, 4, 0, 255, 32, 90, 2, 240, 32, 43, 5, 192, 47, 77, 48, 239, 32, 3, 240,
        32, 63, 89, 0, 255, 32, 90, 4, 191, 32, 1, 192, 47, 77, 64, 255, 32, 80, 4, 0, 255, 32, 82,
        33, 0, 255, 32, 84, 4, 0,
    ];

    // Initiliase the packer with buffer size depending
    // on your application
    let unpacker = Unpacker::default();

    let mut unpacked: Vec<u8> = Vec::new();
    let _written = unpacker
        .unpack(&mut packed.as_slice(), &mut unpacked)
        .unwrap();

    let unpacked = String::from_utf8(unpacked).expect("Should be valid ASCII");

    println!("{unpacked}");
}
