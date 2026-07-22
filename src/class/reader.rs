use std::array::TryFromSliceError;

pub struct ClassReader {
    data: Vec<u8>,
    pos: usize,
}

impl ClassReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.data[self.pos];
        self.pos += 1;
        value
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
}
