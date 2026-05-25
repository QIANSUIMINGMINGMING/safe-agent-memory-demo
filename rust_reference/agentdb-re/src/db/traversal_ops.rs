use std::collections::BTreeSet;

use crate::db::Database;
use crate::error::DbError;
use crate::row::RowId;

impl Database {
    pub fn compatible_neighbors(
        &self,
        table_name: &str,
        row_id: RowId,
    ) -> Result<BTreeSet<RowId>, DbError> {
        let table = self.table(table_name)?;
        if table.row(row_id).is_none() {
            return Err(DbError::RowNotFound {
                table: table_name.to_owned(),
                row_id,
            });
        }

        Ok(table
            .compatible_edges()
            .iter()
            .filter_map(|edge| {
                if edge.first() == row_id {
                    Some(edge.second())
                } else if edge.second() == row_id {
                    Some(edge.first())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn support_neighbors(
        &self,
        table_name: &str,
        row_id: RowId,
    ) -> Result<BTreeSet<RowId>, DbError> {
        let table = self.table(table_name)?;
        if table.row(row_id).is_none() {
            return Err(DbError::RowNotFound {
                table: table_name.to_owned(),
                row_id,
            });
        }

        Ok(table
            .support_edges()
            .iter()
            .filter_map(|edge| {
                if edge.first() == row_id {
                    Some(edge.second())
                } else if edge.second() == row_id {
                    Some(edge.first())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn compressible_neighbors(
        &self,
        table_name: &str,
        row_id: RowId,
    ) -> Result<BTreeSet<RowId>, DbError> {
        let table = self.table(table_name)?;
        if table.row(row_id).is_none() {
            return Err(DbError::RowNotFound {
                table: table_name.to_owned(),
                row_id,
            });
        }

        Ok(table
            .compressible_edges()
            .iter()
            .filter_map(|edge| {
                if edge.first() == row_id {
                    Some(edge.second())
                } else if edge.second() == row_id {
                    Some(edge.first())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn conflict_neighbors(
        &self,
        table_name: &str,
        row_id: RowId,
    ) -> Result<BTreeSet<RowId>, DbError> {
        let table = self.table(table_name)?;
        if table.row(row_id).is_none() {
            return Err(DbError::RowNotFound {
                table: table_name.to_owned(),
                row_id,
            });
        }

        Ok(table
            .conflict_edges()
            .iter()
            .filter_map(|edge| {
                if edge.first() == row_id {
                    Some(edge.second())
                } else if edge.second() == row_id {
                    Some(edge.first())
                } else {
                    None
                }
            })
            .collect())
    }
}
