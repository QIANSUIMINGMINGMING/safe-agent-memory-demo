#![forbid(unsafe_code)]

mod binding;
mod db;
mod error;
mod index;
mod judge;
mod persist;
mod projection;
mod relation;
mod row;
mod semantic;
mod schema;
mod table;
mod value;

pub use binding::{BindingId, BindingSet};
pub use db::Database;
pub use error::{DbError, SchemaError, SemanticJudgeError};
pub use index::{
    FixedSemanticEmbedder, InMemoryVectorIndex, SemanticEmbedder, SemanticIndexError,
    SemanticSearchHit,
};
pub use judge::{FixedSemanticJudge, SemanticFieldValue, SemanticJudge, SemanticSubject};
pub use persist::PersistOutcome;
pub use projection::{ProjectionMode, ProjectionOutcome, ProjectionPolicy};
pub use relation::{ConflictId, ConflictRecord, ConflictStatus, SemanticEdge};
pub use row::{ExactFilter, Row, RowId, RowInput};
pub use semantic::{
    semantic_insert_action, CompatibleRelation, SemanticAction, SemanticRelation,
};
pub use schema::{ColumnRole, ColumnSchema, SemanticScope, TableSchema};
pub use table::Table;
pub use value::{DataType, Value};

#[cfg(test)]
mod tests;
