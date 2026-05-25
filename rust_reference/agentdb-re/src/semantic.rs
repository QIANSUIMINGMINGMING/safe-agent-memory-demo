#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibleRelation {
    Generic,
    Nonredundant,
    Redundant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticRelation {
    Conflict,
    None,
    Compatible(CompatibleRelation),
}

impl SemanticRelation {
    pub const fn compatible() -> Self {
        Self::Compatible(CompatibleRelation::Generic)
    }

    pub const fn support() -> Self {
        Self::Compatible(CompatibleRelation::Nonredundant)
    }

    pub const fn compressible() -> Self {
        Self::Compatible(CompatibleRelation::Redundant)
    }

    pub const fn conflict() -> Self {
        Self::Conflict
    }

    pub const fn none() -> Self {
        Self::None
    }

    pub const fn is_conflict(self) -> bool {
        matches!(self, Self::Conflict)
    }

    pub const fn is_support(self) -> bool {
        matches!(self, Self::Compatible(CompatibleRelation::Nonredundant))
    }

    pub const fn is_compressible(self) -> bool {
        matches!(self, Self::Compatible(CompatibleRelation::Redundant))
    }

    pub const fn is_generic_compatible(self) -> bool {
        matches!(self, Self::Compatible(CompatibleRelation::Generic))
    }

    pub const fn is_compatible(self) -> bool {
        matches!(self, Self::Compatible(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticAction {
    Insert,
    Block,
}

pub fn semantic_insert_action(relations: &[SemanticRelation]) -> SemanticAction {
    for relation in relations {
        if relation.is_conflict() {
            return SemanticAction::Block;
        }
    }

    SemanticAction::Insert
}
