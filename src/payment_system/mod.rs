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

    *transaction.amount_mut() = Some(disputed_amount.clone());

    true
}

fn output_accounts_summary(accounts: &Accounts) {
    println!("{ACCOUNT_HEADERS}");
    for account in accounts.values() {
        println!("{account}");
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::payment_system::account::Account;
    use crate::payment_system::apply_transaction;
    use crate::payment_system::chargeback;
    use crate::payment_system::deposit;
    use crate::payment_system::dispute;
    use crate::payment_system::is_valid_transaction;
    use crate::payment_system::resolve;
    use crate::payment_system::transaction::Transaction;
    use crate::payment_system::transaction::TransactionType;
    use crate::payment_system::types::Amount;
    use crate::payment_system::withdraw;

    #[test]
    fn apply_transaction_on_locked_accounts_does_not_apply() {
        let client_id = 0;
        let mut account = Account::new(client_id);
        account.lock();

        let mut transaction = Transaction::new(0, TransactionType::Deposit, client_id, Some(dec!(1.0)));

        assert_eq!(&Amount::ZERO, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(Amount::ZERO, account.total_funds());
        assert!(account.locked());

        assert!(!apply_transaction(&mut account, &mut transaction, None));

        assert_eq!(&Amount::ZERO, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(Amount::ZERO, account.total_funds());
        assert!(account.locked());
    }

    #[test]
    fn is_valid_transaction_for_valid_transactions_returns_true() {
        let client_id = 0;
        let deposit_transaction_id = 0;

        let deposit_transaction = Transaction::new(
            deposit_transaction_id,
            TransactionType::Deposit,
            client_id,
            Some(dec!(1.0)),
        );
        let withdrawal_transaction = Transaction::new(1, TransactionType::Withdrawal, client_id, Some(dec!(1.0)));
        let dispute_transaction = Transaction::new(deposit_transaction_id, TransactionType::Dispute, client_id, None);
        let resolve_transaction = Transaction::new(deposit_transaction_id, TransactionType::Resolve, client_id, None);
        let chargeback_transaction =
            Transaction::new(deposit_transaction_id, TransactionType::Chargeback, client_id, None);

        assert!(is_valid_transaction(&deposit_transaction, &None));
        assert!(is_valid_transaction(&withdrawal_transaction, &None));
        assert!(is_valid_transaction(&dispute_transaction, &Some(&deposit_transaction)));
        assert!(is_valid_transaction(&resolve_transaction, &Some(&deposit_transaction)));
        assert!(is_valid_transaction(
            &chargeback_transaction,
            &Some(&deposit_transaction)
        ));
    }

    #[test]
    fn is_valid_transaction_for_deposit_and_withdrawal_with_existing_transaction_returns_false() {
        let deposit_transaction = Transaction::new(0, TransactionType::Deposit, 0, Some(dec!(1.0)));
        let withdrawal_transaction = Transaction::new(0, TransactionType::Withdrawal, 0, Some(dec!(1.0)));

        assert!(!is_valid_transaction(&deposit_transaction, &Some(&deposit_transaction)));
        assert!(!is_valid_transaction(
            &withdrawal_transaction,
            &Some(&withdrawal_transaction)
        ));
    }

    #[test]
    fn is_valid_transaction_for_deposit_and_withdrawal_with_no_amount_returns_false() {
        let deposit_transaction = Transaction::new(0, TransactionType::Deposit, 0, None);
        let withdrawal_transaction = Transaction::new(0, TransactionType::Withdrawal, 0, None);

        assert!(!is_valid_transaction(&deposit_transaction, &None));
        assert!(!is_valid_transaction(&withdrawal_transaction, &None));
    }

    #[test]
    fn is_valid_transaction_for_dispute_resolve_and_chargeback_with_no_existing_transaction_returns_false() {
        let dispute_transaction = Transaction::new(0, TransactionType::Dispute, 0, None);
        let resolve_transaction = Transaction::new(0, TransactionType::Resolve, 0, None);
        let chargeback_transaction = Transaction::new(0, TransactionType::Chargeback, 0, None);

        assert!(!is_valid_transaction(&dispute_transaction, &None));
        assert!(!is_valid_transaction(&resolve_transaction, &None));
        assert!(!is_valid_transaction(&chargeback_transaction, &None));
    }

    #[test]
    fn is_valid_transaction_for_dispute_resolve_and_chargeback_with_mismatching_client_id_returns_false() {
        let transaction_id = 0;
        let client_id = 0;
        let mismatch_client_id = 1;

        let existing_transaction =
            Transaction::new(transaction_id, TransactionType::Deposit, client_id, Some(dec!(1.0)));

        let dispute_transaction = Transaction::new(transaction_id, TransactionType::Dispute, mismatch_client_id, None);
        let resolve_transaction = Transaction::new(transaction_id, TransactionType::Resolve, mismatch_client_id, None);
        let chargeback_transaction =
            Transaction::new(transaction_id, TransactionType::Chargeback, mismatch_client_id, None);

        assert!(!is_valid_transaction(
            &dispute_transaction,
            &Some(&existing_transaction)
        ));
        assert!(!is_valid_transaction(
            &resolve_transaction,
            &Some(&existing_transaction)
        ));
        assert!(!is_valid_transaction(
            &chargeback_transaction,
            &Some(&existing_transaction)
        ));
    }

    #[test]
    fn is_valid_transaction_for_dispute_resolve_and_chargeback_with_no_amount_on_existing_transaction_returns_false() {
        let transaction_id = 0;
        let client_id = 0;

        let existing_transaction = Transaction::new(transaction_id, TransactionType::Deposit, client_id, None);

        let dispute_transaction = Transaction::new(transaction_id, TransactionType::Dispute, client_id, None);
        let resolve_transaction = Transaction::new(transaction_id, TransactionType::Resolve, client_id, None);
        let chargeback_transaction = Transaction::new(transaction_id, TransactionType::Chargeback, client_id, None);

        assert!(!is_valid_transaction(
            &dispute_transaction,
            &Some(&existing_transaction)
        ));
        assert!(!is_valid_transaction(&resolve_transaction, &Some(&dispute_transaction)));
        assert!(!is_valid_transaction(
            &chargeback_transaction,
            &Some(&dispute_transaction)
        ));
    }

    #[test]
    fn deposit_only_increases_accounts_available_funds() {
        let mut account = Account::new(0);
        let deposit_amount = dec!(5.123);

        assert_eq!(&Amount::ZERO, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(Amount::ZERO, account.total_funds());

        assert!(deposit(&mut account, &deposit_amount));

        assert_eq!(&deposit_amount, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(deposit_amount, account.total_funds());
    }

    #[test]
    fn withdraw_only_decreases_accounts_available_funds() {
        let starting_amount = dec!(10);

        let mut account = Account::new(0);
        *account.available_funds_mut() = starting_amount;
        let withdrawal_amount = dec!(5.123);

        assert_eq!(&starting_amount, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(starting_amount, account.total_funds());

        assert!(withdraw(&mut account, &withdrawal_amount));

        let expected_leftover_amount = starting_amount - withdrawal_amount;

        assert_eq!(&expected_leftover_amount, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(expected_leftover_amount, account.total_funds());
    }

    #[test]
    fn withdraw_when_amount_exceeds_accounts_available_funds_fails_to_apply() {
        let mut account = Account::new(0);
        let withdrawal_amount = dec!(10);

        assert_eq!(&Amount::ZERO, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(Amount::ZERO, account.total_funds());

        assert!(!withdraw(&mut account, &withdrawal_amount));

        assert_eq!(&Amount::ZERO, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(Amount::ZERO, account.total_funds());
    }

    #[test]
    fn dispute_on_non_deposit_or_resolve_transaction_fails_to_apply() {
        let client_id = 0;
        let disputed_transaction_id = 0;

        let mut account = Account::new(client_id);

        let mut dispute_transaction =
            Transaction::new(disputed_transaction_id, TransactionType::Dispute, client_id, None);

        assert!(!dispute(
            &mut account,
            &mut dispute_transaction,
            TransactionType::Withdrawal,
            &dec!(1.0)
        ));
        assert!(!dispute(
            &mut account,
            &mut dispute_transaction,
            TransactionType::Dispute,
            &dec!(1.0)
        ));
        assert!(!dispute(
            &mut account,
            &mut dispute_transaction,
            TransactionType::Chargeback,
            &dec!(1.0)
        ));
    }

    #[test]
    fn dispute_with_valid_disputed_transaction_transfers_funds_from_available_to_held() {
        let client_id = 0;
        let starting_available_funds = dec!(10);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;

        let mut dispute_transaction = Transaction::new(0, TransactionType::Dispute, client_id, None);

        assert_eq!(&starting_available_funds, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(starting_available_funds, account.total_funds());

        assert!(dispute(
            &mut account,
            &mut dispute_transaction,
            TransactionType::Resolve,
            &disputed_amount
        ));

        assert_eq!(&(starting_available_funds - disputed_amount), account.available_funds());
        assert_eq!(&disputed_amount, account.held_funds());
        assert_eq!(starting_available_funds, account.total_funds());
    }

    #[test]
    fn dispute_with_valid_disputed_transaction_updates_transaction_with_disputed_amount() {
        let client_id = 0;
        let starting_available_funds = dec!(10);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;

        let mut dispute_transaction = Transaction::new(0, TransactionType::Dispute, client_id, None);

        assert!(dispute_transaction.amount().is_none());

        assert!(dispute(
            &mut account,
            &mut dispute_transaction,
            TransactionType::Deposit,
            &disputed_amount
        ));

        assert!(dispute_transaction.amount().is_some());
        assert_eq!(&disputed_amount, dispute_transaction.amount().unwrap());
    }

    #[test]
    fn resolve_on_non_dispute_transaction_fails_to_apply() {
        let client_id = 0;
        let disputed_transaction_id = 0;

        let mut account = Account::new(client_id);

        let mut resolve_transaction =
            Transaction::new(disputed_transaction_id, TransactionType::Resolve, client_id, None);

        assert!(!resolve(
            &mut account,
            &mut resolve_transaction,
            TransactionType::Deposit,
            &dec!(1.0)
        ));
        assert!(!resolve(
            &mut account,
            &mut resolve_transaction,
            TransactionType::Withdrawal,
            &dec!(1.0)
        ));
        assert!(!resolve(
            &mut account,
            &mut resolve_transaction,
            TransactionType::Resolve,
            &dec!(1.0)
        ));
        assert!(!resolve(
            &mut account,
            &mut resolve_transaction,
            TransactionType::Chargeback,
            &dec!(1.0)
        ));
    }

    #[test]
    fn resolve_with_valid_disputed_transaction_transfers_funds_from_held_to_available() {
        let client_id = 0;
        let starting_available_funds = dec!(6);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;
        *account.held_funds_mut() = disputed_amount;

        let mut resolve_transaction = Transaction::new(0, TransactionType::Resolve, client_id, None);

        assert_eq!(&starting_available_funds, account.available_funds());
        assert_eq!(&disputed_amount, account.held_funds());
        assert_eq!(starting_available_funds + disputed_amount, account.total_funds());

        assert!(resolve(
            &mut account,
            &mut resolve_transaction,
            TransactionType::Dispute,
            &disputed_amount
        ));

        assert_eq!(&(starting_available_funds + disputed_amount), account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(starting_available_funds + disputed_amount, account.total_funds());
    }

    #[test]
    fn resolve_with_valid_disputed_transaction_updates_transaction_with_disputed_amount() {
        let client_id = 0;
        let starting_available_funds = dec!(6);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;
        *account.held_funds_mut() = disputed_amount;

        let mut resolve_transaction = Transaction::new(0, TransactionType::Resolve, client_id, None);

        assert!(resolve_transaction.amount().is_none());

        assert!(resolve(
            &mut account,
            &mut resolve_transaction,
            TransactionType::Dispute,
            &disputed_amount
        ));

        assert!(resolve_transaction.amount().is_some());
        assert_eq!(&disputed_amount, resolve_transaction.amount().unwrap());
    }

    #[test]
    fn chargeback_on_non_dispute_transaction_fails_to_apply() {
        let client_id = 0;
        let disputed_transaction_id = 0;

        let mut account = Account::new(client_id);

        let mut chargeback_transaction =
            Transaction::new(disputed_transaction_id, TransactionType::Chargeback, client_id, None);

        assert!(!chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Deposit,
            &dec!(1.0)
        ));
        assert!(!chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Withdrawal,
            &dec!(1.0)
        ));
        assert!(!chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Resolve,
            &dec!(1.0)
        ));
        assert!(!chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Chargeback,
            &dec!(1.0)
        ));
    }

    #[test]
    fn chargeback_with_valid_disputed_transaction_reduces_total_funds_by_disputed_amount() {
        let client_id = 0;
        let starting_available_funds = dec!(6);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;
        *account.held_funds_mut() = disputed_amount;

        let mut chargeback_transaction = Transaction::new(0, TransactionType::Chargeback, client_id, None);

        assert_eq!(&starting_available_funds, account.available_funds());
        assert_eq!(&disputed_amount, account.held_funds());
        assert_eq!(starting_available_funds + disputed_amount, account.total_funds());

        assert!(chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Dispute,
            &disputed_amount
        ));

        assert_eq!(&starting_available_funds, account.available_funds());
        assert_eq!(&Amount::ZERO, account.held_funds());
        assert_eq!(starting_available_funds, account.total_funds());
    }

    #[test]
    fn chargeback_with_valid_disputed_transaction_updates_transaction_with_disputed_amount() {
        let client_id = 0;
        let starting_available_funds = dec!(6);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;
        *account.held_funds_mut() = disputed_amount;

        let mut chargeback_transaction = Transaction::new(0, TransactionType::Chargeback, client_id, None);

        assert!(chargeback_transaction.amount().is_none());

        assert!(chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Dispute,
            &disputed_amount
        ));

        assert!(chargeback_transaction.amount().is_some());
        assert_eq!(&disputed_amount, chargeback_transaction.amount().unwrap());
    }

    #[test]
    fn chargeback_with_valid_disputed_transaction_applied_locks_account() {
        let client_id = 0;
        let starting_available_funds = dec!(6);
        let disputed_amount = dec!(4);

        let mut account = Account::new(client_id);
        *account.available_funds_mut() = starting_available_funds;
        *account.held_funds_mut() = disputed_amount;

        let mut chargeback_transaction = Transaction::new(0, TransactionType::Chargeback, client_id, None);

        assert!(!account.locked());

        assert!(chargeback(
            &mut account,
            &mut chargeback_transaction,
            TransactionType::Dispute,
            &disputed_amount
        ));

        assert!(account.locked());
    }
}
