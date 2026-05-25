use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::SchemaError;
use crate::value::DataType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnRole {
    Normal,
    Semantic,
}

impl fmt::Display for ColumnRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Normal => "normal",
            Self::Semantic => "semantic",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnSchema {
    name: String,
    data_type: DataType,
    role: ColumnRole,
}

impl ColumnSchema {
    pub fn new(
        name: impl Into<String>,
        data_type: DataType,
        role: ColumnRole,
    ) -> Result<Self, SchemaError> {
        let name = normalize_identifier(name.into(), "column name")?;
        Ok(Self {
            name,
            data_type,
            role,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn role(&self) -> ColumnRole {
        self.role
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableSchema {
    name: String,
    columns: Vec<ColumnSchema>,
    semantic_scope: Option<SemanticScope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticScope {
    columns: Vec<String>,
}

impl SemanticScope {
    pub fn new<I, S>(columns: I) -> Result<Self, SchemaError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut normalized_columns = Vec::new();
        let mut seen = BTreeSet::new();

        for raw_column in columns {
            let column = normalize_identifier(raw_column.into(), "semantic scope column")?;
            if !seen.insert(column.clone()) {
                return Err(SchemaError::DuplicateSemanticScopeColumn(column));
            }
            normalized_columns.push(column);
        }

        if normalized_columns.is_empty() {
            return Err(SchemaError::EmptySemanticScope);
        }

        Ok(Self {
            columns: normalized_columns,
        })
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}

impl TableSchema {
    pub fn new(
        name: impl Into<String>,
        columns: Vec<ColumnSchema>,
        semantic_scope: Option<SemanticScope>,
    ) -> Result<Self, SchemaError> {
        let name = normalize_identifier(name.into(), "table name")?;
        if columns.is_empty() {
            return Err(SchemaError::NoColumns);
        }

        let mut seen = BTreeSet::new();
        let mut semantic_column_names = BTreeSet::new();
        for column in &columns {
            if !seen.insert(column.name.clone()) {
                return Err(SchemaError::DuplicateColumnName(column.name.clone()));
            }
            if column.role == ColumnRole::Semantic {
                semantic_column_names.insert(column.name.clone());
            }
        }

        if !semantic_column_names.is_empty() && semantic_scope.is_none() {
            return Err(SchemaError::MissingSemanticScope);
        }

        if let Some(scope) = &semantic_scope {
            for scope_column in scope.columns() {
                let Some(column) = columns.iter().find(|column| column.name == *scope_column) else {
                    return Err(SchemaError::UnknownSemanticScopeColumn(scope_column.clone()));
                };

                if column.role != ColumnRole::Semantic {
                    return Err(SchemaError::NonSemanticScopeColumn {
                        column: scope_column.clone(),
                        role: column.role,
                    });
                }
            }
        }

        Ok(Self {
            name,
            columns,
            semantic_scope,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn columns(&self) -> &[ColumnSchema] {
        &self.columns
    }

    pub fn column(&self, name: &str) -> Option<&ColumnSchema> {
        self.columns.iter().find(|column| column.name == name)
    }

    pub fn semantic_scope(&self) -> Option<&SemanticScope> {
        self.semantic_scope.as_ref()
    }
}

pub(crate) fn normalize_identifier(
    raw: String,
    kind: &'static str,
) -> Result<String, SchemaError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SchemaError::EmptyIdentifier(kind));
    }
    Ok(trimmed.to_owned())
}
