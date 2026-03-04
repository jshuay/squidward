use std::collections::BTreeMap;
use std::fmt;

use crate::payment_system::types::Amount;
use crate::payment_system::types::ClientId;

pub type Accounts = BTreeMap<ClientId, Account>;

const PRECISION: u32 = 4;

/// Structure that represents the latest state of a client's account.
#[derive(Debug, Clone)]
pub struct Account {
    client_id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.client_id,
            self.available.round_dp(PRECISION).normalize(),
            self.held.round_dp(PRECISION).normalize(),
            self.total_funds().round_dp(PRECISION).normalize(),
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

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use crate::payment_system::account::Account;

    #[test]
    fn total_funds_correctly_adds_available_and_held_funds() {
        let account = Account {
            client_id: 1,
            available: dec!(3),
            held: dec!(7),
            locked: false,
        };
        assert_eq!(account.total_funds(), dec!(10));

        let account = Account {
            client_id: 1,
            available: dec!(-3),
            held: dec!(7),
            locked: false,
        };
        assert_eq!(account.total_funds(), dec!(4));

        let account = Account {
            client_id: 1,
            available: dec!(-10),
            held: dec!(7),
            locked: false,
        };
        assert_eq!(account.total_funds(), dec!(-3));
    }

    #[test]
    fn account_displays_amounts_with_4_decimal_precision() {
        let account = Account {
            client_id: 1,
            available: dec!(1.0002),
            held: dec!(0.9998),
            locked: false,
        };

        assert_eq!("1,1.0002,0.9998,2,false", account.to_string());
    }

    #[test]
    fn account_rounds_amounts_using_bankers_rounding_strategy() {
        let account = Account {
            client_id: 1,
            available: dec!(0.00055),
            held: dec!(0.00065),
            locked: false,
        };
        assert_eq!("1,0.0006,0.0006,0.0012,false", account.to_string());
    }

    #[test]
    fn account_displays_amounts_without_extraneous_trailing_0s() {
        let account = Account {
            client_id: 1,
            available: dec!(0.505),
            held: dec!(0.495),
            locked: false,
        };
        assert_eq!("1,0.505,0.495,1,false", account.to_string());
    }
}
