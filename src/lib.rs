#![doc = include_str!("../README.md")]
#![no_std]

mod arrwriter;
mod meat;
mod pack;
mod unpack;

pub use meat::is_meatpack_newline;
pub use pack::{pack_cmd, Pack};
pub use unpack::Unpack;
