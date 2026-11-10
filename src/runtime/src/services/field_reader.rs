// reads little-endian fields from one capture record; the codec checks the length before it reads
pub(crate) struct FieldReader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> FieldReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    pub(crate) fn rest(&self) -> &'a [u8] {
        &self.bytes[self.at..]
    }

    pub(crate) fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }

    pub(crate) fn flag(&mut self) -> bool {
        self.u8() != 0
    }

    pub(crate) fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take())
    }

    pub(crate) fn i16(&mut self) -> i16 {
        i16::from_le_bytes(self.take())
    }

    pub(crate) fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take())
    }

    pub(crate) fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take())
    }

    pub(crate) fn f32(&mut self) -> f32 {
        f32::from_le_bytes(self.take())
    }

    fn take<const N: usize>(&mut self) -> [u8; N] {
        let mut out = [0u8; N];
        out.copy_from_slice(&self.bytes[self.at..self.at + N]);
        self.at += N;
        out
    }
}
