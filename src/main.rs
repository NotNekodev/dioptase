mod class;

use std::{env, fs};

use class::reader::ClassReader;

use crate::class::{class_file::ClassFile, constant_pool::ConstantPoolEntry};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().count() < 2 {
        panic!("No .class file to run provided!");
    }

    let class_file = &args[1];

    let data: Vec<u8> = fs::read(class_file).unwrap();
    let mut class_reader: ClassReader = ClassReader::new(data);

    let class_file = ClassFile::read(&mut class_reader);

    println!("\nConstant pool dump:");

    for (i, entry) in class_file.constant_pool.entries.iter().enumerate() {
        println!(
            "#{} {}",
            i,
            match entry {
                ConstantPoolEntry::Utf8(s) => format!("Utf8(value=\"{}\")", s),
                ConstantPoolEntry::Class { name_index } =>
                    format!("Class(name_idx={})", name_index),
                ConstantPoolEntry::MethodRef {
                    class_index,
                    name_and_type_index,
                } => format!(
                    "MethodRef(class_idx={}, name_and_type_index={})",
                    class_index, name_and_type_index
                ),
                ConstantPoolEntry::NameAndType {
                    name_index,
                    descriptor_index,
                } => format!(
                    "NameAndType(name_idx={}, descriptor_idx={}",
                    name_index, descriptor_index
                ),
                ConstantPoolEntry::Unknown(_) => "Unknown".to_string(),
            }
        );
    }
}
