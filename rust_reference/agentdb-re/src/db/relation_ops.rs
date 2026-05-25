use std::collections::{BTreeMap, BTreeSet};

use crate::db::Database;
use crate::error::DbError;
use crate::relation::{ConflictId, ConflictRecord, ConflictStatus, SemanticEdge};
use crate::row::RowId;

impl Database {
    pub fn add_compatible_edge(
        &mut self,
        table_name: &str,
        left: RowId,
        right: RowId,
    ) -> Result<(), DbError> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
        table.add_compatible_edge(left, right)
    }

    pub fn add_support_edge(
        &mut self,
        table_name: &str,
        left: RowId,
        right: RowId,
    ) -> Result<(), DbError> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
        table.add_support_edge(left, right)
    }

    pub fn add_compressible_edge(
        &mut self,
        table_name: &str,
        left: RowId,
        right: RowId,
    ) -> Result<(), DbError> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
        table.add_compressible_edge(left, right)
    }

    pub fn add_conflict_edge(
        &mut self,
        table_name: &str,
        left: RowId,
        right: RowId,
    ) -> Result<(), DbError> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
        table.add_conflict_edge(left, right)
    }

    pub fn support_edges(&self, table_name: &str) -> Result<&BTreeSet<SemanticEdge>, DbError> {
        Ok(self.table(table_name)?.support_edges())
    }

    pub fn compatible_edges(
        &self,
        table_name: &str,
    ) -> Result<&BTreeSet<SemanticEdge>, DbError> {
        Ok(self.table(table_name)?.compatible_edges())
    }

    pub fn conflict_edges(&self, table_name: &str) -> Result<&BTreeSet<SemanticEdge>, DbError> {
        Ok(self.table(table_name)?.conflict_edges())
    }

    pub fn compressible_edges(
        &self,
        table_name: &str,
    ) -> Result<&BTreeSet<SemanticEdge>, DbError> {
        Ok(self.table(table_name)?.compressible_edges())
    }

    pub fn create_conflict_record<I>(
        &mut self,
        table_name: &str,
        row_ids: I,
    ) -> Result<ConflictId, DbError>
    where
        I: IntoIterator<Item = RowId>,
    {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
        table.create_conflict_record(row_ids)
    }

    pub fn conflict_record(
        &self,
        table_name: &str,
        conflict_id: ConflictId,
    ) -> Result<&ConflictRecord, DbError> {
        let table = self.table(table_name)?;
        table.conflict_record(conflict_id)
            .ok_or_else(|| DbError::ConflictRecordNotFound {
                table: table_name.to_owned(),
                conflict_id,
            })
    }

    pub fn conflict_records(
        &self,
        table_name: &str,
    ) -> Result<&BTreeMap<ConflictId, ConflictRecord>, DbError> {
        Ok(self.table(table_name)?.conflict_records())
    }

    pub fn set_conflict_status(
        &mut self,
        table_name: &str,
        conflict_id: ConflictId,
        status: ConflictStatus,
    ) -> Result<(), DbError> {
        let table = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
        table.set_conflict_status(conflict_id, status)
    }
}
