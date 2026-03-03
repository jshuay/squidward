use std::path::PathBuf;

use clap::Parser;
use log::LevelFilter;
use log::debug;

/// Squidward payment system simulator
#[derive(Parser, Debug)]
struct Args {
    /// Path to the transactions file (CSV format)
    transactions_file: PathBuf,
    /// Output debug logs to STDOUT
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let mut log_level = LevelFilter::Off;
    if args.debug {
        log_level = LevelFilter::Debug;
    }
    env_logger::builder().filter_level(log_level).init();

    debug!("Debug logs enabled");
    debug!("Arguments: {:?}", args);
}
