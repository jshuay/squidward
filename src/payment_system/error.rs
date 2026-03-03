use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentSystemError {
    #[error("Failed to parse CSV transactions file")]
    CsvParseFailure(#[from] csv::Error),
}
