#![no_std]
#![no_main]

use defmt::{error, info};
use meatpack::{Packer, Unpacker};
use {defmt_rtt as _, panic_probe as _};

static GCODE: &[u8] = include_bytes!("../../../test_files/snippet.gcode");

#[cortex_m_rt::entry]
fn main() -> ! {
    let _p = embassy_rp::init(Default::default());

    let mut packer = Packer::new(false, false);
    let mut reader: &[u8] = GCODE;
    let mut packed: [u8; 256] = [0u8; 256];
    let mut writer: &mut [u8] = &mut packed;

    let packer_written = match packer.pack(&mut reader, &mut writer) {
        Ok(packer_written) => packer_written,
        Err(err) => {
            error!("{:?}", err);
            panic!();
        }
    };

    info!("Packed {} into {} bytes", GCODE.len(), packer_written);

    let mut reader: &[u8] = &packed[..packer_written];
    let mut unpacked: [u8; 256] = [0u8; 256];
    let mut writer: &mut [u8] = &mut unpacked;

    let mut unpacker = Unpacker::default();
    let unpacker_written = unpacker.unpack(&mut reader, &mut writer).unwrap();

    info!(
        "Unpacked {} into {} bytes",
        packer_written, unpacker_written
    );

    loop {
        cortex_m::asm::wfi();
    }
}
