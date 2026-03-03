use std::collections::HashMap;
use std::hash::Hash;

use crate::payment_system::database::Database;
use crate::payment_system::database::DatabaseError;

pub type HashMapDatabase<K, R> = HashMap<K, R>;

impl<K, R> Database for HashMapDatabase<K, R>
where
    K: Hash + Eq,
    R: Clone,
{
    type Key = K;
    type Record = R;

    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError> {
        HashMap::insert(self, key, record);
        Ok(())
    }

    fn retrieve(&self, key: &Self::Key) -> Result<Option<Self::Record>, DatabaseError> {
        Ok(self.get(key).map(|record| record.clone()))
    }

    fn delete(&mut self, key: Self::Key) -> Result<(), DatabaseError> {
        self.remove(&key);
        Ok(())
    }
}
