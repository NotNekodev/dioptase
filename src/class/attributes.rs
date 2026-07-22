use std::error::Error;

use crate::class::{class_file::ClassFile, reader::ClassReader};

#[allow(dead_code)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[allow(dead_code)]
pub enum Attribute {
    ConstantValue {
        constantvalue_index: u16,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: Vec<ExceptionTableEntry>,
        attributes: Vec<Attribute>,
    },
}

impl Attribute {
    pub fn read(
        reader: &mut ClassReader,
        class_file: &ClassFile,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let attribute_name_idx = reader.read_u16()?;

        let _attribute_len = reader.read_u32()?;

        let binding = class_file.constant_pool.get_utf8(attribute_name_idx);
        let attribute_name: &str = binding.as_str();

        match attribute_name {
            "ConstantValue" => {
                let constvalue_idx = reader.read_u16()?;
                return Ok(Self::ConstantValue {
                    constantvalue_index: constvalue_idx,
                });
            }
            "Code" => {
                let max_stacks = reader.read_u16()?;
                let max_locals = reader.read_u16()?;

                let code_length = reader.read_u32()?;
                let mut code: Vec<u8> = Vec::new();

                for _ in 0..code_length {
                    code.push(reader.read_u8());
                }

                let exception_table_length = reader.read_u16()?;
                let mut exception_table: Vec<ExceptionTableEntry> = Vec::new();

                for _ in 0..exception_table_length {
                    let start_pc = reader.read_u16()?;
                    let end_pc = reader.read_u16()?;
                    let handler_pc = reader.read_u16()?;
                    let catch_type = reader.read_u16()?;

                    exception_table.push(ExceptionTableEntry {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type,
                    })
                }

                let attributes_count = reader.read_u16()?;
                let mut attributes: Vec<Attribute> = Vec::new();

                for _ in 0..attributes_count {
                    attributes.push(Attribute::read(reader, class_file)?);
                }

                return Ok(Self::Code {
                    max_stack: max_stacks,
                    max_locals: max_locals,
                    code: code,
                    exception_table: exception_table,
                    attributes: attributes,
                });
            }
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown attribute name {}", attribute_name),
                )));
            }
        }
    }
}
