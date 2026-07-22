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

    pub fn read_u16(&mut self) -> u16 {
        let bytes: [u8; 2] = self.data[self.pos..self.pos + 2].try_into().unwrap();
        self.pos += 2;
        u16::from_be_bytes(bytes)
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes: [u8; 4] = self.data[self.pos..self.pos + 4].try_into().unwrap();
        self.pos += 4;
        u32::from_be_bytes(bytes)
    }
}
