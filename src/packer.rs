use no_std_io::io::{BufReader, BufWriter, Read, Write};

use crate::core::{forward_lookup, MeatPackError, ENABLE_PACKING_BYTE, LINEFEED_BYTE, SIGNAL_BYTE};

/// A struct that can unpack meatpack packed gcode.
pub struct Packer<const I: usize, const O: usize, R, W>
where
	W: Write,
{
	#[cfg(feature = "std")]
	pub writer: BufWriter<W>,
	#[cfg(feature = "std")]
	reader: BufReader<R>,
	#[cfg(feature = "no_std")]
	writer: BufWriter<W, O>,
	#[cfg(feature = "no_std")]
	reader: BufReader<R, O>,
}

impl<const I: usize, const O: usize, R: Read, W: Write> Packer<I, O, R, W> {
	#[cfg(feature = "std")]
	pub fn new(
		reader: BufReader<R>,
		writer: BufWriter<W>,
	) -> Self {
		Self { reader, writer }
	}

	#[cfg(feature = "no_std")]
	pub fn new(
		reader: BufReader<R, O>,
		writer: BufWriter<W, O>,
	) -> Self {
		Self { reader, writer }
	}

	/// Reads a single byte from the reader
	fn read_one_byte(&mut self) -> Result<u8, MeatPackError> {
		let mut byte: [u8; 1] = [0];
		match self.reader.read_exact(&mut byte) {
			Ok(_) => Ok(byte[0]),
			Err(e) => Err(MeatPackError::IOError(e)),
		}
	}

	pub fn pack(&mut self) -> Result<(), MeatPackError> {
		// Write first command lines.
		let buf = [SIGNAL_BYTE, SIGNAL_BYTE, ENABLE_PACKING_BYTE];
		if let Err(e) = self.writer.write(&buf) {
			return Err(MeatPackError::IOError(e));
		}

		// Now read in the data from inner.
		while let Ok(byte_one) = self.read_one_byte() {
			// Check if the byte is \n and the lower is not populated
			// so we add a \n\n packed byte.
			if byte_one == LINEFEED_BYTE {
				let lf: [u8; 1] = [204];
				if let Err(e) = self.writer.write(&lf) {
					return Err(MeatPackError::IOError(e));
				}
			}

			let byte_two = self.read_one_byte();
			if byte_two.is_err() {
				let e = byte_two.err().unwrap();
				return Err(e);
			}
			let byte_two = byte_two.unwrap();

			let packed_one = forward_lookup(&byte_one, false);
			let packed_two = forward_lookup(&byte_two, false);
			match (packed_one, packed_two) {
				// Both packable
				(Ok(lower), Ok(upper)) => {
					let packed = upper << 4;
					let packed = packed ^ lower;
					if let Err(e) = self.writer.write(&[packed]) {
						return Err(MeatPackError::IOError(e));
					}
				}
				(Ok(lower), Err(_)) => {
					let upper: u8 = 0b1111;
					let packed = upper << 4;
					let packed = packed ^ lower;
					if let Err(e) = self.writer.write(&[packed, byte_two]) {
						return Err(MeatPackError::IOError(e));
					}
				}
				(Err(_), Ok(upper)) => {
					let lower: u8 = 0b1111;
					let packed = upper << 4;
					let packed = packed ^ lower;
					if let Err(e) = self.writer.write(&[packed, byte_one]) {
						return Err(MeatPackError::IOError(e));
					}
				}
				// Both not packable so needs a signal byte in front
				(Err(_), Err(_)) => {
					if let Err(e) = self.writer.write(&[SIGNAL_BYTE, byte_one, byte_two]) {
						return Err(MeatPackError::IOError(e));
					}
				}
			}
		}
		Ok(())
	}
}
