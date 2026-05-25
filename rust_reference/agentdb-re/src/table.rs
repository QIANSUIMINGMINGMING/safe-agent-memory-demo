use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::DbError;
use crate::relation::{ConflictId, ConflictRecord, ConflictStatus, SemanticEdge};
use crate::row::{Row, RowId};
use crate::schema::TableSchema;
use crate::value::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    schema: TableSchema,
    rows: Vec<Row>,
    #[serde(default)]
    compatible_edges: BTreeSet<SemanticEdge>,
    support_edges: BTreeSet<SemanticEdge>,
    #[serde(default)]
    compressible_edges: BTreeSet<SemanticEdge>,
    conflict_edges: BTreeSet<SemanticEdge>,
    conflict_records: BTreeMap<ConflictId, ConflictRecord>,
    next_row_id: u64,
    next_conflict_id: u64,
}

impl Table {
    pub(crate) fn new(schema: TableSchema) -> Self {
        Self {
            schema,
            rows: Vec::new(),
            compatible_edges: BTreeSet::new(),
            support_edges: BTreeSet::new(),
            compressible_edges: BTreeSet::new(),
            conflict_edges: BTreeSet::new(),
            conflict_records: BTreeMap::new(),
            next_row_id: 1,
            next_conflict_id: 1,
        }
    }

    pub(crate) fn insert_row(&mut self, values: Vec<Value>) -> RowId {
        let row_id = RowId::new(self.next_row_id);
        self.next_row_id += 1;
        let row = Row::new(row_id, values);
        self.rows.push(row);
        row_id
    }

    pub fn schema(&self) -> &TableSchema {
        &self.schema
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn row(&self, row_id: RowId) -> Option<&Row> {
        self.rows.iter().find(|row| row.id() == row_id)
    }

    pub fn add_support_edge(&mut self, left: RowId, right: RowId) -> Result<(), DbError> {
        let edge = self.validate_edge(left, right)?;
        self.compatible_edges.insert(edge);
        self.support_edges.insert(edge);
        Ok(())
    }

    pub fn add_conflict_edge(&mut self, left: RowId, right: RowId) -> Result<(), DbError> {
        let edge = self.validate_edge(left, right)?;
        self.conflict_edges.insert(edge);
        Ok(())
    }

    pub fn add_compressible_edge(&mut self, left: RowId, right: RowId) -> Result<(), DbError> {
        let edge = self.validate_edge(left, right)?;
        self.compatible_edges.insert(edge);
        self.compressible_edges.insert(edge);
        Ok(())
    }

    pub fn add_compatible_edge(&mut self, left: RowId, right: RowId) -> Result<(), DbError> {
        let edge = self.validate_edge(left, right)?;
        self.compatible_edges.insert(edge);
        Ok(())
    }

    pub fn compatible_edges(&self) -> &BTreeSet<SemanticEdge> {
        &self.compatible_edges
    }

    pub fn support_edges(&self) -> &BTreeSet<SemanticEdge> {
        &self.support_edges
    }

    pub fn conflict_edges(&self) -> &BTreeSet<SemanticEdge> {
        &self.conflict_edges
    }

    pub fn compressible_edges(&self) -> &BTreeSet<SemanticEdge> {
        &self.compressible_edges
    }

    pub fn has_support_edge(&self, left: RowId, right: RowId) -> bool {
        self.support_edges.contains(&SemanticEdge::new(left, right))
    }

    pub fn has_compatible_edge(&self, left: RowId, right: RowId) -> bool {
        self.compatible_edges.contains(&SemanticEdge::new(left, right))
    }

    pub fn has_compressible_edge(&self, left: RowId, right: RowId) -> bool {
        self.compressible_edges.contains(&SemanticEdge::new(left, right))
    }

    pub fn has_conflict_edge(&self, left: RowId, right: RowId) -> bool {
        self.conflict_edges.contains(&SemanticEdge::new(left, right))
    }

    pub fn create_conflict_record<I>(&mut self, row_ids: I) -> Result<ConflictId, DbError>
    where
        I: IntoIterator<Item = RowId>,
    {
        let rows: BTreeSet<RowId> = row_ids.into_iter().collect();
        if rows.is_empty() {
            return Err(DbError::EmptyConflictRecordRows {
                table: self.schema.name().to_owned(),
            });
        }

        for row_id in &rows {
            if self.row(*row_id).is_none() {
                return Err(DbError::RowNotFound {
                    table: self.schema.name().to_owned(),
                    row_id: *row_id,
                });
            }
        }

        let conflict_id = ConflictId::new(self.next_conflict_id);
        self.next_conflict_id += 1;
        self.conflict_records
            .insert(conflict_id, ConflictRecord::new(conflict_id, rows));

        Ok(conflict_id)
    }

    pub fn conflict_record(&self, conflict_id: ConflictId) -> Option<&ConflictRecord> {
        self.conflict_records.get(&conflict_id)
    }

    pub fn conflict_records(&self) -> &BTreeMap<ConflictId, ConflictRecord> {
        &self.conflict_records
    }

    pub fn set_conflict_status(
        &mut self,
        conflict_id: ConflictId,
        status: ConflictStatus,
    ) -> Result<(), DbError> {
        let Some(conflict) = self.conflict_records.get_mut(&conflict_id) else {
            return Err(DbError::ConflictRecordNotFound {
                table: self.schema.name().to_owned(),
                conflict_id,
            });
        };

        conflict.set_status(status);
        Ok(())
    }

    pub fn named_rows(&self) -> Vec<BTreeMap<String, Value>> {
        self.rows
            .iter()
            .map(|row| row.as_named_values(&self.schema))
            .collect()
    }

    fn validate_edge(&self, left: RowId, right: RowId) -> Result<SemanticEdge, DbError> {
        if left == right || self.row(left).is_none() || self.row(right).is_none() {
            return Err(DbError::InvalidSemanticEdge {
                table: self.schema.name().to_owned(),
                left,
                right,
            });
        }

        Ok(SemanticEdge::new(left, right))
    }
}
