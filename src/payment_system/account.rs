use std::collections::BTreeMap;
use std::fmt::Display;

use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;

pub type Accounts = BTreeMap<ClientId, Account>;

const PRECISION: u32 = 4;

#[derive(Debug, Clone)]
pub struct Account {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.client_id,
            self.available.round_dp(PRECISION),
            self.held.round_dp(PRECISION),
            self.total_funds().round_dp(PRECISION),
            self.locked
        )
    }
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
