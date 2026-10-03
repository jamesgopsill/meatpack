/// The bytes (0 to 3) produced by a single `pack_byte` call.
pub(crate) struct Emit {
    buf: [u8; 3],
    len: usize,
}

impl Emit {
    pub const NONE: Self = Self {
        buf: [0; 3],
        len: 0,
    };

    pub fn one(a: u8) -> Self {
        Self {
            buf: [a, 0, 0],
            len: 1,
        }
    }

    pub fn two(
        a: u8,
        b: u8,
    ) -> Self {
        Self {
            buf: [a, b, 0],
            len: 2,
        }
    }

    pub fn three(
        a: u8,
        b: u8,
        c: u8,
    ) -> Self {
        Self {
            buf: [a, b, c],
            len: 3,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}
