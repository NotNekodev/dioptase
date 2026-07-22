use std::array::TryFromSliceError;

pub struct ClassReader {
    data: Vec<u8>,
    pos: usize,
}

#[allow(dead_code)]
impl ClassReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn skip(&mut self, len: usize) -> Result<(), TryFromSliceError> {
        let _ = self.data[self.pos..self.pos + len].try_into() as Result<&[u8], _>;
        self.pos += len;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, TryFromSliceError> {
        let bytes: [u8; 1] = self.data[self.pos..self.pos + 1].try_into()?;
        self.pos += 1;
        Ok(bytes[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, TryFromSliceError> {
        let bytes: [u8; 2] = self.data[self.pos..self.pos + 2].try_into()?;
        self.pos += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    pub fn read_u32(&mut self) -> Result<u32, TryFromSliceError> {
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into()?;
        self.pos += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, TryFromSliceError> {
        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;
        Ok(bytes)
    }
}
