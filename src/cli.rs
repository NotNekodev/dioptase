use clap::Parser;

#[derive(Parser)]
#[command(about)]
pub struct Cli {
    /// Print version
    #[arg(short, long)]
    pub version: bool,

    /// If set, the rt.jar file will not be included into the classpath
    #[arg(long)]
    pub no_rt: bool,

    /// If set, the JVM will will not load extensions from $JAVA_HOME/jre/lib/ext
    #[arg(long)]
    pub no_ext: bool,

    /// Classpath, path to directories containing .class files or to .jar archives. Denominator is ; on Windows and : on UNIX
    #[arg(long = "cp", visible_alias = "classpath")]
    pub classpath: Option<String>,

    /// Input class to run
    pub class: Option<String>,
}
