use clap::{Parser, Subcommand};
use meatpack::{MeatPackResult, Packer, Unpacker};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, IsTerminal as _, Write},
    path::{Path, PathBuf},
    process,
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
            let mut reader = open_input(input, false, cli.force)?;
            let mut writer = open_output(cli.output.as_deref(), true, true)?;
            let mut packer = Packer::new(*strip_comments, *strip_whitespace);
            let stats = pack_stream(&mut reader, &mut writer.bufwriter, &mut packer);
            if stats.is_err() {
                writer.close_and_delete();
            }
            let stats = stats?;
            eprintln!("Bytes Read: {}", stats.bytes_in);
            eprintln!("Bytes Written: {}", stats.bytes_out);
        }
        Command::Unpack { input } => {
            let mut reader = open_input(input, false, cli.force)?;
            let mut writer = open_output(cli.output.as_deref(), true, true)?;

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
                        writer.bufwriter.write_all(line).unwrap();
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
                    "Data remains in the unpacker. The last line was not terminated by a new line."
                )
            }

            eprintln!("Lines unpacked: {}", line_count);
            eprintln!(
                "{} packed bytes -> {} unpacked bytes",
                packed_byte_count, unpacked_byte_count,
            );
        }
    }
    Ok(())
}

fn is_stdio(p: &Path) -> bool {
    p == Path::new("-")
}

pub type GenericReader = Box<dyn BufRead>;

/// `binary`: refuse to read from a terminal (unpack reads binary input).
fn open_input(
    path: &Path,
    binary: bool,
    force: bool,
) -> Result<GenericReader, CliError> {
    if is_stdio(path) {
        let stdin = io::stdin();
        if binary && stdin.is_terminal() && !force {
            return Err(CliError::Terminal("read binary data from"));
        }
        // StdinLock already implements BufRead, so no extra BufReader.
        Ok(Box::new(stdin.lock()))
    } else {
        let file = File::open(path).map_err(|e| CliError::io(path, e))?;
        Ok(Box::new(BufReader::new(file)))
    }
}

pub struct GenericWriter {
    bufwriter: BufWriter<Box<dyn Write>>,
    path: Option<PathBuf>,
}

impl GenericWriter {
    fn new(
        bufwriter: BufWriter<Box<dyn Write>>,
        path: Option<PathBuf>,
    ) -> Self {
        Self { bufwriter, path }
    }

    fn close_and_delete(self) {
        let writer = self.bufwriter;
        let path = self.path;
        drop(writer);
        if let Some(p) = path {
            let _ = std::fs::remove_file(p);
        }
    }
}

// Returns the writer, plus the file path (if any) so it can be
/// removed if processing fails.
fn open_output(
    path: Option<&Path>,
    binary: bool,
    force: bool,
) -> Result<GenericWriter, CliError> {
    match path.filter(|p| !is_stdio(p)) {
        None => {
            let stdout = io::stdout();
            if binary && stdout.is_terminal() && !force {
                return Err(CliError::Terminal("write binary data to"));
            }
            Ok(GenericWriter::new(
                BufWriter::new(Box::new(stdout.lock())),
                None,
            ))
        }
        Some(p) => {
            let file = if force {
                File::create(p)
            } else {
                File::create_new(p) // fails if the file already exists
            }
            .map_err(|e| CliError::io(p, e))?;
            Ok(GenericWriter::new(
                BufWriter::new(Box::new(file)),
                Some(p.to_path_buf()),
            ))
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
    Pack(#[from] meatpack::MeatPackError),
    #[error("refusing to {0} a terminal (use -f to force)")]
    Terminal(&'static str),
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

struct Stats {
    bytes_in: usize,
    bytes_out: usize,
    lines: usize,
}

fn pack_stream<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    packer: &mut Packer<128>,
) -> Result<Stats, CliError> {
    let mut stats = Stats {
        bytes_in: 0,
        bytes_out: 0,
        lines: 0,
    };
    loop {
        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            break; // EOF, and only EOF
        }
        for &b in buf {
            if let MeatPackResult::Line(line) = packer.pack(&b)? {
                writer.write_all(line)?;
                stats.bytes_out += line.len();
                stats.lines += 1;
            }
        }
        let n = buf.len();
        stats.bytes_in += n;
        reader.consume(n);
    }
    Ok(stats)
}
