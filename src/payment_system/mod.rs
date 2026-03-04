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

    let mut account_database: BTreeMapDatabase<ClientId, Account> = BTreeMapDatabase::new();
    let mut transaction_database: BTreeMapDatabase<TransactionId, Transaction> = BTreeMapDatabase::new();

    debug!("Iterating through each transaction");

    for transaction in transactions_csv.deserialize::<Transaction>() {
        debug!("Transaction: {transaction:?}");

        if let Err(error) = transaction {
            error!("Transaction deserialize error: {}", error);
            continue;
        }
        let transaction = transaction.unwrap();

        process_transaction(&mut account_database, &mut transaction_database, transaction)?;
    }

    debug!("Account database: {:#?}", account_database);
    debug!("Transaction database: {:#?}", transaction_database);

    debug!("Payment system simulation completed");

    Ok(())
}

fn process_transaction<A, T>(
    account_database: &mut A, transaction_database: &mut T, mut transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing transaction {}", transaction.id());

    debug!("Retrieving account info for client");

    let account = account_database.retrieve(transaction.client_id())?.unwrap_or_else(|| {
        debug!("Client account does not exist in database");
        Account::new(transaction.client_id().clone())
    });

    if account.locked() {
        error!("Account is locked. No further transactions for this account will be processed");
        return Ok(());
    }

    match transaction.transaction_type() {
        &TransactionType::Deposit => deposit(account_database, transaction_database, account, transaction)?,
        &TransactionType::Withdrawal => withdraw(account_database, transaction_database, account, transaction)?,
        &TransactionType::Dispute => dispute(account_database, transaction_database, account, transaction)?,
        &TransactionType::Resolve => resolve(account_database, transaction_database, account, transaction)?,
        &TransactionType::Chargeback => chargeback(account_database, transaction_database, account, transaction)?,
    }

    Ok(())
}

fn deposit<A, T>(
    account_database: &mut A, transaction_database: &mut T, mut account: Account, mut transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing deposit transaction");

    if transaction.amount().is_none() {
        error!("Transaction did not have an Amount specified");
        return Ok(());
    }

    if transaction_database.retrieve(transaction.id())?.is_some() {
        error!("Transaction {} has already been processed", transaction.id());
        return Ok(());
    }

    *account.available_funds_mut() += transaction.amount().unwrap();

    account_database.insert(account.client_id().clone(), account)?;
    transaction_database.insert(transaction.id().clone(), transaction)?;

    debug!("Deposit successful");

    Ok(())
}

fn withdraw<A, T>(
    account_database: &mut A, transaction_database: &mut T, mut account: Account, mut transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing withdraw transaction");

    if transaction.amount().is_none() {
        error!("Transaction did not have an Amount specified");
        return Ok(());
    }

    if transaction_database.retrieve(transaction.id())?.is_some() {
        error!("Transaction {} has already been processed", transaction.id());
        return Ok(());
    }

    let tentative_amount = account.available_funds() - transaction.amount().unwrap();

    if tentative_amount < Amount::ZERO {
        error!("Client does not have sufficient funds to withdraw");
        return Ok(());
    }

    *account.available_funds_mut() = tentative_amount;

    account_database.insert(account.client_id().clone(), account)?;
    transaction_database.insert(transaction.id().clone(), transaction)?;

    debug!("Withdrawal successful");

    Ok(())
}

fn dispute<A, T>(
    account_database: &mut A, transaction_database: &mut T, mut account: Account, transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing dispute transaction");

    let Some(mut disputed_transaction) = transaction_database.retrieve(transaction.id())? else {
        error!("The disputed transaction does not exist");
        return Ok(());
    };

    if disputed_transaction.transaction_type() != &TransactionType::Deposit
        && disputed_transaction.transaction_type() != &TransactionType::Resolve
    {
        error!("Can only dispute deposit transactions or previously resolved ones");
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

    *disputed_transaction.transaction_type_mut() = TransactionType::Dispute;
    transaction_database.insert(disputed_transaction.id().clone(), disputed_transaction)?;

    debug!("Dispute successful");

    Ok(())
}

fn resolve<A, T>(
    account_database: &mut A, transaction_database: &mut T, mut account: Account, transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing resolve transaction");

    let Some(mut disputed_transaction) = transaction_database.retrieve(transaction.id())? else {
        error!("The disputed transaction does not exist");
        return Ok(());
    };

    if disputed_transaction.transaction_type() != &TransactionType::Dispute {
        error!("Can only resolve disputed transactions");
        return Ok(());
    }

    if disputed_transaction.amount().is_none() {
        error!("Disupted transaction did not have an Amount");
        return Ok(());
    }

    let disputed_amount = disputed_transaction.amount().unwrap();

    *account.available_funds_mut() += disputed_amount;
    *account.held_funds_mut() -= disputed_amount;

    account_database.insert(account.client_id().clone(), account)?;

    *disputed_transaction.transaction_type_mut() = TransactionType::Resolve;
    transaction_database.insert(disputed_transaction.id().clone(), disputed_transaction)?;

    debug!("Resolve successful");

    Ok(())
}

fn chargeback<A, T>(
    account_database: &mut A, transaction_database: &mut T, mut account: Account, transaction: Transaction,
) -> Result<()>
where
    A: Database<Key = ClientId, Record = Account>,
    T: Database<Key = TransactionId, Record = Transaction>,
{
    debug!("Processing chargeback transaction");

    let Some(mut disputed_transaction) = transaction_database.retrieve(transaction.id())? else {
        error!("The disputed transaction does not exist");
        return Ok(());
    };

    if disputed_transaction.transaction_type() != &TransactionType::Dispute {
        error!("Can only chargeback disputed transactions");
        return Ok(());
    }

    if disputed_transaction.amount().is_none() {
        error!("Disupted transaction did not have an Amount");
        return Ok(());
    }

    let disputed_amount = disputed_transaction.amount().unwrap();

    *account.held_funds_mut() -= disputed_amount;
    account.lock();

    account_database.insert(account.client_id().clone(), account)?;

    *disputed_transaction.transaction_type_mut() = TransactionType::Chargeback;
    transaction_database.insert(disputed_transaction.id().clone(), disputed_transaction)?;

    debug!("Chargeback successful");

    Ok(())
}
