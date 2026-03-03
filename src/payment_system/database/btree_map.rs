use std::collections::BTreeMap;

use crate::payment_system::account::Account;
use crate::payment_system::database::Database;
use crate::payment_system::database::DatabaseError;
use crate::payment_system::transaction::Transaction;
use crate::payment_system::types::ClientId;
use crate::payment_system::types::TransactionId;

pub type BTreeAccountDatabase = BTreeMap<ClientId, Account>;

pub type BTreeTransactionDatabase = BTreeMap<TransactionId, Transaction>;

impl Database for BTreeAccountDatabase {
    type Key = ClientId;
    type Record = Account;

    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError> {
        BTreeMap::insert(self, key, record);
        Ok(())
    }

    fn retrieve(&self, key: &Self::Key) -> Result<Option<Self::Record>, DatabaseError> {
        Ok(self.get(key).map(|record| record.clone()))
    }
}

impl Database for BTreeTransactionDatabase {
    type Key = TransactionId;
    type Record = Transaction;

    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError> {
        BTreeMap::insert(self, key, record);
        Ok(())
    }

    fn retrieve(&self, key: &Self::Key) -> Result<Option<Self::Record>, DatabaseError> {
        Ok(self.get(key).map(|record| record.clone()))
    }
}
