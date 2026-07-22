use crate::{
    class::{class_file::ClassFile, constant_pool::ConstantPoolEntry, reader::ClassReader},
    cli::Cli,
};
use anyhow::{Context, Result};
use clap::Parser;
use std::{env, error::Error, fs};

mod class;
mod cli;

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let cli = Cli::parse();

    if cli.version {
        println!("dioptase - {}", env!("CARGO_PKG_DESCRIPTION"));
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Copyright (C) 2026 NotNekodev and contributors");
        println!("SPDX License Identifier: GPL-3.0-only");
        return Ok(());
    }

    let data: Vec<u8> = fs::read(cli.class_file.context("No class file provided")?)
        .context("Could not read the provided class file")?;
    let mut class_reader: ClassReader = ClassReader::new(data);

    let class_file = ClassFile::read(&mut class_reader)?;

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

    println!("\nMethodInfo dump:\n");

    for (i, entry) in class_file.methods.iter().enumerate() {
        let name: String = class_file.constant_pool.get_utf8(entry.name_index);
        let descriptor: String = class_file.constant_pool.get_utf8(entry.descriptor_index);
        println!(
            "Method #{}: Name={} Descriptor={} AccessFlags={:#06x}",
            i,
            name.as_str(),
            descriptor.as_str(),
            entry.access_flags.bits()
        );

        println!("Method #{} Attributes:", i);

        for (j, att_entry) in entry.attributes.iter().enumerate() {
            println!(
                "\tAttribute #{}: Name={} DataSize={:#06x}",
                j,
                class_file
                    .constant_pool
                    .get_utf8(att_entry.attribute_name_idx),
                att_entry.info.iter().count()
            );
        }

        println!();
    }

    Ok(())
}
