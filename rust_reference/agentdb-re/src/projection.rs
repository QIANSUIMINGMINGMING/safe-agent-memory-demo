use crate::binding::BindingId;
use crate::relation::ConflictId;
use crate::row::RowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    Conservative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionPolicy {
    mode: ProjectionMode,
    include_support_neighbors: bool,
    include_compressible_neighbors: bool,
    include_conflict_neighbors: bool,
}

impl ProjectionPolicy {
    pub fn conservative() -> Self {
        Self {
            mode: ProjectionMode::Conservative,
            include_support_neighbors: true,
            include_compressible_neighbors: false,
            include_conflict_neighbors: true,
        }
    }

    pub fn mode(&self) -> ProjectionMode {
        self.mode
    }

    pub fn include_support_neighbors(&self) -> bool {
        self.include_support_neighbors
    }

    pub fn include_compressible_neighbors(&self) -> bool {
        self.include_compressible_neighbors
    }

    pub fn include_conflict_neighbors(&self) -> bool {
        self.include_conflict_neighbors
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionOutcome {
    binding_id: BindingId,
    considered_rows: Vec<RowId>,
    accepted_rows: Vec<RowId>,
    suppressed_rows: Vec<RowId>,
    ambiguous_rows: Vec<RowId>,
    consulted_conflicts: Vec<ConflictId>,
}

impl ProjectionOutcome {
    pub fn new(
        binding_id: BindingId,
        considered_rows: Vec<RowId>,
        accepted_rows: Vec<RowId>,
        suppressed_rows: Vec<RowId>,
        ambiguous_rows: Vec<RowId>,
        consulted_conflicts: Vec<ConflictId>,
    ) -> Self {
        Self {
            binding_id,
            considered_rows,
            accepted_rows,
            suppressed_rows,
            ambiguous_rows,
            consulted_conflicts,
        }
    }

    pub fn binding_id(&self) -> BindingId {
        self.binding_id
    }

    pub fn considered_rows(&self) -> &[RowId] {
        &self.considered_rows
    }

    pub fn accepted_rows(&self) -> &[RowId] {
        &self.accepted_rows
    }

    pub fn suppressed_rows(&self) -> &[RowId] {
        &self.suppressed_rows
    }

    pub fn ambiguous_rows(&self) -> &[RowId] {
        &self.ambiguous_rows
    }

    pub fn consulted_conflicts(&self) -> &[ConflictId] {
        &self.consulted_conflicts
    }
}
