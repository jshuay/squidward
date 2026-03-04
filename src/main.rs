use std::process::ExitCode;

use clap::Parser;
use log::LevelFilter;
use log::debug;
use log::error;

/// Squidward payment system simulator
#[derive(Parser, Debug)]
struct Args {
    /// Path to the transactions file (CSV format)
    transactions_file: String,
    /// Output debug logs to STDOUT
    #[arg(short, long)]
    debug: bool,
}

/// Program entry point.
///
/// This main function's primary job is to setup the CLI component as well as validating the input.
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

    let transactions_csv = squidward::load_transactions_csv(&args.transactions_file);
    if let Err(error) = transactions_csv {
        error!("Failed to read transactions file: {}", error);
        return ExitCode::FAILURE;
    }

    squidward::payment_system::simulate(transactions_csv.unwrap());

    ExitCode::SUCCESS
}
