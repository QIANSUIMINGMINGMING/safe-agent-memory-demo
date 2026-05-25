use std::fs;
use std::path::Path;

use crate::db::Database;
use crate::error::DbError;

impl Database {
    pub fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), DbError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| DbError::StorageIo {
                path: parent.display().to_string(),
                message: error.to_string(),
            })?;
        }

        let snapshot =
            serde_json::to_string_pretty(self).map_err(|error| DbError::StorageFormat {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;

        fs::write(path, snapshot).map_err(|error| DbError::StorageIo {
            path: path.display().to_string(),
            message: error.to_string(),
        })
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, DbError> {
        let path = path.as_ref();
        let snapshot = fs::read_to_string(path).map_err(|error| DbError::StorageIo {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;

        let mut db: Self =
            serde_json::from_str(&snapshot).map_err(|error| DbError::StorageFormat {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;
        db.semantic_indexes.clear();
        Ok(db)
    }
}
