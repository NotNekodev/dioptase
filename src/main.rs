use crate::{
    cli::Cli,
    error::RuntimeError,
    vm::{classpath::ClassPath, value::Value, vm::VM},
};
use anyhow::Result;
use clap::Parser;
use std::{env, error::Error, process::ExitCode};

mod class;
mod cli;
mod error;
mod vm;

fn main() -> ExitCode {
    match real_main() {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<i32, Box<dyn Error + Send + Sync + 'static>> {
    let args: Vec<String> = std::env::args()
        .map(|a| if a == "-cp" { "--cp".to_string() } else { a })
        .collect();
    let cli = Cli::parse_from(args);

    if cli.version {
        println!("dioptase - {}", env!("CARGO_PKG_DESCRIPTION"));
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Copyright (C) 2026 NotNekodev and contributors");
        println!("SPDX License Identifier: GPL-3.0-only");
        return Ok(0);
    }

    let classpath = match &cli.classpath {
        Some(cp) => ClassPath::parse(cp),
        None => ClassPath::empty(),
    };

    let main_class = match &cli.class {
        Some(class) => class,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "no main class specified",
            )
            .into());
        }
    };

    let mut vm: VM = VM::new();
    vm.set_classpath(classpath);

    println!("JVM Main file: {}", main_class);

    let ret_value = vm.run_main(main_class)?;

    match ret_value {
        Value::Int(val) => Ok(val),
        _ => return Err(Box::new(RuntimeError::InvalidType)),
    }
}
