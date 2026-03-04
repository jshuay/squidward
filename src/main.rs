mod payment_system;

use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use csv::Reader;
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

    let transactions_csv = load_transactions_csv(&args.transactions_file);
    if let Err(error) = transactions_csv {
        error!("Failed to read transactions file: {}", error);
        return ExitCode::FAILURE;
    }

    payment_system::simulate(transactions_csv.unwrap());

    ExitCode::SUCCESS
}

fn load_transactions_csv(transactions_file: &PathBuf) -> csv::Result<Reader<File>> {
    csv::ReaderBuilder::new().trim(Trim::All).from_path(transactions_file)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::load_transactions_csv;

    #[test]
    fn load_transactions_csv_with_valid_file_path() {
        let valid_csv_path = PathBuf::from("test_files/valid.csv");

        assert!(load_transactions_csv(&valid_csv_path).is_ok());
    }

    #[test]
    fn load_transactions_csv_with_invalid_file_path_returns_error() {
        let valid_csv_path = PathBuf::from("test_files/invalid_file_path.csv");

        assert!(load_transactions_csv(&valid_csv_path).is_err());
    }
}
