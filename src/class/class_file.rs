use crate::class::{
    attributes::AttributeInfo,
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
    pub attributes: Vec<AttributeInfo>,
}

impl ClassFile {
    pub fn read(reader: &mut ClassReader) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
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

        let constant_pool = ConstantPool::read(reader)?;

        println!(
            "Constant pool size: {} entries",
            constant_pool.entries.iter().count() - 1
        );

        let access_flags = reader.read_u16()?;
        let this_class = reader.read_u16()?;
        let super_class = reader.read_u16()?;

        println!("Access flags: {:#X}", access_flags);
        println!("this_class: {}", this_class);
        println!("super_class: {}", super_class);

        match constant_pool.entries.get(this_class as usize) {
            Some(ConstantPoolEntry::Class { .. }) => {}
            Some(_) => panic!("this_class does not point to a CONSTANT_Class entry"),
            None => panic!("this_class index out of bounds"),
        }

        match constant_pool.entries.get(super_class as usize) {
            Some(ConstantPoolEntry::Class { .. }) => {}
            Some(_) => panic!("super_class does not point to a CONSTANT_Class entry"),
            None => panic!("super_class index out of bounds"),
        }

        let interfaces_count = reader.read_u16()?;

        println!("Class file contains {} interfaces", interfaces_count);

        let mut interfaces = Vec::new();

        for i in 0..interfaces_count {
            let idx = reader.read_u16()?;

            match constant_pool.entries.get(idx as usize) {
                Some(ConstantPoolEntry::Class { .. }) => {}
                Some(_) => panic!("interfaces[{}] does not point to a CONSTANT_Class entry", i),
                None => panic!("interfaces[{}] index out of bounds", i),
            }

            interfaces.push(idx);
        }

        let fields_count = reader.read_u16()?;

        println!("Class file contains {} fields", fields_count);

        let mut fields: Vec<FieldInfo> = Vec::new();

        for _ in 0..fields_count {
            fields.push(FieldInfo::read(reader)?);
        }

        let methods_count = reader.read_u16()?;
        println!("Class file contains {} methods", methods_count);

        let mut methods: Vec<MethodInfo> = Vec::new();

        for _ in 0..methods_count {
            methods.push(MethodInfo::read(reader)?);
        }

        let attributes_count = reader.read_u16()?;
        println!("Class file contains {} attributes", attributes_count);

        let mut attributes: Vec<AttributeInfo> = Vec::new();

        for _ in 0..attributes_count {
            attributes.push(AttributeInfo::read(reader)?);
        }

        Ok(Self {
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }
}
