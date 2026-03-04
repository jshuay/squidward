use std::collections::BTreeMap;

use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;

pub type Accounts = BTreeMap<ClientId, Account>;

#[derive(Debug, Clone)]
pub struct Account {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Account {
    pub fn new(client_id: ClientId) -> Self {
        Account {
            client_id,
            available: Amount::ZERO,
            held: Amount::ZERO,
            locked: false,
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn available_funds(&self) -> &Amount {
        &self.available
    }

    pub fn available_funds_mut(&mut self) -> &mut Amount {
        &mut self.available
    }

    pub fn held_funds(&self) -> &Amount {
        &self.held
    }

    pub fn held_funds_mut(&mut self) -> &mut Amount {
        &mut self.held
    }

    pub fn total_funds(&self) -> Amount {
        self.available + self.held
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }
}
