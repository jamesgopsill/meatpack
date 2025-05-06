#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(test)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod components;

pub use components::meat::MeatPackError;
pub use components::meat::MeatPackResult;
pub use components::meat::{MEATPACK_HEADER, NO_SPACES_COMMAND};
pub use components::pack::Packer;
pub use components::unpack::Unpacker;
