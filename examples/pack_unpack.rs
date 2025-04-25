use std::process;

use meatpack::{MeatPackResult, Packer, Unpacker, MEATPACK_HEADER};

fn main() {
    let gcode = "M73 P0 R3
M73 Q0 S3 ; Hello
M201 X4000 Y4000 Z200 E2500
M203 X300 Y300 Z40 E100
M204 P4000 R1200 T4000
";

    println!("## IN ##");
    println!("{}", gcode);
    println!("####");

    // Initiliase the packer with buffer size depending
    // on your application
    let mut packer = Packer::<64>::default();
    let mut out: Vec<u8> = vec![];

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
                println!("{:?}", line);
                out.extend(line);
            }
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Err(e) => println!("{:?}", e),
        }
    }

    println!("{:?}", out);

    println!("## OUT ##");

    // Now we create an unpacker to unpack the meatpacked data.
    let mut unpacker = Unpacker::<64>::default();

    // Imagine receiving the bytes from some I/O and we want
    // to construct gcode lines and deal with them as we form them.
    for byte in out.iter() {
        let res = unpacker.unpack(byte);
        match res {
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Ok(MeatPackResult::Line(line)) => {
                // If in std.
                for byte in line {
                    let c = char::from(*byte);
                    print!("{}", c);
                }
            }
            Err(e) => {
                println!("{:?}", e);
                process::exit(0)
            }
        }
    }

    println!("####");
}
