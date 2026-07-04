pub struct Cursor<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let b = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    pub fn peek_byte(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    pub fn advance(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.data.len());
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        if end > self.data.len() {
            return None;
        }
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    pub fn read_i8(&mut self) -> Option<i8> {
        let b = self.read_byte()?;
        Some(b as i8)
    }

    pub fn read_i16(&mut self) -> Option<i16> {
        let bytes: [u8; 2] = self.read_bytes(2)?.try_into().ok()?;
        Some(i16::from_be_bytes(bytes))
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        let bytes: [u8; 4] = self.read_bytes(4)?.try_into().ok()?;
        Some(i32::from_be_bytes(bytes))
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        let bytes: [u8; 8] = self.read_bytes(8)?.try_into().ok()?;
        Some(i64::from_be_bytes(bytes))
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        let bytes: [u8; 4] = self.read_bytes(4)?.try_into().ok()?;
        Some(f32::from_be_bytes(bytes))
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        let bytes: [u8; 8] = self.read_bytes(8)?.try_into().ok()?;
        Some(f64::from_be_bytes(bytes))
    }
}
