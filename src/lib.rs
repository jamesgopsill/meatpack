#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "no_std", no_std)]

pub mod core;
mod packer;
mod unpacker;

pub use packer::Packer;
pub use unpacker::Unpacker;
