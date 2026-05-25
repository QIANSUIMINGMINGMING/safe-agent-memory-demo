use std::collections::BTreeMap;

use crate::error::SemanticJudgeError;
use crate::row::Row;
use crate::schema::TableSchema;
use crate::semantic::SemanticRelation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticFieldValue {
    column: String,
    value: String,
}

impl SemanticFieldValue {
    pub fn column(&self) -> &str {
        &self.column
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSubject {
    fields: Vec<SemanticFieldValue>,
}

impl SemanticSubject {
    pub fn from_row(schema: &TableSchema, row: &Row) -> Result<Self, SemanticJudgeError> {
        let scope = schema
            .semantic_scope()
            .ok_or(SemanticJudgeError::NoSemanticScopeDefined)?;

        let mut fields = Vec::with_capacity(scope.columns().len());
        for scope_column in scope.columns() {
            let value = row
                .get(schema, scope_column)
                .ok_or_else(|| SemanticJudgeError::MissingScopeValue(scope_column.clone()))?;

            fields.push(SemanticFieldValue {
                column: scope_column.clone(),
                value: value.to_string(),
            });
        }

        Ok(Self { fields })
    }

    pub fn fields(&self) -> &[SemanticFieldValue] {
        &self.fields
    }

    pub fn canonical_text(&self) -> String {
        let mut rendered = String::new();

        for (index, field) in self.fields.iter().enumerate() {
            if index > 0 {
                rendered.push('\n');
            }
            rendered.push_str(field.column());
            rendered.push('=');
            rendered.push_str(field.value());
        }

        rendered
    }
}

pub trait SemanticJudge {
    fn judge(
        &self,
        left: &SemanticSubject,
        right: &SemanticSubject,
    ) -> Result<SemanticRelation, SemanticJudgeError>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticPairKey {
    first: String,
    second: String,
}

impl SemanticPairKey {
    fn new(left: String, right: String) -> Self {
        if left <= right {
            Self {
                first: left,
                second: right,
            }
        } else {
            Self {
                first: right,
                second: left,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FixedSemanticJudge {
    default_relation: SemanticRelation,
    pair_relations: BTreeMap<SemanticPairKey, SemanticRelation>,
}

impl Default for FixedSemanticJudge {
    fn default() -> Self {
        Self::new(SemanticRelation::none())
    }
}

impl FixedSemanticJudge {
    pub fn new(default_relation: SemanticRelation) -> Self {
        Self {
            default_relation,
            pair_relations: BTreeMap::new(),
        }
    }

    pub fn with_pair_text(
        mut self,
        left: impl Into<String>,
        right: impl Into<String>,
        relation: SemanticRelation,
    ) -> Self {
        let key = SemanticPairKey::new(left.into(), right.into());
        self.pair_relations.insert(key, relation);
        self
    }

    pub fn with_pair(
        self,
        left: &SemanticSubject,
        right: &SemanticSubject,
        relation: SemanticRelation,
    ) -> Self {
        self.with_pair_text(left.canonical_text(), right.canonical_text(), relation)
    }
}

impl SemanticJudge for FixedSemanticJudge {
    fn judge(
        &self,
        left: &SemanticSubject,
        right: &SemanticSubject,
    ) -> Result<SemanticRelation, SemanticJudgeError> {
        let key = SemanticPairKey::new(left.canonical_text(), right.canonical_text());
        Ok(self
            .pair_relations
            .get(&key)
            .copied()
            .unwrap_or(self.default_relation))
    }
}
