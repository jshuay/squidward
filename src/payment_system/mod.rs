mod account;
mod database;
mod error;
mod transaction;
mod types;

use std::path::PathBuf;

use csv::Trim;
use log::debug;
use log::error;

use crate::payment_system::account::Account;
use crate::payment_system::database::BTreeMapDatabase;
use crate::payment_system::database::Database;
use crate::payment_system::error::PaymentSystemError;
use crate::payment_system::transaction::Transaction;
use crate::payment_system::transaction::TransactionType;
use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

pub type Result<T> = std::result::Result<T, PaymentSystemError>;

pub fn simulate(transactions_file: &PathBuf) -> Result<()> {
    debug!("Running payment system simulator");

    debug!("Reading input transactions CSV file: {:?}", transactions_file);

    let transactions_csv = csv::ReaderBuilder::new().trim(Trim::All).from_path(transactions_file);

    if let Err(error) = transactions_csv {
        error!("Failed to read transactions file: {}", error);
        return Err(error)?;
    }
    let mut transactions_csv = transactions_csv.unwrap();

    let mut account_database = BTreeMapDatabase::new();
    let mut transaction_database = BTreeMapDatabase::new();

    debug!("Iterating through each transaction");

    for transaction in transactions_csv.deserialize::<Transaction>() {
        debug!("Transaction: {transaction:?}");

        if let Err(error) = transaction {
            error!("Transaction deserialize error: {}", error);
            continue;
        }

        process_transaction(&mut account_database, &mut transaction_database, transaction.unwrap())?;
    }

    println!("Account database: {:?}", account_database);
    println!("Transaction database: {:?}", transaction_database);

    debug!("Payment system simulation completed");

    Ok(())
}

fn process_transaction<A, T>(
    account_database: &mut A, transaction_database: &mut T, transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing transaction {}", transaction.id());

    if transaction_database.retrieve(transaction.id())?.is_some() {
        error!("Transaction {} has already been processed", transaction.id());
        return Ok(());
    }

    debug!("Retrieving account info for client");

    let account = account_database.retrieve(transaction.client_id())?.unwrap_or_else(|| {
        debug!("Client account does not exist in database");
        Account::new(transaction.client_id().clone())
    });

    match transaction.transaction_type() {
        &TransactionType::Deposit => deposit(account_database, account, transaction.amount())?,
        &TransactionType::Withdrawal => withdraw(account_database, account, transaction.amount())?,
        &TransactionType::Dispute => todo!(),
        &TransactionType::Resolve => todo!(),
        &TransactionType::Chargeback => todo!(),
    }

    transaction_database.insert(transaction.id().clone(), transaction)?;

    Ok(())
}

fn deposit<A>(account_database: &mut A, mut account: Account, amount: Option<&Amount>) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
{
    debug!("Processing deposit transaction");

    if amount.is_none() {
        error!("Transaction did not have an Amount specified");
        return Ok(());
    }

    *account.available_funds_mut() += amount.unwrap();

    account_database.insert(account.client_id().clone(), account)?;

    debug!("Deposit successful");

    Ok(())
}

fn withdraw<A>(account_database: &mut A, mut account: Account, amount: Option<&Amount>) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
{
    debug!("Processing withdraw transaction");

    if amount.is_none() {
        error!("Transaction did not have an Amount specified");
        return Ok(());
    }

    let tentative_amount = account.available_funds() - amount.unwrap();

    if tentative_amount < Amount::ZERO {
        error!("Client does not have sufficient funds to withdraw");
        return Ok(());
    }

    *account.available_funds_mut() = tentative_amount;

    account_database.insert(account.client_id().clone(), account)?;

    debug!("Withdrawal successful");

    Ok(())
}

fn dispute<A, T>(
    account_database: &mut A, transaction_database: &T, mut account: Account, disputed_transaction_id: &TransactionId,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing dispute transaction");

    let Some(disputed_transaction) = transaction_database.retrieve(disputed_transaction_id)? else {
        error!("The disputed transaction does not exist");
        return Ok(());
    };

    if disputed_transaction.transaction_type() != &TransactionType::Deposit {
        error!("Can only dispute deposit transactions");
        return Ok(());
    }

    if disputed_transaction.amount().is_none() {
        error!("Disupted transaction did not have an Amount");
        return Ok(());
    }

    let disputed_amount = disputed_transaction.amount().unwrap();

    *account.available_funds_mut() -= disputed_amount;
    *account.held_funds_mut() += disputed_amount;

    account_database.insert(account.client_id().clone(), account)?;

    debug!("Dispute successful");

    Ok(())
}
