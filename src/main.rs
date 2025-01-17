use std::{
	fs::File,
	io::{BufRead, BufReader, BufWriter, Read, Write},
	path::PathBuf,
};

use clap::{Parser, Subcommand};
use meatpack::{is_meatpack_newline, Pack, Unpack};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
	Pack { infile: PathBuf, outfile: PathBuf },
	Unpack { infile: PathBuf, outfile: PathBuf },
}

fn main() {
	println!("MeatPack!");
	let cli = Cli::parse();

	println!("{:?}", cli);

	match &cli.command {
		Some(Command::Pack { infile, outfile }) => {
			let infile = File::open(infile).unwrap();
			let mut reader = BufReader::new(infile);

			let outfile = File::create(outfile).unwrap();
			let mut writer = BufWriter::new(outfile);

			let mut buf: Vec<u8> = vec![];
			let mut packer = Pack::<100>::default();

			writer.write_all(&packer.header()).unwrap();

			while reader.read_until(10, &mut buf).is_ok() {
				if buf.is_empty() {
					break;
				}
				let packed = packer.pack(buf.as_slice()).unwrap();
				if !packed.is_empty() {
					writer.write_all(packed).unwrap();
				}
				buf.clear();
			}
		}
		Some(Command::Unpack { infile, outfile }) => {
			let infile = File::open(infile).unwrap();
			let mut reader = BufReader::new(infile);

			let outfile = File::create(outfile).unwrap();
			let mut writer = BufWriter::new(outfile);

			let mut buf: Vec<u8> = vec![];
			let mut unpacker = Unpack::<100>::default();

			let mut byte: [u8; 1] = [0];

			while reader.read_exact(&mut byte).is_ok() {
				buf.extend(byte);
				if is_meatpack_newline(&byte[0]) {
					let unpacked = unpacker.unpack(buf.as_slice()).unwrap();
					writer.write_all(unpacked).unwrap();
					buf.clear();
				}
			}
		}
		None => {
			println!("Please provide a subcommand --pack or --unpack");
		}
	}
}

/*
fn print_cmd(cmd: &[u8]) {
	for byte in cmd {
		let c = char::from(*byte);
		print!("{}", c);
	}
}
*/
