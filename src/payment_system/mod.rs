mod database;
mod error;
mod transaction;
mod types;

use std::path::PathBuf;

use csv::Trim;
use log::debug;
use log::error;

use crate::payment_system::transaction::Transaction;
use crate::payment_system::types::Result;

pub fn simulate(transactions_file: &PathBuf) -> Result<()> {
    debug!("Running payment system simulator");

    let transactions_csv = csv::ReaderBuilder::new().trim(Trim::All).from_path(transactions_file);

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
