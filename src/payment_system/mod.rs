mod account;
mod transaction;
mod types;

use std::fs::File;

use csv::Reader;
use log::debug;
use log::error;

use crate::payment_system::account::Account;
use crate::payment_system::account::Accounts;
use crate::payment_system::transaction::Transaction;
use crate::payment_system::transaction::TransactionType;
use crate::payment_system::transaction::Transactions;
use crate::payment_system::types::Amount;

pub fn simulate(mut transactions_csv: Reader<File>) {
    debug!("Running payment system simulator");

    let mut accounts = Accounts::new();
    let mut transactions = Transactions::new();

    debug!("Iterating through each transaction");

    for transaction in transactions_csv.deserialize::<Transaction>() {
        debug!("Transaction: {transaction:?}");

        if let Err(error) = transaction {
            error!("Transaction deserialize error: {}", error);
            continue;
        }
        let transaction = transaction.unwrap();

        process_transaction(&mut accounts, &mut transactions, transaction);
    }

    debug!("Account database: {:#?}", accounts);
    debug!("Transaction database: {:#?}", transactions);

    println!("client,available,held,total,locked");
    for account in accounts.values() {
        println!("{account}");
    }

    debug!("Payment system simulation completed");
}

fn process_transaction(accounts: &mut Accounts, transactions: &mut Transactions, transaction: Transaction) {
    debug!("Processing transaction {}", transaction.id());

    debug!("Retrieving account info for client");

    let account = accounts.get(&transaction.client_id()).cloned().unwrap_or_else(|| {
        debug!("Client account does not exist in database");
        Account::new(transaction.client_id())
    });

    if account.locked() {
        error!("Account is locked. No further transactions for this account will be processed");
        return;
    }

    match transaction.transaction_type() {
        &TransactionType::Deposit => deposit(accounts, transactions, account, transaction),
        &TransactionType::Withdrawal => withdraw(accounts, transactions, account, transaction),
        &TransactionType::Dispute => dispute(accounts, transactions, account, transaction),
        &TransactionType::Resolve => resolve(accounts, transactions, account, transaction),
        &TransactionType::Chargeback => chargeback(accounts, transactions, account, transaction),
    }
}

fn deposit(accounts: &mut Accounts, transactions: &mut Transactions, mut account: Account, transaction: Transaction) {
    debug!("Processing deposit transaction");

    if transaction.amount().is_none() {
        error!("Transaction did not have an Amount specified");
        return;
    }

    if transactions.get(&transaction.id()).is_some() {
        error!("Transaction {} has already been processed", transaction.id());
        return;
    }

    *account.available_funds_mut() += transaction.amount().unwrap();

    accounts.insert(account.client_id(), account);
    transactions.insert(transaction.id(), transaction);

    debug!("Deposit successful");
}

fn withdraw(accounts: &mut Accounts, transactions: &mut Transactions, mut account: Account, transaction: Transaction) {
    debug!("Processing withdraw transaction");

    if transaction.amount().is_none() {
        error!("Transaction did not have an Amount specified");
        return;
    }

    if transactions.get(&transaction.id()).is_some() {
        error!("Transaction {} has already been processed", transaction.id());
        return;
    }

    let tentative_amount = account.available_funds() - transaction.amount().unwrap();

    if tentative_amount < Amount::ZERO {
        error!("Client does not have sufficient funds to withdraw");
        return;
    }

    *account.available_funds_mut() = tentative_amount;

    accounts.insert(account.client_id(), account);
    transactions.insert(transaction.id(), transaction);

    debug!("Withdrawal successful");
}

fn dispute(accounts: &mut Accounts, transactions: &mut Transactions, mut account: Account, transaction: Transaction) {
    debug!("Processing dispute transaction");

    let Some(mut disputed_transaction) = transactions.get(&transaction.id()).cloned() else {
        error!("The disputed transaction does not exist");
        return;
    };

    if disputed_transaction.transaction_type() != &TransactionType::Deposit
        && disputed_transaction.transaction_type() != &TransactionType::Resolve
    {
        error!("Can only dispute deposit transactions or previously resolved ones");
        return;
    }

    if disputed_transaction.client_id() != transaction.client_id() {
        error!("Requesting ClientId does not match disputed transaction's ClientId");
        return;
    }

    if disputed_transaction.amount().is_none() {
        error!("Disupted transaction did not have an Amount");
        return;
    }

    let disputed_amount = disputed_transaction.amount().unwrap();

    *account.available_funds_mut() -= disputed_amount;
    *account.held_funds_mut() += disputed_amount;

    accounts.insert(account.client_id(), account);

    *disputed_transaction.transaction_type_mut() = TransactionType::Dispute;
    transactions.insert(disputed_transaction.id(), disputed_transaction);

    debug!("Dispute successful");
}

fn resolve(accounts: &mut Accounts, transactions: &mut Transactions, mut account: Account, transaction: Transaction) {
    debug!("Processing resolve transaction");

    let Some(mut disputed_transaction) = transactions.get(&transaction.id()).cloned() else {
        error!("The disputed transaction does not exist");
        return;
    };

    if disputed_transaction.transaction_type() != &TransactionType::Dispute {
        error!("Can only resolve disputed transactions");
        return;
    }

    if disputed_transaction.client_id() != transaction.client_id() {
        error!("Requesting ClientId does not match disputed transaction's ClientId");
        return;
    }

    if disputed_transaction.amount().is_none() {
        error!("Disupted transaction did not have an Amount");
        return;
    }

    let disputed_amount = disputed_transaction.amount().unwrap();

    *account.available_funds_mut() += disputed_amount;
    *account.held_funds_mut() -= disputed_amount;

    accounts.insert(account.client_id(), account);

    *disputed_transaction.transaction_type_mut() = TransactionType::Resolve;
    transactions.insert(disputed_transaction.id(), disputed_transaction);

    debug!("Resolve successful");
}

fn chargeback(
    accounts: &mut Accounts, transactions: &mut Transactions, mut account: Account, transaction: Transaction,
) {
    debug!("Processing chargeback transaction");

    let Some(mut disputed_transaction) = transactions.get(&transaction.id()).cloned() else {
        error!("The disputed transaction does not exist");
        return;
    };

    if disputed_transaction.transaction_type() != &TransactionType::Dispute {
        error!("Can only chargeback disputed transactions");
        return;
    }

    if disputed_transaction.client_id() != transaction.client_id() {
        error!("Requesting ClientId does not match disputed transaction's ClientId");
        return;
    }

    if disputed_transaction.amount().is_none() {
        error!("Disupted transaction did not have an Amount");
        return;
    }

    let disputed_amount = disputed_transaction.amount().unwrap();

    *account.held_funds_mut() -= disputed_amount;
    account.lock();

    accounts.insert(account.client_id(), account);

    *disputed_transaction.transaction_type_mut() = TransactionType::Chargeback;
    transactions.insert(disputed_transaction.id(), disputed_transaction);

    debug!("Chargeback successful");
}
