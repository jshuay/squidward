use serde::Deserialize;

use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "tx")]
    id: TransactionId,

    #[serde(rename = "type")]
    action: TransactionAction,

    #[serde(rename = "client")]
    client_id: ClientId,

    amount: Option<Amount>,
}

impl Transaction {
    pub fn id(&self) -> &TransactionId {
        &self.id
    }
    pub fn action(&self) -> &TransactionAction {
        &self.action
    }
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }
    pub fn amount(&self) -> Option<&Amount> {
        self.amount.as_ref()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TransactionAction {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
