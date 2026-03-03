use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

pub trait AccountDatabase {
    fn deposit(&mut self, client_id: &ClientId, amount: &Amount) -> Result<(), DatabaseError>;
    fn withdrawal(&mut self, client_id: &ClientId, amount: &Amount) -> Result<(), DatabaseError>;
    fn hold_funds(&mut self, client_id: &ClientId, amount: &Amount) -> Result<(), DatabaseError>;
    fn lock_account(&mut self, client_id: &ClientId) -> Result<(), DatabaseError>;
}

pub trait DepositDatabase {
    fn insert(
        &mut self, client_id: &ClientId, transaction_id: &TransactionId, amount: &Amount,
    ) -> Result<(), DatabaseError>;

    fn retrieve(&self, client_id: &ClientId, transaction_id: &TransactionId) -> Result<Option<Amount>, DatabaseError>;
}

pub enum DatabaseError {}
