use crate::class::reader::ClassReader;

#[allow(dead_code)]
pub struct AttributeInfo {
    pub attribute_name_idx: u16,
    pub info: Vec<u8>,
}

impl AttributeInfo {
    pub fn read(reader: &mut ClassReader) -> Self {
        let attribute_name_idx = reader.read_u16();

        let attribute_len = reader.read_u32();

        let mut info: Vec<u8> = Vec::new();

        for _ in 0..attribute_len {
            info.push(reader.read_u8());
        }

        Self {
            attribute_name_idx,
            info,
        }
    }
}
