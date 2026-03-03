mod error;

use std::path::PathBuf;

use log::debug;
use log::error;

use crate::payment_system::error::PaymentSystemError;

pub type Result<T> = std::result::Result<T, PaymentSystemError>;

pub fn simulate(transactions_file: &PathBuf) -> Result<()> {
    debug!("Running payment system simulator");

    let transactions_csv = csv::Reader::from_path(transactions_file);

    if let Err(error) = transactions_csv {
        error!("Failed to read transactions file: {}", error);
        return Err(error)?;
    }

    Ok(())
}
