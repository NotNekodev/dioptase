use crate::{
    cli::Cli,
    error::{InternalError, RuntimeError},
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
            eprintln!("\x1b[1;31merror:\x1b[0m {}", err);
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

    let mut classpath = match &cli.classpath {
        Some(cp) => ClassPath::parse(cp),
        None => ClassPath::empty(),
    };

    if !cli.no_rt {
        classpath.add_bootstrap("/usr/lib/jvm/openjdk8/jre/lib/rt.jar".into());
    }

    if !cli.no_ext {
        for entry in std::fs::read_dir("/usr/lib/jvm/openjdk8/jre/lib/ext")? {
            let path = entry?.path();
            classpath.add_extension(path);
        }
    }

    let main_class = match &cli.class {
        Some(class) => class,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No main class to run specified",
            )
            .into());
        }
    };

    let mut vm: VM = VM::new();
    vm.set_classpath(classpath);

    match vm.run_main(main_class) {
        Ok(Value::Int(val)) => Ok(val),
        Ok(_) => Err(Box::new(RuntimeError::from(InternalError::InvalidType))),
        Err(RuntimeError::Thrown(obj_ref)) => {
            eprintln!(
                "\x1b[1;31merror:\x1b[0m Exception in thread \"main\" {}",
                vm.describe_exception(obj_ref)
            );
            Ok(1)
        }
        Err(e) => Err(Box::new(e)),
    }
}
