use crate::error::DbError;
use crate::row::{ExactFilter, Row, RowInput};
use crate::schema::{ColumnRole, TableSchema};
use crate::value::Value;

pub(super) fn materialize_row_values(
    table_name: &str,
    schema: &TableSchema,
    input: &RowInput,
) -> Result<Vec<Value>, DbError> {
    for column_name in input.values().keys() {
        if schema.column(column_name).is_none() {
            return Err(DbError::UnknownColumn {
                table: table_name.to_owned(),
                column: column_name.clone(),
            });
        }
    }

    let mut values = Vec::with_capacity(schema.columns().len());
    for column in schema.columns() {
        let value = input
            .values()
            .get(column.name())
            .ok_or_else(|| DbError::MissingColumn {
                table: table_name.to_owned(),
                column: column.name().to_owned(),
            })?;

        if value.data_type() != column.data_type() {
            return Err(DbError::TypeMismatch {
                table: table_name.to_owned(),
                column: column.name().to_owned(),
                expected: column.data_type(),
                found: value.data_type(),
            });
        }

        values.push(value.clone());
    }

    Ok(values)
}

pub(super) fn row_matches_exact_filters(row: &Row, filters: &[(usize, &ExactFilter)]) -> bool {
    filters.iter().all(|(index, filter)| {
        row.value_at(*index)
            .is_some_and(|value| value == filter.value())
    })
}

pub(super) fn resolve_exact_filters<'a>(
    schema: &TableSchema,
    table_name: &str,
    filters: &'a [ExactFilter],
) -> Result<Vec<(usize, &'a ExactFilter)>, DbError> {
    let mut resolved = Vec::with_capacity(filters.len());

    for filter in filters {
        let index = schema
            .columns()
            .iter()
            .position(|column| column.name() == filter.column())
            .ok_or_else(|| DbError::UnknownColumn {
                table: table_name.to_owned(),
                column: filter.column().to_owned(),
            })?;

        let column = &schema.columns()[index];
        if column.role() != ColumnRole::Normal {
            return Err(DbError::ExactFilterOnNonNormalColumn {
                table: table_name.to_owned(),
                column: column.name().to_owned(),
                role: column.role(),
            });
        }

        if filter.value().data_type() != column.data_type() {
            return Err(DbError::TypeMismatch {
                table: table_name.to_owned(),
                column: column.name().to_owned(),
                expected: column.data_type(),
                found: filter.value().data_type(),
            });
        }

        resolved.push((index, filter));
    }

    Ok(resolved)
}
