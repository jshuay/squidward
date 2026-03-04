pub mod payment_system;

use std::fs::File;

use csv::Reader;
use csv::Trim;

/// Attempt to create a CSV Reader from the given input path.
pub fn load_transactions_csv(transactions_file: &str) -> csv::Result<Reader<File>> {
    csv::ReaderBuilder::new().trim(Trim::All).from_path(transactions_file)
}

#[cfg(test)]
mod tests {
    use crate::load_transactions_csv;

    #[test]
    fn load_transactions_csv_with_valid_file_path() {
        assert!(load_transactions_csv("test_files/valid.csv").is_ok());
    }

    #[test]
    fn load_transactions_csv_with_invalid_file_path_returns_error() {
        assert!(load_transactions_csv("test_files/invalid_file_path.csv").is_err());
    }
}
