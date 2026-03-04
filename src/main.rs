mod payment_system;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use csv::Trim;
use log::LevelFilter;
use log::debug;
use log::error;

/// Squidward payment system simulator
#[derive(Parser, Debug)]
struct Args {
    /// Path to the transactions file (CSV format)
    transactions_file: PathBuf,
    /// Output debug logs to STDOUT
    #[arg(short, long)]
    debug: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let mut log_level = LevelFilter::Off;
    if args.debug {
        log_level = LevelFilter::Debug;
    }
    env_logger::builder().filter_level(log_level).init();

    debug!("Debug logs enabled");
    debug!("Arguments: {:?}", args);

    debug!("Reading input transactions CSV file: {:?}", args.transactions_file);

    let transactions_csv = csv::ReaderBuilder::new().trim(Trim::All).from_path(&args.transactions_file);

    if let Err(error) = transactions_csv {
        error!("Failed to read transactions file: {}", error);
        return ExitCode::FAILURE;
    }

    payment_system::simulate(transactions_csv.unwrap());

    ExitCode::SUCCESS
}
