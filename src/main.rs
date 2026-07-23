use crate::{
    class::{class_file::ClassFile, reader::ClassReader},
    cli::Cli,
    vm::{
        runtime_class::{ClassRef, RuntimeClass},
        vm::VM,
    },
};
use anyhow::{Context, Result};
use clap::Parser;
use std::{env, error::Error, fs};

mod class;
mod cli;
mod error;
mod vm;

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
    let mut vm: VM = VM::new();

    let rt_class_ref: ClassRef = vm
        .load_class(class_file)
        .expect("Failed to load .class file");

    println!("\n.class file ref: {}", rt_class_ref.0);

    let class: &RuntimeClass = vm.get_class(rt_class_ref).expect("Invalid rt_class_ref");

    println!("Class name: {}", class.name);
    println!("Class method count: {}", class.methods.iter().count());
    println!("Class methods:");
    for (i, method) in class.methods.iter().enumerate() {
        println!(
            "\t#{}: {}{} ({:#06x})",
            i,
            method.name,
            method.descriptor,
            method.access.bits()
        );
    }

    Ok(())
}
