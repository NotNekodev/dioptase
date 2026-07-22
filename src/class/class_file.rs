use crate::class::{
    constant_pool::{ConstantPool, ConstantPoolEntry},
    reader::ClassReader,
};

#[allow(dead_code)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
}

impl ClassFile {
    pub fn read(reader: &mut ClassReader) -> Self {
        let magic = reader.read_u32();

        if magic != 0xCAFEBABE {
            panic!(
                "Invalid class file, signature is not 0xCAFEBABE bu {:#010X}",
                magic
            );
        }

        let _minor = reader.read_u16();
        let _major = reader.read_u16();

        let constant_pool = ConstantPool::read(reader);

        println!(
            "Constant pool size: {} entries",
            constant_pool.entries.iter().count() - 1
        );

        let access_flags = reader.read_u16();
        let this_class = reader.read_u16();
        let super_class = reader.read_u16();

        println!("Access flags: {}", access_flags);
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

        let interfaces_count = reader.read_u16();

        println!("Interface count: {}", interfaces_count);

        let mut interfaces = Vec::new();

        for i in 0..interfaces_count {
            let idx = reader.read_u16();

            match constant_pool.entries.get(idx as usize) {
                Some(ConstantPoolEntry::Class { .. }) => {}
                Some(_) => panic!("interfaces[{}] does not point to a CONSTANT_Class entry", i),
                None => panic!("interfaces[{}] index out of bounds", i),
            }

            interfaces.push(idx);
        }

        Self {
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
        }
    }
}
