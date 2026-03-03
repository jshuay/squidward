mod error;
mod transaction;

use std::path::PathBuf;

use log::debug;
use log::error;

use crate::payment_system::error::PaymentSystemError;
use crate::payment_system::transaction::Transaction;

pub type Result<T> = std::result::Result<T, PaymentSystemError>;

pub fn simulate(transactions_file: &PathBuf) -> Result<()> {
    debug!("Running payment system simulator");

    let transactions_csv = csv::Reader::from_path(transactions_file);

    if let Err(error) = transactions_csv {
        error!("Failed to read transactions file: {}", error);
        return Err(error)?;
    }

    let mut transactions_csv = transactions_csv.unwrap();

    for transaction in transactions_csv.deserialize::<Transaction>() {
        debug!("transaction: {transaction:?}");
    }

    Ok(())
}
