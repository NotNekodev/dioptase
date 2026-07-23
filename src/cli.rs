use clap::Parser;

#[derive(Parser)]
#[command(about)]
pub struct Cli {
    /// Print version
    #[arg(short, long)]
    pub version: bool,

    /// Classpath, path to directories containing .class files or to .jar archives. Denominator is ; on Windows and : on UNIX
    #[arg(long = "cp", visible_alias = "classpath")]
    pub classpath: Option<String>,

    /// Input class to run
    pub class: Option<String>,
}
