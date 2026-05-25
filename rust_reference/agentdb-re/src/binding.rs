use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::row::RowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BindingId(u64);

impl BindingId {
    pub(crate) fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn from_value(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingSet {
    id: BindingId,
    table_name: String,
    rows: BTreeSet<RowId>,
}

impl BindingSet {
    pub(crate) fn new(id: BindingId, table_name: String) -> Self {
        Self {
            id,
            table_name,
            rows: BTreeSet::new(),
        }
    }

    pub fn id(&self) -> BindingId {
        self.id
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn rows(&self) -> &BTreeSet<RowId> {
        &self.rows
    }

    pub fn contains(&self, row_id: RowId) -> bool {
        self.rows.contains(&row_id)
    }

    pub(crate) fn bind_row(&mut self, row_id: RowId) -> bool {
        self.rows.insert(row_id)
    }

    pub(crate) fn unbind_row(&mut self, row_id: RowId) -> bool {
        self.rows.remove(&row_id)
    }
}
