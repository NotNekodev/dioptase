use crate::{
    class::reader::ClassReader,
    error::{InternalError, RuntimeError},
};
use simd_cesu8::mutf8;
use std::error::Error;

#[derive(Clone)]
pub struct ConstantPool {
    pub entries: Vec<ConstantPoolEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ConstantPoolEntry {
    Utf8(String),
    Class {
        name_index: u16,
    },
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String {
        string_index: u16,
    },
    Unknown,
}

#[allow(dead_code)]
impl ConstantPool {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let count = reader.read_u16()?;

        let mut entries = Vec::new();

        // index 0 is unused
        entries.push(ConstantPoolEntry::Unknown);

        let mut i = 1;

        while i < count {
            let tag = reader.read_u8()?;

            let entry = match tag {
                1 => {
                    let length = reader.read_u16()?;

                    let bytes = reader.read_bytes(length as usize)?;

                    let string = mutf8::decode(bytes.as_slice())?.into_owned();

                    ConstantPoolEntry::Utf8(string)
                }

                3 => {
                    let bits = reader.read_u32()?;
                    ConstantPoolEntry::Integer(bits as i32)
                }

                4 => {
                    let bits = reader.read_u32()?;
                    ConstantPoolEntry::Float(f32::from_bits(bits))
                }

                5 => {
                    let bits = reader.read_u64()?;
                    ConstantPoolEntry::Long(bits as i64)
                }

                6 => {
                    let bits = reader.read_u64()?;
                    ConstantPoolEntry::Double(f64::from_bits(bits))
                }

                7 => {
                    let name_index = reader.read_u16()?;

                    ConstantPoolEntry::Class { name_index }
                }

                8 => {
                    let string_index = reader.read_u16()?;
                    ConstantPoolEntry::String { string_index }
                }

                9 => {
                    let class_index = reader.read_u16()?;
                    let name_and_type_index = reader.read_u16()?;

                    ConstantPoolEntry::FieldRef {
                        class_index,
                        name_and_type_index,
                    }
                }

                10 => {
                    let class_index = reader.read_u16()?;
                    let name_and_type_index = reader.read_u16()?;

                    ConstantPoolEntry::MethodRef {
                        class_index,
                        name_and_type_index,
                    }
                }

                11 => {
                    let class_index = reader.read_u16()?;
                    let name_and_type_index = reader.read_u16()?;
                    ConstantPoolEntry::InterfaceMethodRef {
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

                15 => {
                    let reference_kind = reader.read_u8()?;
                    let reference_index = reader.read_u16()?;
                    ConstantPoolEntry::MethodHandle {
                        reference_kind,
                        reference_index,
                    }
                }

                16 => {
                    let descriptor_index = reader.read_u16()?;
                    ConstantPoolEntry::MethodType { descriptor_index }
                }

                18 => {
                    let bootstrap_method_attr_index = reader.read_u16()?;
                    let name_and_type_index = reader.read_u16()?;
                    ConstantPoolEntry::InvokeDynamic {
                        bootstrap_method_attr_index,
                        name_and_type_index,
                    }
                }

                _ => {
                    panic!("Unknown constant pool tag {}", tag);
                }
            };

            let is_wide = matches!(
                entry,
                ConstantPoolEntry::Long(_) | ConstantPoolEntry::Double(_)
            );

            entries.push(entry);
            i += 1;

            if is_wide {
                // JVMS §4.4.5
                entries.push(ConstantPoolEntry::Unknown);
                i += 1;
            }
        }

        Ok(Self { entries })
    }

    pub fn get_utf8(&self, index: u16) -> Result<String, RuntimeError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Utf8(s)) => Ok(s.clone()),
            Some(other) => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "Utf8",
                found: format!("{:?}", other),
            }
            .into()),
            None => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "Utf8",
                found: "out of bounds".to_string(),
            }
            .into()),
        }
    }

    pub fn get_class_name(&self, index: u16) -> Result<String, RuntimeError> {
        let entry = self
            .entries
            .get(index as usize)
            .ok_or(RuntimeError::Internal(
                InternalError::InvalidConstantPoolEntry {
                    index,
                    expected: "Class",
                    found: "out of bounds".to_string(),
                },
            ))?;

        match entry {
            ConstantPoolEntry::Class { name_index } => self.get_utf8(*name_index),
            other => Err(InternalError::InvalidConstantPoolEntry {
                index: index,
                expected: "Class",
                found: format!("{:?}", other),
            }
            .into()),
        }
    }

    pub fn get_name_and_type(&self, index: u16) -> Result<(String, String), RuntimeError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::NameAndType {
                name_index,
                descriptor_index,
            }) => Ok((
                self.get_utf8(*name_index)?,
                self.get_utf8(*descriptor_index)?,
            )),
            Some(other) => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "NameAndType",
                found: format!("{:?}", other),
            }
            .into()),
            None => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "NameAndType",
                found: "out of bounds".to_string(),
            }
            .into()),
        }
    }

    pub fn get_method_ref(&self, index: u16) -> Result<(String, String, String), RuntimeError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::MethodRef {
                class_index,
                name_and_type_index,
            }) => {
                let class_name = self.get_class_name(*class_index)?;
                let (name, descriptor) = self.get_name_and_type(*name_and_type_index)?;
                Ok((class_name, name, descriptor))
            }
            Some(other) => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "MethodRef",
                found: format!("{:?}", other),
            }
            .into()),
            None => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "MethodRef",
                found: "out of bounds".to_string(),
            }
            .into()),
        }
    }

    pub fn get_field_ref(&self, index: u16) -> Result<(String, String, String), RuntimeError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::FieldRef {
                class_index,
                name_and_type_index,
            }) => {
                let class_name = self.get_class_name(*class_index)?;
                let (name, descriptor) = self.get_name_and_type(*name_and_type_index)?;
                Ok((class_name, name, descriptor))
            }
            Some(other) => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "FieldRef",
                found: format!("{:?}", other),
            }
            .into()),
            None => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "Utf8",
                found: "out of bounds".to_string(),
            }
            .into()),
        }
    }

    pub fn get_interface_method_ref(
        &self,
        index: u16,
    ) -> Result<(String, String, String), RuntimeError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            }) => {
                let class_name = self.get_class_name(*class_index)?;
                let (name, descriptor) = self.get_name_and_type(*name_and_type_index)?;
                Ok((class_name, name, descriptor))
            }
            Some(other) => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "InterfaceRef",
                found: format!("{:?}", other),
            }
            .into()),
            None => Err(InternalError::InvalidConstantPoolEntry {
                index,
                expected: "InterfaceRef",
                found: "out of bounds".to_string(),
            }
            .into()),
        }
    }

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}
