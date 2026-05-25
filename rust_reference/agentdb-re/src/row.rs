use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::SchemaError;
use crate::schema::{normalize_identifier, TableSchema};
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RowId(u64);

impl RowId {
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RowInput {
    values: BTreeMap<String, Value>,
}

impl RowInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(mut self, column: impl Into<String>, value: Value) -> Self {
        self.values.insert(column.into().trim().to_owned(), value);
        self
    }

    pub(crate) fn values(&self) -> &BTreeMap<String, Value> {
        &self.values
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactFilter {
    column: String,
    value: Value,
}

impl ExactFilter {
    pub fn new(column: impl Into<String>, value: Value) -> Result<Self, SchemaError> {
        let column = normalize_identifier(column.into(), "filter column")?;
        Ok(Self { column, value })
    }

    pub fn column(&self) -> &str {
        &self.column
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    id: RowId,
    values: Vec<Value>,
}

impl Row {
    pub(crate) fn new(id: RowId, values: Vec<Value>) -> Self {
        Self { id, values }
    }

    pub fn id(&self) -> RowId {
        self.id
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub(crate) fn value_at(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn get<'a>(&'a self, schema: &'a TableSchema, column: &str) -> Option<&'a Value> {
        let index = schema
            .columns()
            .iter()
            .position(|candidate| candidate.name() == column)?;
        self.values.get(index)
    }

    pub fn as_named_values(&self, schema: &TableSchema) -> BTreeMap<String, Value> {
        schema
            .columns()
            .iter()
            .zip(self.values.iter())
            .map(|(column, value)| (column.name().to_owned(), value.clone()))
            .collect()
    }
}
