mod binding_ops;
mod index_ops;
mod persist_ops;
mod projection_ops;
mod query_ops;
mod relation_ops;
mod storage_ops;
mod traversal_ops;
mod validation;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::binding::{BindingId, BindingSet};
use crate::error::DbError;
use crate::index::InMemoryVectorIndex;
use crate::row::{Row, RowId};
use crate::schema::TableSchema;
use crate::table::Table;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Database {
    tables: BTreeMap<String, Table>,
    binding_sets: BTreeMap<BindingId, BindingSet>,
    #[serde(skip, default)]
    semantic_indexes: BTreeMap<String, InMemoryVectorIndex>,
    next_binding_id: u64,
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_table(&mut self, schema: TableSchema) -> Result<(), DbError> {
        let table_name = schema.name().to_owned();
        if self.tables.contains_key(&table_name) {
            return Err(DbError::TableAlreadyExists(table_name));
        }

        self.tables.insert(table_name, Table::new(schema));
        Ok(())
    }

    pub fn binding_set(&self, binding_id: BindingId) -> Result<&BindingSet, DbError> {
        self.binding_sets
            .get(&binding_id)
            .ok_or(DbError::BindingSetNotFound(binding_id))
    }

    pub fn table(&self, table_name: &str) -> Result<&Table, DbError> {
        self.tables
            .get(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))
    }

    pub fn row(&self, table_name: &str, row_id: RowId) -> Result<&Row, DbError> {
        let table = self.table(table_name)?;
        table.row(row_id).ok_or_else(|| DbError::RowNotFound {
            table: table_name.to_owned(),
            row_id,
        })
    }
}
