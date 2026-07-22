use crate::class::reader::ClassReader;
use std::error::Error;

pub struct ConstantPool {
    pub entries: Vec<ConstantPoolEntry>,
}

pub enum ConstantPoolEntry {
    Utf8(String),
    Class {
        name_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    Unknown(()),
}

impl ConstantPool {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let count = reader.read_u16()?;

        let mut entries = Vec::new();

        // index 0 is unused
        entries.push(ConstantPoolEntry::Unknown(()));

        let mut i = 1;

        while i < count {
            let tag = reader.read_u8();

            let entry = match tag {
                1 => {
                    let length = reader.read_u16()?;

                    let mut bytes = Vec::new();

                    for _ in 0..length {
                        bytes.push(reader.read_u8());
                    }

                    let string = String::from_utf8(bytes)?;

                    ConstantPoolEntry::Utf8(string)
                }

                7 => {
                    let name_index = reader.read_u16()?;

                    ConstantPoolEntry::Class { name_index }
                }

                10 => {
                    let class_index = reader.read_u16()?;
                    let name_and_type_index = reader.read_u16()?;

                    ConstantPoolEntry::MethodRef {
                        class_index,
                        name_and_type_index,
                    }
                }

                12 => {
                    let name_index = reader.read_u16()?;
                    let descriptor_index = reader.read_u16()?;

                    ConstantPoolEntry::NameAndType {
                        name_index,
                        descriptor_index,
                    }
                }

                _ => {
                    panic!("Unknown constant pool tag {}", tag);
                }
            };

            entries.push(entry);
            i += 1;
        }

        Ok(Self { entries })
    }
}
