use crate::class::{attributes::AttributeInfo, reader::ClassReader};
use anyhow::Result;
use bitflags::bitflags;
use std::error::Error;

// https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.5-200-A.1
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FieldAccessFlags: u16 {
        const PUBLIC    = 0x0001;
        const PRIVATE   = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC    = 0x0008;
        const FINAL     = 0x0010;
        const VOLATILE  = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM      = 0x4000;
    }
}

#[allow(dead_code)]
pub struct FieldInfo {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub access_flags: FieldAccessFlags,
    pub attributes: Vec<AttributeInfo>,
}

impl FieldInfo {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let access_flag_bitmask = reader.read_u16()?;
        let access_flags: FieldAccessFlags =
            FieldAccessFlags::from_bits_truncate(access_flag_bitmask);

        let name_idx = reader.read_u16()?;
        let descriptor_idx = reader.read_u16()?;

        let attributes_count = reader.read_u16()?;
        let mut attributes: Vec<AttributeInfo> = Vec::new();

        for _ in 0..attributes_count {
            attributes.push(AttributeInfo::read(reader)?);
        }

        Ok(Self {
            name_index: name_idx,
            descriptor_index: descriptor_idx,
            access_flags: access_flags,
            attributes: attributes,
        })
    }
}
