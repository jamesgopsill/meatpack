#![doc = include_str!("../README.md")]
#![cfg_attr(feature = "no_std", no_std)]

pub mod core;
//pub mod _unpacked_lines;
pub mod unpacker;

pub use unpacker::Unpacker;
