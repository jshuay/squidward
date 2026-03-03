use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

pub struct Deposit {
    client_id: ClientId,
    transaction_id: TransactionId,
    amount: Amount,
}

impl Deposit {
    pub fn new(client_id: ClientId, transaction_id: TransactionId, amount: Amount) -> Self {
        Deposit {
            client_id,
            transaction_id,
            amount,
        }
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    pub fn amount(&self) -> &Amount {
        &self.amount
    }
}
