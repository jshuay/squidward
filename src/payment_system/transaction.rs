use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "tx")]
    id: u32,
    #[serde(rename = "type")]
    action: TransactionAction,
    #[serde(rename = "client")]
    client_id: u16,
    amount: Option<Decimal>,
}

impl Transaction {
    pub fn id(&self) -> &u32 {
        &self.id
    }
    pub fn action(&self) -> &TransactionAction {
        &self.action
    }
    pub fn client_id(&self) -> &u16 {
        &self.client_id
    }
    pub fn amount(&self) -> Option<&Decimal> {
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
