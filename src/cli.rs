use clap::Parser;

#[derive(Parser)]
#[command(about)]
pub struct Cli {
    /// Print version
    #[arg(short, long)]
    pub version: bool,

    /// Input class file
    pub class_file: Option<String>,
}
