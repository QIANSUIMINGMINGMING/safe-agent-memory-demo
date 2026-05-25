use crate::binding::{BindingId, BindingSet};
use crate::db::Database;
use crate::error::DbError;
use crate::row::RowId;

impl Database {
    pub fn create_binding_set(&mut self, table_name: &str) -> Result<BindingId, DbError> {
        if !self.tables.contains_key(table_name) {
            return Err(DbError::TableNotFound(table_name.to_owned()));
        }

        let binding_id = BindingId::new(self.next_binding_id.max(1));
        self.next_binding_id = binding_id.value() + 1;
        self.binding_sets.insert(
            binding_id,
            BindingSet::new(binding_id, table_name.to_owned()),
        );

        Ok(binding_id)
    }

    pub fn bind_row(&mut self, binding_id: BindingId, row_id: RowId) -> Result<bool, DbError> {
        let table_name = self.binding_set(binding_id)?.table_name().to_owned();
        if self.row(&table_name, row_id).is_err() {
            return Err(DbError::RowNotFound {
                table: table_name,
                row_id,
            });
        }

        let binding_set = self
            .binding_sets
            .get_mut(&binding_id)
            .ok_or(DbError::BindingSetNotFound(binding_id))?;
        Ok(binding_set.bind_row(row_id))
    }

    pub fn unbind_row(&mut self, binding_id: BindingId, row_id: RowId) -> Result<bool, DbError> {
        let binding_set = self
            .binding_sets
            .get_mut(&binding_id)
            .ok_or(DbError::BindingSetNotFound(binding_id))?;
        Ok(binding_set.unbind_row(row_id))
    }
}
