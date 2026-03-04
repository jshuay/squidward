mod account;
mod transaction;

mod types {
    use rust_decimal::Decimal;

    pub type ClientId = u16;

    pub type TransactionId = u32;

    pub type Amount = Decimal;
}

use std::fs::File;

use csv::Reader;
use log::debug;
use log::error;

use crate::payment_system::account::ACCOUNT_HEADERS;
use crate::payment_system::account::Account;
use crate::payment_system::account::Accounts;
use crate::payment_system::transaction::Transaction;
use crate::payment_system::transaction::TransactionType;
use crate::payment_system::transaction::TransactionType::*;
use crate::payment_system::transaction::Transactions;
use crate::payment_system::types::Amount;

/// Main engine for simulating the payment system.
pub fn simulate(mut transactions_csv: Reader<File>) {
    debug!("Running payment system simulator");

    let mut accounts = Accounts::new();
    let mut transactions = Transactions::new();

    debug!("Iterating through each transaction");

    for transaction in transactions_csv.deserialize::<Transaction>() {
        debug!("==========================================");
        debug!("Transaction: {transaction:?}");

        if let Err(error) = transaction {
            error!("Transaction deserialize error: {}", error);
            continue;
        }
        let mut transaction = transaction.unwrap();

        let mut account = accounts.get(&transaction.client_id()).cloned().unwrap_or_else(|| {
            debug!("Client account does not exist in database");
            Account::new(transaction.client_id())
        });
        debug!("Account: {account:?}");

        let existing_transaction = transactions.get(&transaction.id());
        debug!("Existing Transaction: {existing_transaction:?}");

        if apply_transaction(&mut account, &mut transaction, existing_transaction) {
            accounts.insert(account.client_id(), account);
            transactions.insert(transaction.id(), transaction);
            debug!("Successfully applied transaction");
        }
    }

    debug!("==========================================");
    debug!("Account database: {:#?}", accounts);
    debug!("Transaction database: {:#?}", transactions);

    output_accounts_summary(&accounts);

    debug!("Payment system simulation completed");
}

/// Attempts to apply the transaction locally, without comitting to database.
///
/// Returns a boolean that indicates if the transaction was applied or not.
fn apply_transaction(
    account: &mut Account, transaction: &mut Transaction, existing_transaction: Option<&Transaction>,
) -> bool {
    debug!("Attempting to locally apply transaction {}", transaction.id());

    if account.locked() {
        error!("Account is locked. No further transactions for this account will be processed");
        return false;
    }

    if !is_valid_transaction(transaction, &existing_transaction) {
        error!("Received invalid transaction. Ignoring");
        return false;
    }

    let transaction_type = transaction.transaction_type();

    if transaction_type == Deposit {
        return deposit(account, transaction.amount().unwrap());
    }
    if transaction_type == Withdrawal {
        return withdraw(account, transaction.amount().unwrap());
    }

    let disputed_transaction = existing_transaction.unwrap();
    let disputed_transaction_type = disputed_transaction.transaction_type();
    let disputed_amount = disputed_transaction.amount().unwrap();

    match transaction_type {
        Dispute => dispute(account, transaction, disputed_transaction_type, disputed_amount),
        Resolve => resolve(account, transaction, disputed_transaction_type, disputed_amount),
        Chargeback => chargeback(account, transaction, disputed_transaction_type, disputed_amount),
        _ => unreachable!("Deposit and Withdrawal are handled above"),
    }
}

fn is_valid_transaction(transaction: &Transaction, existing_transaction: &Option<&Transaction>) -> bool {
    match transaction.transaction_type() {
        Deposit | Withdrawal => {
            if existing_transaction.is_some() {
                error!("Transaction {} has already been processed", transaction.id());
                return false;
            }
            if transaction.amount().is_none() {
                error!("Transaction did not have an Amount specified");
                return false;
            }
        },
        Dispute | Resolve | Chargeback => {
            if existing_transaction.is_none() {
                error!("The disputed transaction does not exist");
                return false;
            }

            let existing_transaction = existing_transaction.unwrap();

            if existing_transaction.client_id() != transaction.client_id() {
                error!("Requesting ClientId does not match disputed transaction's ClientId");
                return false;
            }
            if existing_transaction.amount().is_none() {
                error!("Transaction did not have an Amount specified");
                return false;
            }
        },
    }
    true
}

fn deposit(account: &mut Account, amount: &Amount) -> bool {
    debug!("Applying deposit transaction");

    *account.available_funds_mut() += amount;

    true
}

fn withdraw(account: &mut Account, amount: &Amount) -> bool {
    debug!("Applying withdraw transaction");

    let tentative_amount = account.available_funds() - amount;

    if tentative_amount < Amount::ZERO {
        error!("Client does not have sufficient funds to withdraw");
        return false;
    }

    *account.available_funds_mut() = tentative_amount;

    true
}

fn dispute(
    account: &mut Account, transaction: &mut Transaction, disputed_transaction_type: TransactionType,
    disputed_amount: &Amount,
) -> bool {
    debug!("Applying dispute transaction");

    if disputed_transaction_type != Deposit && disputed_transaction_type != Resolve {
        error!("Can only dispute deposit transactions or previously resolved ones");
        return false;
    }

    *account.available_funds_mut() -= disputed_amount;
    *account.held_funds_mut() += disputed_amount;

    *transaction.transaction_type_mut() = Dispute;
    *transaction.amount_mut() = Some(disputed_amount.clone());

    true
}

fn resolve(
    account: &mut Account, transaction: &mut Transaction, disputed_transaction_type: TransactionType,
    disputed_amount: &Amount,
) -> bool {
    debug!("Applying resolve transaction");

    if disputed_transaction_type != Dispute {
        error!("Can only resolve disputed transactions");
        return false;
    }

    *account.available_funds_mut() += disputed_amount;
    *account.held_funds_mut() -= disputed_amount;

    *transaction.transaction_type_mut() = Resolve;
    *transaction.amount_mut() = Some(disputed_amount.clone());

    true
}

fn chargeback(
    account: &mut Account, transaction: &mut Transaction, disputed_transaction_type: TransactionType,
    disputed_amount: &Amount,
) -> bool {
    debug!("Applying chargeback transaction");

    if disputed_transaction_type != Dispute {
        error!("Can only chargeback disputed transactions");
        return false;
    }

    *account.held_funds_mut() -= disputed_amount;
    account.lock();

    *transaction.transaction_type_mut() = Chargeback;
    *transaction.amount_mut() = Some(disputed_amount.clone());

    true
}

fn output_accounts_summary(accounts: &Accounts) {
    println!("{ACCOUNT_HEADERS}");
    for account in accounts.values() {
        println!("{account}");
    }
}
