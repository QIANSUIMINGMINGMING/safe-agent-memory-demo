use std::collections::{BTreeMap, BTreeSet};

use crate::binding::BindingId;
use crate::db::Database;
use crate::error::DbError;
use crate::projection::{ProjectionMode, ProjectionOutcome, ProjectionPolicy};
use crate::relation::{ConflictStatus, SemanticEdge};
use crate::row::RowId;
use crate::table::Table;

impl Database {
    pub fn project_binding(
        &self,
        binding_id: BindingId,
        policy: ProjectionPolicy,
    ) -> Result<ProjectionOutcome, DbError> {
        let binding_set = self.binding_set(binding_id)?;
        let table = self.table(binding_set.table_name())?;

        let seed_rows = binding_set.rows().clone();
        let mut considered_rows = seed_rows.clone();

        if policy.include_support_neighbors() {
            expand_rows_with_edges(&mut considered_rows, &seed_rows, table.support_edges());
        }
        if policy.include_compressible_neighbors() {
            expand_rows_with_edges(&mut considered_rows, &seed_rows, table.compressible_edges());
        }
        if policy.include_conflict_neighbors() {
            expand_rows_with_edges(&mut considered_rows, &seed_rows, table.conflict_edges());
        }

        match policy.mode() {
            ProjectionMode::Conservative => {
                Ok(project_conservative(binding_id, table, considered_rows))
            }
        }
    }
}

fn expand_rows_with_edges(
    considered_rows: &mut BTreeSet<RowId>,
    seed_rows: &BTreeSet<RowId>,
    edges: &BTreeSet<SemanticEdge>,
) {
    for edge in edges {
        if seed_rows.contains(&edge.first()) {
            considered_rows.insert(edge.second());
        }
        if seed_rows.contains(&edge.second()) {
            considered_rows.insert(edge.first());
        }
    }
}

fn project_conservative(
    binding_id: BindingId,
    table: &Table,
    considered_rows: BTreeSet<RowId>,
) -> ProjectionOutcome {
    let support_scores = build_support_scores(&considered_rows, table.support_edges());
    let mut suppressed_rows = BTreeSet::new();
    let mut ambiguous_rows = BTreeSet::new();
    let mut consulted_conflicts = Vec::new();
    let mut recorded_conflict_edges = BTreeSet::new();

    resolve_compressible_groups(
        &considered_rows,
        table.compressible_edges(),
        &support_scores,
        &mut suppressed_rows,
    );

    let effective_rows: BTreeSet<RowId> = considered_rows
        .iter()
        .copied()
        .filter(|row_id| !suppressed_rows.contains(row_id))
        .collect();

    for (conflict_id, conflict_record) in table.conflict_records() {
        if conflict_record.rows().len() == 2 {
            let mut rows = conflict_record.rows().iter().copied();
            let left = rows.next().expect("two-row conflict has first row");
            let right = rows.next().expect("two-row conflict has second row");
            recorded_conflict_edges.insert(SemanticEdge::new(left, right));
        }

        if conflict_record.status() != ConflictStatus::Open {
            continue;
        }

        let involved_rows: BTreeSet<RowId> = conflict_record
            .rows()
            .iter()
            .copied()
            .filter(|row_id| effective_rows.contains(row_id))
            .collect();

        if involved_rows.len() < 2 {
            continue;
        }

        consulted_conflicts.push(*conflict_id);
        resolve_conflict_group(
            &involved_rows,
            &support_scores,
            &mut suppressed_rows,
            &mut ambiguous_rows,
        );
    }

    for conflict_edge in table.conflict_edges() {
        if recorded_conflict_edges.contains(conflict_edge) {
            continue;
        }
        if !effective_rows.contains(&conflict_edge.first())
            || !effective_rows.contains(&conflict_edge.second())
        {
            continue;
        }

        let involved_rows = BTreeSet::from([conflict_edge.first(), conflict_edge.second()]);
        resolve_conflict_group(
            &involved_rows,
            &support_scores,
            &mut suppressed_rows,
            &mut ambiguous_rows,
        );
    }

    for suppressed_row in &suppressed_rows {
        ambiguous_rows.remove(suppressed_row);
    }

    let accepted_rows: Vec<RowId> = considered_rows
        .iter()
        .copied()
        .filter(|row_id| !suppressed_rows.contains(row_id) && !ambiguous_rows.contains(row_id))
        .collect();

    ProjectionOutcome::new(
        binding_id,
        considered_rows.iter().copied().collect(),
        accepted_rows,
        suppressed_rows.iter().copied().collect(),
        ambiguous_rows.iter().copied().collect(),
        consulted_conflicts,
    )
}

fn resolve_compressible_groups(
    considered_rows: &BTreeSet<RowId>,
    compressible_edges: &BTreeSet<SemanticEdge>,
    support_scores: &BTreeMap<RowId, usize>,
    suppressed_rows: &mut BTreeSet<RowId>,
) {
    let components = compressible_components(considered_rows, compressible_edges);

    for component in components {
        if component.len() < 2 {
            continue;
        }

        let winner = component
            .iter()
            .copied()
            .max_by_key(|row_id| (support_scores.get(row_id).copied().unwrap_or(0), std::cmp::Reverse(row_id.value())))
            .expect("component is non-empty");

        for row_id in &component {
            if *row_id != winner {
                suppressed_rows.insert(*row_id);
            }
        }
    }
}

fn compressible_components(
    considered_rows: &BTreeSet<RowId>,
    compressible_edges: &BTreeSet<SemanticEdge>,
) -> Vec<BTreeSet<RowId>> {
    let mut adjacency: BTreeMap<RowId, BTreeSet<RowId>> = BTreeMap::new();

    for edge in compressible_edges {
        if !considered_rows.contains(&edge.first()) || !considered_rows.contains(&edge.second()) {
            continue;
        }

        adjacency.entry(edge.first()).or_default().insert(edge.second());
        adjacency.entry(edge.second()).or_default().insert(edge.first());
    }

    let mut visited = BTreeSet::new();
    let mut components = Vec::new();

    for row_id in adjacency.keys().copied().collect::<Vec<_>>() {
        if !visited.insert(row_id) {
            continue;
        }

        let mut stack = vec![row_id];
        let mut component = BTreeSet::new();

        while let Some(current) = stack.pop() {
            component.insert(current);
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if visited.insert(*neighbor) {
                        stack.push(*neighbor);
                    }
                }
            }
        }

        components.push(component);
    }

    components
}

fn build_support_scores(
    considered_rows: &BTreeSet<RowId>,
    support_edges: &BTreeSet<SemanticEdge>,
) -> BTreeMap<RowId, usize> {
    let mut scores = BTreeMap::new();

    for edge in support_edges {
        if considered_rows.contains(&edge.first()) && considered_rows.contains(&edge.second()) {
            *scores.entry(edge.first()).or_insert(0) += 1;
            *scores.entry(edge.second()).or_insert(0) += 1;
        }
    }

    scores
}

fn resolve_conflict_group(
    involved_rows: &BTreeSet<RowId>,
    support_scores: &BTreeMap<RowId, usize>,
    suppressed_rows: &mut BTreeSet<RowId>,
    ambiguous_rows: &mut BTreeSet<RowId>,
) {
    let mut max_score = 0usize;
    let mut winners = Vec::new();

    for row_id in involved_rows {
        let score = support_scores.get(row_id).copied().unwrap_or(0);
        if score > max_score {
            max_score = score;
            winners.clear();
            winners.push(*row_id);
        } else if score == max_score {
            winners.push(*row_id);
        }
    }

    if max_score > 0 && winners.len() == 1 {
        let winner = winners[0];
        ambiguous_rows.remove(&winner);
        for row_id in involved_rows {
            if *row_id != winner {
                suppressed_rows.insert(*row_id);
                ambiguous_rows.remove(row_id);
            }
        }
    } else {
        for row_id in involved_rows {
            if !suppressed_rows.contains(row_id) {
                ambiguous_rows.insert(*row_id);
            }
        }
    }
}
