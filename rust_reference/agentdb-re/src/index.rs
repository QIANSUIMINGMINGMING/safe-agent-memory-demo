use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::row::RowId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticIndexError {
    MissingEmbedding(String),
    EmptyEmbedding,
    InconsistentEmbeddingDimension { expected: usize, found: usize },
    QueryDimensionMismatch { expected: usize, found: usize },
}

impl fmt::Display for SemanticIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEmbedding(text) => {
                write!(f, "missing embedding for semantic text: {text}")
            }
            Self::EmptyEmbedding => f.write_str("semantic index embeddings must be non-empty"),
            Self::InconsistentEmbeddingDimension { expected, found } => write!(
                f,
                "semantic index embeddings must share one dimension, expected {expected}, found {found}"
            ),
            Self::QueryDimensionMismatch { expected, found } => write!(
                f,
                "semantic search query dimension mismatch: expected {expected}, found {found}"
            ),
        }
    }
}

impl Error for SemanticIndexError {}

pub trait SemanticEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, SemanticIndexError>;
}

#[derive(Debug, Clone, Default)]
pub struct FixedSemanticEmbedder {
    embeddings: BTreeMap<String, Vec<f32>>,
}

impl FixedSemanticEmbedder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_text_embedding(mut self, text: impl Into<String>, embedding: Vec<f32>) -> Self {
        self.embeddings.insert(text.into(), embedding);
        self
    }
}

impl SemanticEmbedder for FixedSemanticEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>, SemanticIndexError> {
        self.embeddings
            .get(text)
            .cloned()
            .ok_or_else(|| SemanticIndexError::MissingEmbedding(text.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSearchHit {
    row_id: RowId,
    score: f32,
}

impl SemanticSearchHit {
    pub fn new(row_id: RowId, score: f32) -> Self {
        Self { row_id, score }
    }

    pub fn row_id(&self) -> RowId {
        self.row_id
    }

    pub fn score(&self) -> f32 {
        self.score
    }
}

#[derive(Debug, Clone)]
struct IndexedVectorEntry {
    vector: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct InMemoryVectorIndex {
    dimension: usize,
    entries: BTreeMap<RowId, IndexedVectorEntry>,
}

impl InMemoryVectorIndex {
    pub fn build<I>(entries: I) -> Result<Self, SemanticIndexError>
    where
        I: IntoIterator<Item = (RowId, Vec<f32>)>,
    {
        let mut dimension = None;
        let mut built_entries = BTreeMap::new();

        for (row_id, vector) in entries {
            validate_embedding_dimension(dimension, &vector)?;
            dimension = Some(vector.len());
            built_entries.insert(row_id, IndexedVectorEntry { vector });
        }

        let Some(dimension) = dimension else {
            return Err(SemanticIndexError::EmptyEmbedding);
        };

        Ok(Self {
            dimension,
            entries: built_entries,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn contains_row(&self, row_id: RowId) -> bool {
        self.entries.contains_key(&row_id)
    }

    pub fn insert(&mut self, row_id: RowId, vector: Vec<f32>) -> Result<(), SemanticIndexError> {
        validate_embedding_dimension(Some(self.dimension), &vector)?;
        self.entries.insert(row_id, IndexedVectorEntry { vector });
        Ok(())
    }

    pub fn search(
        &self,
        query: &[f32],
        limit: usize,
        excluded_row: Option<RowId>,
    ) -> Result<Vec<SemanticSearchHit>, SemanticIndexError> {
        validate_query_dimension(self.dimension, query)?;
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut hits: Vec<SemanticSearchHit> = self
            .entries
            .iter()
            .filter_map(|(row_id, entry)| {
                if Some(*row_id) == excluded_row {
                    return None;
                }

                Some(SemanticSearchHit::new(
                    *row_id,
                    cosine_similarity(query, &entry.vector),
                ))
            })
            .collect();

        hits.sort_by(compare_hits_desc);
        if hits.len() > limit {
            hits.truncate(limit);
        }

        Ok(hits)
    }

    pub fn search_rows<I>(
        &self,
        query: &[f32],
        row_ids: I,
        limit: usize,
        excluded_row: Option<RowId>,
    ) -> Result<Vec<SemanticSearchHit>, SemanticIndexError>
    where
        I: IntoIterator<Item = RowId>,
    {
        validate_query_dimension(self.dimension, query)?;
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut hits = Vec::new();
        for row_id in row_ids {
            if Some(row_id) == excluded_row {
                continue;
            }

            let Some(entry) = self.entries.get(&row_id) else {
                continue;
            };
            hits.push(SemanticSearchHit::new(
                row_id,
                cosine_similarity(query, &entry.vector),
            ));
        }

        hits.sort_by(compare_hits_desc);
        if hits.len() > limit {
            hits.truncate(limit);
        }

        Ok(hits)
    }

    pub fn row_vector(&self, row_id: RowId) -> Option<&[f32]> {
        self.entries.get(&row_id).map(|entry| entry.vector.as_slice())
    }
}

fn validate_embedding_dimension(
    expected_dimension: Option<usize>,
    embedding: &[f32],
) -> Result<(), SemanticIndexError> {
    if embedding.is_empty() {
        return Err(SemanticIndexError::EmptyEmbedding);
    }

    if let Some(expected_dimension) = expected_dimension {
        if embedding.len() != expected_dimension {
            return Err(SemanticIndexError::InconsistentEmbeddingDimension {
                expected: expected_dimension,
                found: embedding.len(),
            });
        }
    }

    Ok(())
}

fn validate_query_dimension(
    expected_dimension: usize,
    query: &[f32],
) -> Result<(), SemanticIndexError> {
    if query.is_empty() {
        return Err(SemanticIndexError::EmptyEmbedding);
    }
    if query.len() != expected_dimension {
        return Err(SemanticIndexError::QueryDimensionMismatch {
            expected: expected_dimension,
            found: query.len(),
        });
    }

    Ok(())
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(left, right)| left * right)
        .sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();

    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn compare_hits_desc(left: &SemanticSearchHit, right: &SemanticSearchHit) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.row_id.cmp(&right.row_id))
}
