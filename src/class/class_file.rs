use crate::class::{
    attributes::Attribute,
    constant_pool::{ConstantPool, ConstantPoolEntry},
    field::FieldInfo,
    method::MethodInfo,
    reader::ClassReader,
};
use std::error::Error;

#[allow(dead_code)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let mut class_file: Self = Self::new();
        let magic = reader.read_u32()?;

        if magic != 0xCAFEBABE {
            panic!(
                "Invalid class file, signature is not 0xCAFEBABE but {:#010X}",
                magic
            );
        }

        let _minor = reader.read_u16()?;
        let major = reader.read_u16()?;

        if major != 52 {
            panic!("Invalid class file, version is not 52 but {}", major);
        }

        class_file.constant_pool = ConstantPool::read(reader)?;

        class_file.access_flags = reader.read_u16()?;
        class_file.this_class = reader.read_u16()?;
        class_file.super_class = reader.read_u16()?;

        match class_file
            .constant_pool
            .entries
            .get(class_file.this_class as usize)
        {
            Some(ConstantPoolEntry::Class { .. }) => {}
            Some(_) => panic!("this_class does not point to a CONSTANT_Class entry"),
            None => panic!("this_class index out of bounds"),
        }

        match class_file
            .constant_pool
            .entries
            .get(class_file.super_class as usize)
        {
            Some(ConstantPoolEntry::Class { .. }) => {}
            Some(_) => panic!("super_class does not point to a CONSTANT_Class entry"),
            None => panic!("super_class index out of bounds"),
        }

        let interfaces_count = reader.read_u16()?;

        for i in 0..interfaces_count {
            let idx = reader.read_u16()?;

            match class_file.constant_pool.entries.get(idx as usize) {
                Some(ConstantPoolEntry::Class { .. }) => {}
                Some(_) => panic!("interfaces[{}] does not point to a CONSTANT_Class entry", i),
                None => panic!("interfaces[{}] index out of bounds", i),
            }

            class_file.interfaces.push(idx);
        }

        let fields_count = reader.read_u16()?;

        for _ in 0..fields_count {
            class_file
                .fields
                .push(FieldInfo::read(reader, &class_file)?);
        }

        let methods_count = reader.read_u16()?;

        for _ in 0..methods_count {
            class_file
                .methods
                .push(MethodInfo::read(reader, &class_file)?);
        }

        let attributes_count = reader.read_u16()?;

        for _ in 0..attributes_count {
            class_file
                .attributes
                .push(Attribute::read(reader, &class_file)?);
        }

        Ok(class_file)
    }

    pub fn new() -> Self {
        Self {
            constant_pool: ConstantPool::new(),
            access_flags: 0,
            this_class: 0,
            super_class: 0,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
        }
    }
}
