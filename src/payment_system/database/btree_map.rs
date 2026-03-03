use std::collections::BTreeMap;

use crate::payment_system::account::Account;
use crate::payment_system::database::Database;
use crate::payment_system::database::DatabaseError;
use crate::payment_system::types::ClientId;

pub type BTreeAccountDatabase = BTreeMap<ClientId, Account>;

impl Database for BTreeAccountDatabase {
    type Key = ClientId;
    type Record = Account;

    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError> {
        BTreeMap::insert(self, key, record);
        Ok(())
    }

    fn retrieve(&self, key: &Self::Key) -> Result<Option<&Self::Record>, DatabaseError> {
        Ok(self.get(key))
    }
}
