use std::error::Error;

use crate::{class::reader::ClassReader, error::RuntimeError};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Double,
    Long,
    Null,
    UninitializedThis,
    Object { cpool_index: u16 },
    Uninitialized { offset: u16 },
}

impl VerificationTypeInfo {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let tag = reader.read_u8()?;
        Ok(match tag {
            0 => Self::Top,
            1 => Self::Integer,
            2 => Self::Float,
            3 => Self::Double,
            4 => Self::Long,
            5 => Self::Null,
            6 => Self::UninitializedThis,
            7 => Self::Object {
                cpool_index: reader.read_u16()?,
            },
            8 => Self::Uninitialized {
                offset: reader.read_u16()?,
            },
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown verification_type_info tag {}", tag),
                )));
            }
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StackMapFrame {
    SameFrame {
        offset_delta: u16,
    },
    SameLocals1StackItemFrame {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    ChopFrame {
        offset_delta: u16,
        chopped_locals: u8,
    },
    SameFrameExtended {
        offset_delta: u16,
    },
    AppendFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    FullFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
}

impl StackMapFrame {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let frame_type = reader.read_u8()?;
        Ok(match frame_type {
            0..=63 => Self::SameFrame {
                offset_delta: frame_type as u16,
            },
            64..=127 => Self::SameLocals1StackItemFrame {
                offset_delta: (frame_type - 64) as u16,
                stack: VerificationTypeInfo::read(reader)?,
            },
            128..=246 => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Reserved stack_map_frame frame_type {}", frame_type),
                )));
            }
            247 => Self::SameLocals1StackItemFrameExtended {
                offset_delta: reader.read_u16()?,
                stack: VerificationTypeInfo::read(reader)?,
            },
            248..=250 => Self::ChopFrame {
                offset_delta: reader.read_u16()?,
                chopped_locals: 251 - frame_type,
            },
            251 => Self::SameFrameExtended {
                offset_delta: reader.read_u16()?,
            },
            252..=254 => {
                let offset_delta = reader.read_u16()?;
                let count = (frame_type - 251) as usize;
                let mut locals = Vec::with_capacity(count);
                for _ in 0..count {
                    locals.push(VerificationTypeInfo::read(reader)?);
                }
                Self::AppendFrame {
                    offset_delta,
                    locals,
                }
            }
            255 => {
                let offset_delta = reader.read_u16()?;
                let locals_count = reader.read_u16()?;
                let mut locals = Vec::with_capacity(locals_count as usize);
                for _ in 0..locals_count {
                    locals.push(VerificationTypeInfo::read(reader)?);
                }
                let stack_count = reader.read_u16()?;
                let mut stack = Vec::with_capacity(stack_count as usize);
                for _ in 0..stack_count {
                    stack.push(VerificationTypeInfo::read(reader)?);
                }
                Self::FullFrame {
                    offset_delta,
                    locals,
                    stack,
                }
            }
        })
    }
}
