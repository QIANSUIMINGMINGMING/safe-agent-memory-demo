use std::collections::BTreeMap;

use crate::db::validation::resolve_exact_filters;
use crate::db::Database;
use crate::error::DbError;
use crate::row::ExactFilter;
use crate::value::Value;

impl Database {
    pub fn named_rows(&self, table_name: &str) -> Result<Vec<BTreeMap<String, Value>>, DbError> {
        Ok(self.table(table_name)?.named_rows())
    }

    pub fn scan_exact(
        &self,
        table_name: &str,
        filters: &[ExactFilter],
    ) -> Result<Vec<BTreeMap<String, Value>>, DbError> {
        let table = self.table(table_name)?;
        let resolved_filters = resolve_exact_filters(table.schema(), table_name, filters)?;

        let mut matches = Vec::new();
        for row in table.rows() {
            let is_match = resolved_filters.iter().all(|(index, filter)| {
                row.value_at(*index)
                    .is_some_and(|value| value == filter.value())
            });
            if is_match {
                matches.push(row.as_named_values(table.schema()));
            }
        }

        Ok(matches)
    }
}
