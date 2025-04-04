#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod meat;
mod pack;
mod unpack;

pub use meat::MeatPackError;
pub use meat::MeatPackResult;
pub use meat::MEATPACK_HEADER;
pub use pack::Packer;
pub use unpack::Unpacker;
