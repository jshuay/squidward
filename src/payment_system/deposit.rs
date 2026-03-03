use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

pub struct Deposit {
    client_id: ClientId,
    transaction_id: TransactionId,
    amount: Amount,
}
