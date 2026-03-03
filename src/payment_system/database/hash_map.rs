use std::collections::HashMap;

use crate::payment_system::account::Account;
use crate::payment_system::database::Database;
use crate::payment_system::database::DatabaseError;
use crate::payment_system::types::ClientId;

pub type HashMapAccountDatabase = HashMap<ClientId, Account>;

impl Database for HashMapAccountDatabase {
    type Key = ClientId;
    type Record = Account;

    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError> {
        HashMap::insert(self, key, record);
        Ok(())
    }

    fn retrieve(&self, key: &Self::Key) -> Result<Option<&Self::Record>, DatabaseError> {
        Ok(self.get(key))
    }
}
