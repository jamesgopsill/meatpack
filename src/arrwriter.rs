use crate::meat::MeatPackError;

pub struct ArrWriter<'a, const N: usize> {
	arr: &'a mut [u8; N],
	pos: usize,
}

impl<'a, const N: usize> ArrWriter<'a, N> {
	pub fn new(arr: &'a mut [u8; N]) -> Self {
		arr.fill(0);
		Self { arr, pos: 0 }
	}

	/// Pushed a byte to the buffer.
	pub fn push(
		&mut self,
		byte: &u8,
	) -> Result<(), MeatPackError> {
		if self.pos >= N {
			return Err(MeatPackError::BufferFull);
		}
		self.arr[self.pos] = *byte;
		self.pos += 1;
		Ok(())
	}
}
