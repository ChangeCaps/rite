use std::path::PathBuf;

use clap::Parser;
use rite::Compiler;
use tracing::Level;

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = "main.ri")]
    pub path: PathBuf,
    #[clap(short, long, default_value = "info")]
    pub log_level: Level,
}

fn main() {
    let args = Args::parse();

    let layer = tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .finish();

    tracing::subscriber::set_global_default(layer).unwrap();

    Compiler::compile(args.path);
}
