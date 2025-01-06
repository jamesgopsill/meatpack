#[cfg(feature = "no_std")]
use no_std_io::io::{BufReader, Read, Result, Seek, SeekFrom};
#[cfg(feature = "std")]
use std::io::{BufReader, Read, Result, Seek, SeekFrom};

use super::unpacked_lines::UnpackedLines;

/// A struct that can unpack meatpack packed gcode.
pub struct Unpacker<R> {
    #[cfg(feature = "std")]
    reader: BufReader<R>,
    #[cfg(feature = "no_std")]
    reader: BufReader<R, 128>,
}

impl<R> Unpacker<R> {
    #[cfg(feature = "std")]
    pub fn new(reader: BufReader<R>) -> Self {
        Self { reader }
    }

    #[cfg(feature = "no_std")]
    pub fn new(reader: BufReader<R, 128>) -> Self {
        Self { reader }
    }
}

impl<R: Read> Unpacker<R> {
    pub fn lines(&mut self) -> UnpackedLines<'_, R> {
        UnpackedLines::new(&mut self.reader)
    }
}

impl<R: Read> Read for Unpacker<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Seek> Seek for Unpacker<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.reader.seek(pos)
    }
}
