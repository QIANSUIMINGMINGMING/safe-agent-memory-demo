use crate::db::validation::{
    materialize_row_values, resolve_exact_filters, row_matches_exact_filters,
};
use crate::db::Database;
use crate::error::DbError;
use crate::index::SemanticEmbedder;
use crate::judge::{SemanticJudge, SemanticSubject};
use crate::persist::PersistOutcome;
use crate::relation::SemanticEdge;
use crate::row::{ExactFilter, Row, RowId, RowInput};
use crate::semantic::SemanticRelation;
use crate::table::Table;
use std::collections::BTreeSet;

impl Database {
    pub fn persist(&mut self, table_name: &str, input: RowInput) -> Result<PersistOutcome, DbError> {
        let row_id = {
            let table = self
                .tables
                .get_mut(table_name)
                .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;

            let values = materialize_row_values(table_name, table.schema(), &input)?;
            table.insert_row(values)
        };
        self.invalidate_semantic_index(table_name);
        Ok(PersistOutcome::plain(row_id))
    }

    pub fn persist_with_index<E: SemanticEmbedder>(
        &mut self,
        table_name: &str,
        input: RowInput,
        embedder: &E,
    ) -> Result<PersistOutcome, DbError> {
        let outcome = self.persist(table_name, input)?;
        self.refresh_semantic_index_for_row(table_name, outcome.row_id(), embedder)?;
        Ok(outcome)
    }

    pub fn persist_semantic<J: SemanticJudge>(
        &mut self,
        table_name: &str,
        input: RowInput,
        judge: &J,
    ) -> Result<PersistOutcome, DbError> {
        self.persist_semantic_with_filters(table_name, input, &[], judge)
    }

    pub fn persist_semantic_with_filters<J: SemanticJudge>(
        &mut self,
        table_name: &str,
        input: RowInput,
        candidate_filters: &[ExactFilter],
        judge: &J,
    ) -> Result<PersistOutcome, DbError> {
        let outcome = {
            let table = self
                .tables
                .get_mut(table_name)
                .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;

            let values = materialize_row_values(table_name, table.schema(), &input)?;
            let resolved_filters =
                resolve_exact_filters(table.schema(), table_name, candidate_filters)?;
            let candidate_row_ids: Vec<RowId> = table
                .rows()
                .iter()
                .filter(|existing_row| row_matches_exact_filters(existing_row, &resolved_filters))
                .map(|existing_row| existing_row.id())
                .collect();

            persist_insert_with_candidate_rows(table, values, &candidate_row_ids, judge)?
        };
        self.invalidate_semantic_index(table_name);
        Ok(outcome)
    }

    pub fn persist_semantic_with_candidates<J: SemanticJudge>(
        &mut self,
        table_name: &str,
        input: RowInput,
        candidate_row_ids: &[RowId],
        judge: &J,
    ) -> Result<PersistOutcome, DbError> {
        let outcome = {
            let table = self
                .tables
                .get_mut(table_name)
                .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;

            let values = materialize_row_values(table_name, table.schema(), &input)?;
            persist_insert_with_candidate_rows(table, values, candidate_row_ids, judge)?
        };
        self.invalidate_semantic_index(table_name);
        Ok(outcome)
    }

    pub fn persist_semantic_with_index<J: SemanticJudge, E: SemanticEmbedder>(
        &mut self,
        table_name: &str,
        input: RowInput,
        judge: &J,
        embedder: &E,
    ) -> Result<PersistOutcome, DbError> {
        let outcome = self.persist_semantic(table_name, input, judge)?;
        self.refresh_semantic_index_for_row(table_name, outcome.row_id(), embedder)?;
        Ok(outcome)
    }

    pub fn persist_semantic_with_candidates_and_index<J: SemanticJudge, E: SemanticEmbedder>(
        &mut self,
        table_name: &str,
        input: RowInput,
        candidate_row_ids: &[RowId],
        judge: &J,
        embedder: &E,
    ) -> Result<PersistOutcome, DbError> {
        let outcome =
            self.persist_semantic_with_candidates(table_name, input, candidate_row_ids, judge)?;
        self.refresh_semantic_index_for_row(table_name, outcome.row_id(), embedder)?;
        Ok(outcome)
    }

    pub fn enrich_row_semantic<J: SemanticJudge>(
        &mut self,
        table_name: &str,
        row_id: RowId,
        candidate_row_ids: &[RowId],
        judge: &J,
    ) -> Result<PersistOutcome, DbError> {
        let outcome = {
            let table = self
                .tables
                .get_mut(table_name)
                .ok_or_else(|| DbError::TableNotFound(table_name.to_owned()))?;
            enrich_existing_row_with_candidates(table, row_id, candidate_row_ids, judge)?
        };
        self.invalidate_semantic_index(table_name);
        Ok(outcome)
    }

    pub fn persist_semantic_with_filters_and_index<J: SemanticJudge, E: SemanticEmbedder>(
        &mut self,
        table_name: &str,
        input: RowInput,
        candidate_filters: &[ExactFilter],
        judge: &J,
        embedder: &E,
    ) -> Result<PersistOutcome, DbError> {
        let outcome =
            self.persist_semantic_with_filters(table_name, input, candidate_filters, judge)?;
        self.refresh_semantic_index_for_row(table_name, outcome.row_id(), embedder)?;
        Ok(outcome)
    }
}

fn persist_insert_with_candidate_rows<J: SemanticJudge>(
    table: &mut Table,
    values: Vec<crate::value::Value>,
    candidate_row_ids: &[RowId],
    judge: &J,
) -> Result<PersistOutcome, DbError> {
    let candidate_subject = {
        let candidate_row = Row::new(RowId::new(0), values.clone());
        SemanticSubject::from_row(table.schema(), &candidate_row)?
    };
    let (compatible_targets, support_targets, compressible_targets, conflict_targets) =
        judge_candidate_rows(table, &candidate_subject, candidate_row_ids, None, judge)?;
    let row_id = table.insert_row(values);
    let (compatible_edges, support_edges, compressible_edges, conflict_edges, conflict_ids) =
        apply_semantic_effects(
        table,
        row_id,
        &compatible_targets,
        &support_targets,
        &compressible_targets,
        &conflict_targets,
    )?;

    Ok(PersistOutcome::new(
        row_id,
        compatible_edges,
        support_edges,
        compressible_edges,
        conflict_edges,
        conflict_ids,
    ))
}

fn enrich_existing_row_with_candidates<J: SemanticJudge>(
    table: &mut Table,
    row_id: RowId,
    candidate_row_ids: &[RowId],
    judge: &J,
) -> Result<PersistOutcome, DbError> {
    let subject = {
        let row = table.row(row_id).ok_or_else(|| DbError::RowNotFound {
            table: table.schema().name().to_owned(),
            row_id,
        })?;
        SemanticSubject::from_row(table.schema(), row)?
    };
    let (compatible_targets, support_targets, compressible_targets, conflict_targets) =
        judge_candidate_rows(table, &subject, candidate_row_ids, Some(row_id), judge)?;
    let (compatible_edges, support_edges, compressible_edges, conflict_edges, conflict_ids) =
        apply_semantic_effects(
        table,
        row_id,
        &compatible_targets,
        &support_targets,
        &compressible_targets,
        &conflict_targets,
    )?;

    Ok(PersistOutcome::new(
        row_id,
        compatible_edges,
        support_edges,
        compressible_edges,
        conflict_edges,
        conflict_ids,
    ))
}

fn judge_candidate_rows<J: SemanticJudge>(
    table: &Table,
    subject: &SemanticSubject,
    candidate_row_ids: &[RowId],
    skipped_row: Option<RowId>,
    judge: &J,
) -> Result<(Vec<RowId>, Vec<RowId>, Vec<RowId>, Vec<RowId>), DbError> {
    let mut compatible_targets = Vec::new();
    let mut support_targets = Vec::new();
    let mut compressible_targets = Vec::new();
    let mut conflict_targets = Vec::new();
    let mut visited = BTreeSet::new();

    for candidate_row_id in candidate_row_ids {
        if Some(*candidate_row_id) == skipped_row || !visited.insert(*candidate_row_id) {
            continue;
        }

        let existing_row = table.row(*candidate_row_id).ok_or_else(|| DbError::RowNotFound {
            table: table.schema().name().to_owned(),
            row_id: *candidate_row_id,
        })?;
        let existing_subject = SemanticSubject::from_row(table.schema(), existing_row)?;

        match judge.judge(subject, &existing_subject)? {
            SemanticRelation::Compatible(crate::semantic::CompatibleRelation::Generic) => {
                compatible_targets.push(*candidate_row_id)
            }
            SemanticRelation::Compatible(crate::semantic::CompatibleRelation::Nonredundant) => {
                support_targets.push(*candidate_row_id)
            }
            SemanticRelation::Compatible(crate::semantic::CompatibleRelation::Redundant) => {
                compressible_targets.push(*candidate_row_id)
            }
            SemanticRelation::Conflict => conflict_targets.push(*candidate_row_id),
            SemanticRelation::None => {}
        }
    }

    Ok((
        compatible_targets,
        support_targets,
        compressible_targets,
        conflict_targets,
    ))
}

fn apply_semantic_effects(
    table: &mut Table,
    row_id: RowId,
    compatible_targets: &[RowId],
    support_targets: &[RowId],
    compressible_targets: &[RowId],
    conflict_targets: &[RowId],
) -> Result<
    (
        Vec<SemanticEdge>,
        Vec<SemanticEdge>,
        Vec<SemanticEdge>,
        Vec<SemanticEdge>,
        Vec<crate::relation::ConflictId>,
    ),
    DbError,
> {
    let mut compatible_edges = Vec::new();
    let mut support_edges = Vec::new();
    let mut compressible_edges = Vec::new();
    let mut conflict_edges = Vec::new();
    let mut conflict_ids = Vec::new();

    for target in compatible_targets {
        if table.has_compatible_edge(row_id, *target) {
            continue;
        }

        table.add_compatible_edge(row_id, *target)?;
        compatible_edges.push(SemanticEdge::new(row_id, *target));
    }

    for target in support_targets {
        if table.has_support_edge(row_id, *target) {
            continue;
        }

        table.add_support_edge(row_id, *target)?;
        let edge = SemanticEdge::new(row_id, *target);
        compatible_edges.push(edge);
        support_edges.push(edge);
    }

    for target in compressible_targets {
        if table.has_compressible_edge(row_id, *target) {
            continue;
        }

        table.add_compressible_edge(row_id, *target)?;
        let edge = SemanticEdge::new(row_id, *target);
        compatible_edges.push(edge);
        compressible_edges.push(edge);
    }

    for target in conflict_targets {
        if table.has_conflict_edge(row_id, *target) {
            continue;
        }

        table.add_conflict_edge(row_id, *target)?;
        conflict_edges.push(SemanticEdge::new(row_id, *target));
        conflict_ids.push(table.create_conflict_record([row_id, *target])?);
    }

    Ok((
        compatible_edges,
        support_edges,
        compressible_edges,
        conflict_edges,
        conflict_ids,
    ))
}
