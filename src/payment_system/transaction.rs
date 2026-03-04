use serde::Deserialize;

use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    #[serde(rename = "tx")]
    id: TransactionId,

    #[serde(rename = "type")]
    transaction_type: TransactionType,

    #[serde(rename = "client")]
    client_id: ClientId,

    amount: Option<Amount>,
}

impl Transaction {
    pub fn id(&self) -> &TransactionId {
        &self.id
    }
    pub fn transaction_type(&self) -> &TransactionType {
        &self.transaction_type
    }
    pub fn transaction_type_mut(&mut self) -> &mut TransactionType {
        &mut self.transaction_type
    }
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }
    pub fn amount(&self) -> Option<&Amount> {
        self.amount.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
