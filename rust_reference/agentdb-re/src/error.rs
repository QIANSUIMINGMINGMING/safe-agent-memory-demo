use std::error::Error;
use std::fmt;

use crate::binding::BindingId;
use crate::index::SemanticIndexError;
use crate::relation::ConflictId;
use crate::row::RowId;
use crate::schema::ColumnRole;
use crate::value::DataType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    EmptyIdentifier(&'static str),
    NoColumns,
    DuplicateColumnName(String),
    EmptySemanticScope,
    DuplicateSemanticScopeColumn(String),
    MissingSemanticScope,
    UnknownSemanticScopeColumn(String),
    NonSemanticScopeColumn {
        column: String,
        role: ColumnRole,
    },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentifier(kind) => write!(f, "{kind} must be non-empty"),
            Self::NoColumns => f.write_str("table schema must contain at least one column"),
            Self::DuplicateColumnName(column) => {
                write!(f, "table schema contains duplicate column name: {column}")
            }
            Self::EmptySemanticScope => {
                f.write_str("semantic scope must contain at least one semantic column")
            }
            Self::DuplicateSemanticScopeColumn(column) => {
                write!(f, "semantic scope contains duplicate column name: {column}")
            }
            Self::MissingSemanticScope => {
                f.write_str("table schema with semantic columns must declare a semantic scope")
            }
            Self::UnknownSemanticScopeColumn(column) => {
                write!(f, "semantic scope references unknown column: {column}")
            }
            Self::NonSemanticScopeColumn { column, role } => write!(
                f,
                "semantic scope can only include semantic columns, but {column} is {role}"
            ),
        }
    }
}

impl Error for SchemaError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticJudgeError {
    NoSemanticScopeDefined,
    MissingScopeValue(String),
}

impl fmt::Display for SemanticJudgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSemanticScopeDefined => {
                f.write_str("semantic judgment requires a table schema with a semantic scope")
            }
            Self::MissingScopeValue(column) => {
                write!(f, "semantic scope column is missing from row values: {column}")
            }
        }
    }
}

impl Error for SemanticJudgeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbError {
    Schema(SchemaError),
    SemanticJudge(SemanticJudgeError),
    SemanticIndex(SemanticIndexError),
    TableAlreadyExists(String),
    TableNotFound(String),
    BindingSetNotFound(BindingId),
    StorageIo {
        path: String,
        message: String,
    },
    StorageFormat {
        path: String,
        message: String,
    },
    SemanticIndexNotBuilt(String),
    SemanticIndexRowNotIndexed {
        table: String,
        row_id: RowId,
    },
    RowNotFound {
        table: String,
        row_id: RowId,
    },
    UnknownColumn {
        table: String,
        column: String,
    },
    MissingColumn {
        table: String,
        column: String,
    },
    TypeMismatch {
        table: String,
        column: String,
        expected: DataType,
        found: DataType,
    },
    ExactFilterOnNonNormalColumn {
        table: String,
        column: String,
        role: ColumnRole,
    },
    InvalidSemanticEdge {
        table: String,
        left: RowId,
        right: RowId,
    },
    EmptyConflictRecordRows {
        table: String,
    },
    ConflictRecordNotFound {
        table: String,
        conflict_id: ConflictId,
    },
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schema(error) => error.fmt(f),
            Self::SemanticJudge(error) => error.fmt(f),
            Self::SemanticIndex(error) => error.fmt(f),
            Self::TableAlreadyExists(table) => write!(f, "table already exists: {table}"),
            Self::TableNotFound(table) => write!(f, "unknown table: {table}"),
            Self::BindingSetNotFound(binding_id) => {
                write!(f, "unknown binding set: {}", binding_id.value())
            }
            Self::StorageIo { path, message } => {
                write!(f, "storage I/O failed for {path}: {message}")
            }
            Self::StorageFormat { path, message } => {
                write!(f, "storage format error for {path}: {message}")
            }
            Self::SemanticIndexNotBuilt(table) => {
                write!(f, "semantic index has not been built for table {table}")
            }
            Self::SemanticIndexRowNotIndexed { table, row_id } => write!(
                f,
                "row {} is not present in the semantic index for table {table}",
                row_id.value()
            ),
            Self::RowNotFound { table, row_id } => {
                write!(f, "unknown row {} in table {table}", row_id.value())
            }
            Self::UnknownColumn { table, column } => {
                write!(f, "unknown column {column} for table {table}")
            }
            Self::MissingColumn { table, column } => {
                write!(f, "missing value for column {column} in table {table}")
            }
            Self::TypeMismatch {
                table,
                column,
                expected,
                found,
            } => write!(
                f,
                "type mismatch for {table}.{column}: expected {expected}, found {found}"
            ),
            Self::ExactFilterOnNonNormalColumn { table, column, role } => write!(
                f,
                "exact scan only supports normal columns, but {table}.{column} is {role}"
            ),
            Self::InvalidSemanticEdge { table, left, right } => write!(
                f,
                "semantic relation in table {table} requires two different existing rows, found {} and {}",
                left.value(),
                right.value()
            ),
            Self::EmptyConflictRecordRows { table } => {
                write!(f, "conflict record in table {table} must reference at least one row")
            }
            Self::ConflictRecordNotFound { table, conflict_id } => write!(
                f,
                "unknown conflict record {} in table {table}",
                conflict_id.value()
            ),
        }
    }
}

impl Error for DbError {}

impl From<SchemaError> for DbError {
    fn from(value: SchemaError) -> Self {
        Self::Schema(value)
    }
}

impl From<SemanticJudgeError> for DbError {
    fn from(value: SemanticJudgeError) -> Self {
        Self::SemanticJudge(value)
    }
}

impl From<SemanticIndexError> for DbError {
    fn from(value: SemanticIndexError) -> Self {
        Self::SemanticIndex(value)
    }
}
