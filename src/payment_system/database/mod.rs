mod btree_map;
mod hash_map;

pub use btree_map::BTreeAccountDatabase;
pub use hash_map::HashMapAccountDatabase;

pub trait Database {
    type Key;
    type Record;

    /// Inserts a new record into the database. Replaces any existing records.
    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError>;

    /// Retrieves a record from the database if it exists.
    fn retrieve(&self, key: &Self::Key) -> Result<Option<&Self::Record>, DatabaseError>;
}

pub enum DatabaseError {}
