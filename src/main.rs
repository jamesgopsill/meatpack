use clap::{Parser, Subcommand};
use meatpack::{MeatPackResult, Packer, Unpacker, MEATPACK_HEADER};
use std::{
	fs::File,
	io::{BufReader, BufWriter, Read, Write},
	path::PathBuf,
	process,
};

/// Command line options
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Command>,
}

/// The different commands that can be parsed
#[derive(Debug, Subcommand)]
enum Command {
	Pack { infile: PathBuf, outfile: PathBuf },
	Unpack { infile: PathBuf, outfile: PathBuf },
}

/// CLI
fn main() {
	println!("MeatPack!");
	let cli = Cli::parse();

	match &cli.command {
		Some(Command::Pack { infile, outfile }) => {
			let infile = File::open(infile).unwrap();
			let mut reader = BufReader::new(infile);

			let outfile = File::create(outfile).unwrap();
			let mut writer = BufWriter::new(outfile);

			let mut packer = Packer::<128>::default();

			writer.write_all(&MEATPACK_HEADER).unwrap();

			let mut line_count: u32 = 0;
			let mut byte = [0u8];
			while reader.read_exact(byte.as_mut_slice()).is_ok() {
				match packer.pack(&byte[0]) {
					Ok(MeatPackResult::Line(line)) => {
						line_count += 1;
						writer.write_all(line).unwrap();
					}
					Ok(MeatPackResult::WaitingForNextByte) => {}
					Err(e) => {
						println!("{:?}", e);
						process::exit(1);
					}
				}
			}

			if packer.data_remains() {
				eprintln!("Data remains in the packer. Please make sure the last line is terminated with a new line.")
			}

			println!("Lines packed: {}", line_count);
		}
		Some(Command::Unpack { infile, outfile }) => {
			let infile = File::open(infile).unwrap();
			let mut reader = BufReader::new(infile);

			let outfile = File::create(outfile).unwrap();
			let mut writer = BufWriter::new(outfile);

			let mut unpacker = Unpacker::<128>::default();

			let mut line_count: u32 = 0;
			let mut byte: [u8; 1] = [0];
			while reader.read_exact(byte.as_mut_slice()).is_ok() {
				match unpacker.unpack(&byte[0]) {
					Ok(MeatPackResult::Line(line)) => {
						line_count += 1;
						writer.write_all(line).unwrap();
					}
					Ok(MeatPackResult::WaitingForNextByte) => {}
					Err(e) => {
						println!("{:?}", e);
						process::exit(1);
					}
				}
			}

			if unpacker.data_remains() {
				eprintln!("Data remains in the unpacker. The last line is not terminated with a new line.")
			}

			println!("Lines packed: {}", line_count);
		}
		None => {
			println!("Please provide a subcommand --pack or --unpack");
		}
	}
}
