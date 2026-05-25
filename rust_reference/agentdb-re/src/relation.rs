use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::row::RowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConflictId(u64);

impl ConflictId {
    pub(crate) fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticEdge {
    first: RowId,
    second: RowId,
}

impl SemanticEdge {
    pub fn new(left: RowId, right: RowId) -> Self {
        if left <= right {
            Self {
                first: left,
                second: right,
            }
        } else {
            Self {
                first: right,
                second: left,
            }
        }
    }

    pub fn first(&self) -> RowId {
        self.first
    }

    pub fn second(&self) -> RowId {
        self.second
    }

    pub fn contains(&self, row_id: RowId) -> bool {
        self.first == row_id || self.second == row_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStatus {
    Open,
    Resolved,
    Dismissed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictRecord {
    id: ConflictId,
    rows: BTreeSet<RowId>,
    status: ConflictStatus,
}

impl ConflictRecord {
    pub(crate) fn new(id: ConflictId, rows: BTreeSet<RowId>) -> Self {
        Self {
            id,
            rows,
            status: ConflictStatus::Open,
        }
    }

    pub fn id(&self) -> ConflictId {
        self.id
    }

    pub fn rows(&self) -> &BTreeSet<RowId> {
        &self.rows
    }

    pub fn status(&self) -> ConflictStatus {
        self.status
    }

    pub(crate) fn set_status(&mut self, status: ConflictStatus) {
        self.status = status;
    }
}
