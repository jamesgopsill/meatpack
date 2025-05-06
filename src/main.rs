use clap::{Parser, Subcommand};
use meatpack::{MEATPACK_HEADER, MeatPackResult, NO_SPACES_COMMAND, Packer, Unpacker};
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
    Pack {
        #[arg(long, default_value_t = false)]
        strip_comments: bool,
        #[arg(long, default_value_t = false)]
        strip_whitespace: bool,
        infile: PathBuf,
        outfile: PathBuf,
    },
    Unpack {
        infile: PathBuf,
        outfile: PathBuf,
    },
}

/// CLI
fn main() {
    println!("MeatPack!");
    let cli = Cli::parse();

    match &cli.command {
        Some(Command::Pack {
            strip_comments,
            strip_whitespace,
            infile,
            outfile,
        }) => {
            println!(
                "Packing {} into {}",
                infile.to_str().unwrap(),
                outfile.to_str().unwrap()
            );
            println!("Strip Comments: {}", strip_comments);
            println!("Strip Whitespace: {}", strip_whitespace);

            let infile = File::open(infile).unwrap();
            let mut reader = BufReader::new(infile);

            let outfile = File::create(outfile).unwrap();
            let mut writer = BufWriter::new(outfile);

            let mut packer = Packer::<128>::new(*strip_comments, *strip_whitespace);

            writer.write_all(&MEATPACK_HEADER).unwrap();
            if *strip_whitespace {
                writer.write_all(&NO_SPACES_COMMAND).unwrap();
            }

            let mut line_count: usize = 0;
            let mut unpacked_byte_count: usize = 0;
            let mut packed_byte_count: usize = 0;
            let mut byte: [u8; 1] = [0];
            while reader.read_exact(byte.as_mut_slice()).is_ok() {
                unpacked_byte_count += 1;
                match packer.pack(&byte[0]) {
                    Ok(MeatPackResult::Line(line)) => {
                        line_count += 1;
                        packed_byte_count += line.len();
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
                eprintln!(
                    "Data remains in the packer. Please make sure the last line is terminated with a new line."
                )
            }

            println!("Lines Packed: {}", line_count);
            println!(
                "{} unpacked bytes -> {} packed bytes ({}%)",
                unpacked_byte_count,
                packed_byte_count,
                (packed_byte_count as f32 / unpacked_byte_count as f32) * 100.0
            );
        }
        Some(Command::Unpack { infile, outfile }) => {
            println!(
                "Unpacking {} into {}",
                infile.to_str().unwrap(),
                outfile.to_str().unwrap()
            );
            let infile = File::open(infile).unwrap();
            let mut reader = BufReader::new(infile);

            let outfile = File::create(outfile).unwrap();
            let mut writer = BufWriter::new(outfile);

            let mut unpacker = Unpacker::<128>::default();

            let mut line_count: usize = 0;
            let mut byte: [u8; 1] = [0];
            let mut unpacked_byte_count: usize = 0;
            let mut packed_byte_count: usize = 0;
            while reader.read_exact(byte.as_mut_slice()).is_ok() {
                packed_byte_count += 1;
                match unpacker.unpack(&byte[0]) {
                    Ok(MeatPackResult::Line(line)) => {
                        line_count += 1;
                        unpacked_byte_count += line.len();
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
                eprintln!(
                    "Data remains in the unpacker. The last line is not terminated with a new line."
                )
            }

            println!("Lines unpacked: {}", line_count);
            println!(
                "{} packed bytes -> {} unpacked bytes",
                packed_byte_count, unpacked_byte_count,
            );
        }
        None => {
            println!("Please provide a subcommand --pack or --unpack");
        }
    }
}
