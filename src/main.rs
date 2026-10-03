use clap::{Parser, Subcommand};
use embedded_io_adapters::std::FromStd;
use meatpack::{Packer, Unpacker};
use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

/// Command line options
#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long, global = true, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Overwrite existing files and allow binary output to a terminal
    #[arg(short, long, global = true)]
    force: bool,
    /// Print a summary to stderr
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    verbose: bool,
    /// Suppress warnings
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Pack G-code into MeatPack
    Pack {
        #[arg(long)]
        strip_comments: bool,
        #[arg(long)]
        strip_whitespace: bool,
        /// Input file ("-" or omitted for stdin)
        #[arg(default_value = "-")]
        input: PathBuf,
    },
    /// Unpack MeatPack into G-code
    Unpack {
        #[arg(default_value = "-")]
        input: PathBuf,
    },
}

/// CLI
fn main() -> Result<(), CliError> {
    let cli = Cli::parse();
    match &cli.command {
        Command::Pack {
            strip_comments,
            strip_whitespace,
            input,
        } => {
            let mut reader = open_input(input)?;
            let mut writer = open_output(cli.output.as_deref(), cli.force)?;
            let packer = Packer::new(*strip_comments, *strip_whitespace);
            let written = packer.pack(&mut reader, &mut writer)?;
            eprintln!("Packed {written} bytes");
        }
        Command::Unpack { input } => {
            let mut reader = open_input(input)?;
            let mut writer = open_output(cli.output.as_deref(), cli.force)?;
            let unpacker = Unpacker::default();
            let written = unpacker.unpack(&mut reader, &mut writer)?;
            eprintln!("Unpacked {written} bytes");
        }
    }
    Ok(())
}

fn is_stdio(p: &Path) -> bool {
    p == Path::new("-")
}

pub type GenericReader = Box<dyn embedded_io::BufRead<Error = std::io::Error>>;

/// `binary`: refuse to read from a terminal (unpack reads binary input).
fn open_input(path: &Path) -> Result<GenericReader, CliError> {
    if is_stdio(path) {
        let stdin = io::stdin();
        // StdinLock already implements BufRead, so no extra BufReader.
        Ok(Box::new(FromStd::new(stdin.lock())))
    } else {
        let file = File::open(path).map_err(|e| CliError::io(path, e))?;
        Ok(Box::new(FromStd::new(BufReader::new(file))))
    }
}

pub type GenericWriter = Box<dyn embedded_io::Write<Error = std::io::Error>>;

// Returns the writer, plus the file path (if any) so it can be
/// removed if processing fails.
fn open_output(
    path: Option<&Path>,
    force: bool,
) -> Result<GenericWriter, CliError> {
    match path.filter(|p| !is_stdio(p)) {
        None => {
            let stdout = io::stdout();
            Ok(Box::new(FromStd::new(stdout.lock())))
        }
        Some(p) => {
            let file = if force {
                File::create(p)
            } else {
                File::create_new(p) // fails if the file already exists
            }
            .map_err(|e| CliError::io(p, e))?;
            Ok(Box::new(FromStd::new(file)))
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("{path}: {source}")]
    File {
        path: String,
        source: std::io::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Pack(#[from] meatpack::Error),
}

impl CliError {
    fn io(
        path: &Path,
        source: io::Error,
    ) -> Self {
        Self::File {
            path: path.display().to_string(),
            source,
        }
    }
}
