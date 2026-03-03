use thiserror::Error;

use crate::payment_system::database::DatabaseError;

#[derive(Error, Debug)]
pub enum PaymentSystemError {
    #[error("Failed to parse CSV transactions file")]
    CsvParseFailure(#[from] csv::Error),
    #[error("Failed to parse CSV transactions file")]
    DatabaseError(#[from] DatabaseError),
}
