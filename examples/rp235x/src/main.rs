#![no_std]
#![no_main]

use defmt::{Debug2Format, info};
use embassy_time::Instant;
use meatpack::{MEATPACK_HEADER, MeatPackResult, Packer, Unpacker};
use {defmt_rtt as _, panic_probe as _};

static GCODE: &[u8] = include_bytes!("../../../test_files/box.gcode");

#[cortex_m_rt::entry]
fn main() -> ! {
    let _p = embassy_rp::init(Default::default());

    let mut packer = Packer::<128>::new(true, false);
    let mut unpacker = Unpacker::<128>::default();
    let (mut packed_bytes, mut lines) = (MEATPACK_HEADER.len(), 0usize);

    let start = Instant::now();
    for &b in MEATPACK_HEADER.iter() {
        unpacker.unpack(b).unwrap();
    }
    for &b in GCODE {
        match packer.pack(b) {
            Ok(MeatPackResult::Line(line)) => {
                packed_bytes += line.len();
                for &pb in line {
                    if let Ok(MeatPackResult::Line(_)) = unpacker.unpack(pb) {
                        lines += 1;
                    }
                }
            }
            Ok(MeatPackResult::WaitingForNextByte) => {}
            Err(e) => defmt::panic!("pack failed: {}", Debug2Format(&e)),
        }
    }
    let elapsed = start.elapsed();

    info!(
        "{} B -> {} B ({} lines) in {} ms",
        GCODE.len(),
        packed_bytes,
        lines,
        elapsed.as_millis()
    );
    loop {
        cortex_m::asm::wfi();
    }
}
