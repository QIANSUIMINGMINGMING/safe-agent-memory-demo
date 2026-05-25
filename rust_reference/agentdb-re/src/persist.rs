use crate::relation::{ConflictId, SemanticEdge};
use crate::row::RowId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistOutcome {
    row_id: RowId,
    compatible_edges: Vec<SemanticEdge>,
    support_edges: Vec<SemanticEdge>,
    compressible_edges: Vec<SemanticEdge>,
    conflict_edges: Vec<SemanticEdge>,
    conflict_ids: Vec<ConflictId>,
}

impl PersistOutcome {
    pub fn new(
        row_id: RowId,
        compatible_edges: Vec<SemanticEdge>,
        support_edges: Vec<SemanticEdge>,
        compressible_edges: Vec<SemanticEdge>,
        conflict_edges: Vec<SemanticEdge>,
        conflict_ids: Vec<ConflictId>,
    ) -> Self {
        Self {
            row_id,
            compatible_edges,
            support_edges,
            compressible_edges,
            conflict_edges,
            conflict_ids,
        }
    }

    pub fn plain(row_id: RowId) -> Self {
        Self::new(
            row_id,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    pub fn row_id(&self) -> RowId {
        self.row_id
    }

    pub fn compatible_edges(&self) -> &[SemanticEdge] {
        &self.compatible_edges
    }

    pub fn support_edges(&self) -> &[SemanticEdge] {
        &self.support_edges
    }

    pub fn compressible_edges(&self) -> &[SemanticEdge] {
        &self.compressible_edges
    }

    pub fn conflict_edges(&self) -> &[SemanticEdge] {
        &self.conflict_edges
    }

    pub fn conflict_ids(&self) -> &[ConflictId] {
        &self.conflict_ids
    }

    pub fn has_semantic_effects(&self) -> bool {
        !(self.compatible_edges.is_empty()
            && self.support_edges.is_empty()
            && self.compressible_edges.is_empty()
            && self.conflict_edges.is_empty()
            && self.conflict_ids.is_empty())
    }
}
