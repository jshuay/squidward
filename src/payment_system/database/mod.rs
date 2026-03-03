mod btree_map;

use std::error::Error;
use std::fmt::Display;

pub use btree_map::BTreeMapDatabase;

pub trait Database {
    type Key;
    type Record;

    /// Inserts a new record into the database. Replaces any existing records.
    fn insert(&mut self, key: Self::Key, record: Self::Record) -> Result<(), DatabaseError>;

    /// Retrieves a record from the database if it exists.
    fn retrieve(&self, key: &Self::Key) -> Result<Option<Self::Record>, DatabaseError>;

    fn delete(&mut self, key: Self::Key) -> Result<(), DatabaseError>;
}

#[derive(Debug)]
pub enum DatabaseError {}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DatabaseError {}
