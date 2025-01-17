#![doc = include_str!("../README.md")]
//#![no_std]

mod arrwriter;
mod meat;
mod pack;
mod parser;
mod unpack;

pub use meat::is_meatpack_newline;
pub use pack::{pack_cmd, Pack};
pub use parser::{MeatPackOutput, Parser};
pub use unpack::Unpack;
