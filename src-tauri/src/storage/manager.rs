use std::ops::Deref;

use crate::storage::db::DatabaseManager;

// Note: Removed Debug derive as DatabaseManager doesn't derive it.
// Clone is fine due to the DatabaseManager using Arc internally.
#[derive(Clone)]
pub struct StorageManager {
    db_manager: DatabaseManager,
}

impl StorageManager {
    pub fn new(db_manager: DatabaseManager) -> Self {
        Self { db_manager }
    }

    pub fn db(&self) -> DatabaseManager {
        self.db_manager.clone()
    }
}

impl Deref for StorageManager {
    type Target = DatabaseManager;
    fn deref(&self) -> &Self::Target {
        &self.db_manager
    }
}
