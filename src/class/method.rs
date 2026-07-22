use crate::class::{attributes::Attribute, class_file::ClassFile, reader::ClassReader};
use anyhow::Result;
use bitflags::bitflags;
use std::error::Error;

// https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.6-200-A.1
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MethodAccessFlags: u16 {
        const PUBLIC       = 0x0001;
        const PRIVATE      = 0x0002;
        const PROTECTED    = 0x0004;
        const STATIC       = 0x0008;
        const FINAL        = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE       = 0x0040;
        const VARARGS      = 0x0080;
        const NATIVE       = 0x0100;
        const ABSTRACT     = 0x0400;
        const STRICT       = 0x0800;
        const SYNTHETIC    = 0x1000;
    }
}

#[allow(dead_code)]
pub struct MethodInfo {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub access_flags: MethodAccessFlags,
    pub attributes: Vec<Attribute>,
}

impl MethodInfo {
    pub fn read(
        reader: &mut ClassReader,
        class_file: &ClassFile,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let access_flags_bitmask = reader.read_u16()?;
        let access_flags = MethodAccessFlags::from_bits_truncate(access_flags_bitmask);

        let name_idx = reader.read_u16()?;
        let descriptor_idx = reader.read_u16()?;

        let attributes_count = reader.read_u16()?;
        let mut attributes: Vec<Attribute> = Vec::new();

        for _ in 0..attributes_count {
            attributes.push(Attribute::read(reader, class_file)?);
        }

        Ok(Self {
            name_index: name_idx,
            descriptor_index: descriptor_idx,
            access_flags: access_flags,
            attributes: attributes,
        })
    }
}
