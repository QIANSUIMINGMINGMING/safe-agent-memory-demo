use crate::{
    semantic_insert_action, ColumnRole, ColumnSchema, ConflictStatus, DataType, Database,
    DbError, ExactFilter, FixedSemanticEmbedder, FixedSemanticJudge, ProjectionPolicy, RowInput,
    SchemaError, SemanticAction, SemanticIndexError, SemanticJudge, SemanticJudgeError,
    SemanticRelation, SemanticScope, SemanticSubject, TableSchema, Value,
};

fn project_table_schema() -> TableSchema {
    TableSchema::new(
        "project_notes",
        vec![
            ColumnSchema::new("project_id", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("topic", DataType::Text, ColumnRole::Semantic).unwrap(),
            ColumnSchema::new("statement", DataType::Text, ColumnRole::Semantic).unwrap(),
            ColumnSchema::new("archived", DataType::Bool, ColumnRole::Normal).unwrap(),
        ],
        Some(SemanticScope::new(["topic", "statement"]).unwrap()),
    )
    .unwrap()
}

fn design_claims_table_schema() -> TableSchema {
    TableSchema::new(
        "design_claims",
        vec![
            ColumnSchema::new("project", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("topic", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("kind", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("source_turn", DataType::Int64, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("claim", DataType::Text, ColumnRole::Semantic).unwrap(),
        ],
        Some(SemanticScope::new(["claim"]).unwrap()),
    )
    .unwrap()
}

fn persist_project_note(
    db: &mut Database,
    project_id: &str,
    topic: &str,
    statement: &str,
    archived: bool,
) -> crate::row::RowId {
    db.persist(
        "project_notes",
        RowInput::new()
            .with_value("project_id", Value::Text(project_id.to_owned()))
            .with_value("topic", Value::Text(topic.to_owned()))
            .with_value("statement", Value::Text(statement.to_owned()))
            .with_value("archived", Value::Bool(archived)),
    )
    .unwrap()
    .row_id()
}

fn persist_design_claim(
    db: &mut Database,
    kind: &str,
    source_turn: i64,
    claim: &str,
) -> crate::row::RowId {
    db.persist(
        "design_claims",
        RowInput::new()
            .with_value("project", Value::Text("agentdb".to_owned()))
            .with_value("topic", Value::Text("semantic-core".to_owned()))
            .with_value("kind", Value::Text(kind.to_owned()))
            .with_value("source_turn", Value::Int64(source_turn))
            .with_value("claim", Value::Text(claim.to_owned())),
    )
    .unwrap()
    .row_id()
}

fn persist_semantic_design_claim(
    db: &mut Database,
    judge: &impl SemanticJudge,
    kind: &str,
    source_turn: i64,
    claim: &str,
) -> crate::persist::PersistOutcome {
    db.persist_semantic(
        "design_claims",
        RowInput::new()
            .with_value("project", Value::Text("agentdb".to_owned()))
            .with_value("topic", Value::Text("semantic-core".to_owned()))
            .with_value("kind", Value::Text(kind.to_owned()))
            .with_value("source_turn", Value::Int64(source_turn))
            .with_value("claim", Value::Text(claim.to_owned())),
        judge,
    )
    .unwrap()
}

fn v1_design_claims_judge() -> FixedSemanticJudge {
    FixedSemanticJudge::default()
        .with_pair_text(
            "claim=Strict atomicity should be a hard invariant for semantic rows.",
            "claim=Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
            SemanticRelation::conflict(),
        )
        .with_pair_text(
            "claim=Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
            "claim=Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
            SemanticRelation::support(),
        )
        .with_pair_text(
            "claim=Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
            "claim=Projection-time resolution should create a task-local view without mutating base storage by default.",
            SemanticRelation::support(),
        )
        .with_pair_text(
            "claim=Projection-time resolution should create a task-local view without mutating base storage by default.",
            "claim=Vector retrieval is an execution strategy for semantic access, not the core data model.",
            SemanticRelation::support(),
        )
        .with_pair_text(
            "claim=Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
            "claim=Persisted support and conflict relations should be first-class state in the kernel.",
            SemanticRelation::support(),
        )
        .with_pair_text(
            "claim=Projection-time resolution should create a task-local view without mutating base storage by default.",
            "claim=Persisted support and conflict relations should be first-class state in the kernel.",
            SemanticRelation::support(),
        )
}

fn v1_design_claims_embedder() -> FixedSemanticEmbedder {
    FixedSemanticEmbedder::new()
        .with_text_embedding(
            "claim=Strict atomicity should be a hard invariant for semantic rows.",
            vec![1.0, 0.0, 0.0],
        )
        .with_text_embedding(
            "claim=Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
            vec![0.96, 0.04, 0.0],
        )
        .with_text_embedding(
            "claim=Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
            vec![0.0, 1.0, 0.0],
        )
        .with_text_embedding(
            "claim=Projection-time resolution should create a task-local view without mutating base storage by default.",
            vec![0.0, 0.95, 0.05],
        )
        .with_text_embedding(
            "claim=Vector retrieval is an execution strategy for semantic access, not the core data model.",
            vec![0.0, 0.7, 0.3],
        )
        .with_text_embedding(
            "claim=Persisted support and conflict relations should be first-class state in the kernel.",
            vec![0.0, 0.9, 0.1],
        )
        .with_text_embedding(
            "claim=Projection-time resolution should create a coherent task-local view over preserved semantic state.",
            vec![0.0, 0.94, 0.06],
        )
}

fn populate_v1_design_claims_semantic_state(
    db: &mut Database,
) -> (
    crate::row::RowId,
    crate::row::RowId,
    crate::row::RowId,
    crate::row::RowId,
    crate::row::RowId,
    crate::row::RowId,
) {
    let judge = v1_design_claims_judge();

    let row_a = persist_design_claim(
        db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );

    let row_b = persist_semantic_design_claim(
        db,
        &judge,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    )
    .row_id();

    let row_c = persist_semantic_design_claim(
        db,
        &judge,
        "design-decision",
        3,
        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
    )
    .row_id();

    let row_d = persist_semantic_design_claim(
        db,
        &judge,
        "design-decision",
        4,
        "Projection-time resolution should create a task-local view without mutating base storage by default.",
    )
    .row_id();

    let row_e = persist_semantic_design_claim(
        db,
        &judge,
        "architecture-note",
        5,
        "Vector retrieval is an execution strategy for semantic access, not the core data model.",
    )
    .row_id();

    let row_f = persist_semantic_design_claim(
        db,
        &judge,
        "supporting-note",
        6,
        "Persisted support and conflict relations should be first-class state in the kernel.",
    )
    .row_id();

    (row_a, row_b, row_c, row_d, row_e, row_f)
}

#[test]
fn creates_table_persists_row_and_lists_rows() {
    let schema = project_table_schema();
    assert_eq!(schema.columns()[1].role(), ColumnRole::Semantic);
    assert_eq!(
        schema.semantic_scope().unwrap().columns(),
        &["topic".to_owned(), "statement".to_owned()]
    );

    let mut db = Database::new();
    db.create_table(schema).unwrap();
    let row_id = persist_project_note(&mut db, "agentdb", "schema", "step 1 stores rows", false);

    let rows = db.named_rows("project_notes").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].get("statement"),
        Some(&Value::Text("step 1 stores rows".to_owned()))
    );

    let row = db.row("project_notes", row_id).unwrap();
    assert_eq!(row.id().value(), 1);
    assert_eq!(
        row.get(db.table("project_notes").unwrap().schema(), "statement"),
        Some(&Value::Text("step 1 stores rows".to_owned()))
    );
}

#[test]
fn rejects_duplicate_column_names() {
    let error = TableSchema::new(
        "dup_columns",
        vec![
            ColumnSchema::new("topic", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("topic", DataType::Text, ColumnRole::Semantic).unwrap(),
        ],
        Some(SemanticScope::new(["topic"]).unwrap()),
    )
    .unwrap_err();

    assert_eq!(error, SchemaError::DuplicateColumnName("topic".to_owned()));
}

#[test]
fn rejects_missing_semantic_scope_when_semantic_columns_exist() {
    let error = TableSchema::new(
        "project_notes",
        vec![
            ColumnSchema::new("project_id", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("statement", DataType::Text, ColumnRole::Semantic).unwrap(),
        ],
        None,
    )
    .unwrap_err();

    assert_eq!(error, SchemaError::MissingSemanticScope);
}

#[test]
fn rejects_semantic_scope_on_normal_columns() {
    let error = TableSchema::new(
        "design_claims",
        vec![
            ColumnSchema::new("project", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("claim", DataType::Text, ColumnRole::Semantic).unwrap(),
        ],
        Some(SemanticScope::new(["project"]).unwrap()),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SchemaError::NonSemanticScopeColumn {
            column: "project".to_owned(),
            role: ColumnRole::Normal,
        }
    );
}

#[test]
fn accepts_single_claim_semantic_scope_for_v1_testcase() {
    let schema = design_claims_table_schema();

    assert_eq!(schema.name(), "design_claims");
    assert_eq!(schema.semantic_scope().unwrap().columns(), &["claim".to_owned()]);
}

#[test]
fn allows_normal_only_tables_without_semantic_scope() {
    let schema = TableSchema::new(
        "turn_index",
        vec![
            ColumnSchema::new("project", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("turn", DataType::Int64, ColumnRole::Normal).unwrap(),
        ],
        None,
    )
    .unwrap();

    assert!(schema.semantic_scope().is_none());
}

#[test]
fn semantic_subject_extracts_values_from_scope_in_scope_order() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    let row_id = persist_project_note(
        &mut db,
        "agentdb",
        "semantic-core",
        "projection-time resolution preserves base storage",
        false,
    );

    let table = db.table("project_notes").unwrap();
    let row = db.row("project_notes", row_id).unwrap();
    let subject = SemanticSubject::from_row(table.schema(), row).unwrap();

    assert_eq!(subject.fields().len(), 2);
    assert_eq!(subject.fields()[0].column(), "topic");
    assert_eq!(subject.fields()[0].value(), "semantic-core");
    assert_eq!(subject.fields()[1].column(), "statement");
    assert_eq!(
        subject.fields()[1].value(),
        "projection-time resolution preserves base storage"
    );
    assert_eq!(
        subject.canonical_text(),
        "topic=semantic-core\nstatement=projection-time resolution preserves base storage"
    );
}

#[test]
fn semantic_subject_requires_semantic_scope() {
    let schema = TableSchema::new(
        "turn_index",
        vec![
            ColumnSchema::new("project", DataType::Text, ColumnRole::Normal).unwrap(),
            ColumnSchema::new("turn", DataType::Int64, ColumnRole::Normal).unwrap(),
        ],
        None,
    )
    .unwrap();

    let row = crate::row::Row::new(
        crate::row::RowId::new(1),
        vec![Value::Text("agentdb".to_owned()), Value::Int64(1)],
    );

    let error = SemanticSubject::from_row(&schema, &row).unwrap_err();
    assert_eq!(error, SemanticJudgeError::NoSemanticScopeDefined);
}

#[test]
fn fixed_semantic_judge_matches_v1_design_claims_fixture() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );

    let row_b = persist_design_claim(
        &mut db,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    );

    let row_c = persist_design_claim(
        &mut db,
        "design-decision",
        3,
        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
    );

    let row_d = persist_design_claim(
        &mut db,
        "design-decision",
        4,
        "Projection-time resolution should create a task-local view without mutating base storage by default.",
    );

    let row_e = persist_design_claim(
        &mut db,
        "architecture-note",
        5,
        "Vector retrieval is an execution strategy for semantic access, not the core data model.",
    );

    let row_f = persist_design_claim(
        &mut db,
        "supporting-note",
        6,
        "Persisted support and conflict relations should be first-class state in the kernel.",
    );

    let table = db.table("design_claims").unwrap();
    let subject_a = SemanticSubject::from_row(table.schema(), db.row("design_claims", row_a).unwrap()).unwrap();
    let subject_b = SemanticSubject::from_row(table.schema(), db.row("design_claims", row_b).unwrap()).unwrap();
    let subject_c = SemanticSubject::from_row(table.schema(), db.row("design_claims", row_c).unwrap()).unwrap();
    let subject_d = SemanticSubject::from_row(table.schema(), db.row("design_claims", row_d).unwrap()).unwrap();
    let subject_e = SemanticSubject::from_row(table.schema(), db.row("design_claims", row_e).unwrap()).unwrap();
    let subject_f = SemanticSubject::from_row(table.schema(), db.row("design_claims", row_f).unwrap()).unwrap();

    let judge = FixedSemanticJudge::default()
        .with_pair(
            &subject_a,
            &subject_b,
            SemanticRelation::conflict(),
        )
        .with_pair(
            &subject_b,
            &subject_c,
            SemanticRelation::support(),
        )
        .with_pair(
            &subject_c,
            &subject_d,
            SemanticRelation::support(),
        )
        .with_pair(
            &subject_d,
            &subject_e,
            SemanticRelation::support(),
        )
        .with_pair(
            &subject_c,
            &subject_f,
            SemanticRelation::support(),
        )
        .with_pair(
            &subject_d,
            &subject_f,
            SemanticRelation::support(),
        );

    assert_eq!(
        judge.judge(&subject_a, &subject_b).unwrap(),
        SemanticRelation::conflict()
    );
    assert_eq!(
        judge.judge(&subject_b, &subject_c).unwrap(),
        SemanticRelation::support()
    );
    assert_eq!(
        judge.judge(&subject_c, &subject_d).unwrap(),
        SemanticRelation::support()
    );
    assert_eq!(
        judge.judge(&subject_d, &subject_e).unwrap(),
        SemanticRelation::support()
    );
    assert_eq!(
        judge.judge(&subject_c, &subject_f).unwrap(),
        SemanticRelation::support()
    );
    assert_eq!(
        judge.judge(&subject_a, &subject_f).unwrap(),
        SemanticRelation::none()
    );
}

#[test]
fn semantic_persist_automatically_records_relations_from_judge() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let judge = v1_design_claims_judge();

    let row_a = persist_design_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );

    let outcome_b = persist_semantic_design_claim(
        &mut db,
        &judge,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    );

    let row_b = outcome_b.row_id();
    assert_eq!(outcome_b.support_edges().len(), 0);
    assert_eq!(outcome_b.compressible_edges().len(), 0);
    assert_eq!(outcome_b.conflict_edges().len(), 1);
    assert_eq!(outcome_b.conflict_ids().len(), 1);
    assert!(outcome_b.conflict_edges()[0].contains(row_a));
    assert!(outcome_b.conflict_edges()[0].contains(row_b));

    let outcome_c = persist_semantic_design_claim(
        &mut db,
        &judge,
        "design-decision",
        3,
        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
    );
    let row_c = outcome_c.row_id();
    assert_eq!(outcome_c.support_edges().len(), 1);
    assert_eq!(outcome_c.compressible_edges().len(), 0);
    assert_eq!(outcome_c.conflict_edges().len(), 0);
    assert!(outcome_c.support_edges()[0].contains(row_b));
    assert!(outcome_c.support_edges()[0].contains(row_c));

    let outcome_d = persist_semantic_design_claim(
        &mut db,
        &judge,
        "design-decision",
        4,
        "Projection-time resolution should create a task-local view without mutating base storage by default.",
    );
    let row_d = outcome_d.row_id();
    assert_eq!(outcome_d.support_edges().len(), 1);
    assert_eq!(outcome_d.compressible_edges().len(), 0);
    assert!(outcome_d.support_edges()[0].contains(row_c));
    assert!(outcome_d.support_edges()[0].contains(row_d));

    let outcome_e = persist_semantic_design_claim(
        &mut db,
        &judge,
        "architecture-note",
        5,
        "Vector retrieval is an execution strategy for semantic access, not the core data model.",
    );
    let row_e = outcome_e.row_id();
    assert_eq!(outcome_e.support_edges().len(), 1);
    assert_eq!(outcome_e.compressible_edges().len(), 0);
    assert!(outcome_e.support_edges()[0].contains(row_d));
    assert!(outcome_e.support_edges()[0].contains(row_e));

    let outcome_f = persist_semantic_design_claim(
        &mut db,
        &judge,
        "supporting-note",
        6,
        "Persisted support and conflict relations should be first-class state in the kernel.",
    );
    let row_f = outcome_f.row_id();
    assert_eq!(outcome_f.support_edges().len(), 2);
    assert_eq!(outcome_f.compressible_edges().len(), 0);
    assert_eq!(outcome_f.conflict_edges().len(), 0);
    assert!(outcome_f.support_edges().iter().any(|edge| edge.contains(row_c) && edge.contains(row_f)));
    assert!(outcome_f.support_edges().iter().any(|edge| edge.contains(row_d) && edge.contains(row_f)));

    assert_eq!(db.support_edges("design_claims").unwrap().len(), 5);
    assert_eq!(db.conflict_edges("design_claims").unwrap().len(), 1);
    assert_eq!(db.conflict_records("design_claims").unwrap().len(), 1);
}

#[test]
fn semantic_persist_records_compressible_edges_from_judge() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let judge = FixedSemanticJudge::default().with_pair_text(
        "claim=We made a decision to validate the Maya implementation before evaluation.",
        "claim=We have a todo to validate the Maya implementation before evaluation.",
        SemanticRelation::compressible(),
    );

    let existing_row = persist_design_claim(
        &mut db,
        "decision",
        1,
        "We have a todo to validate the Maya implementation before evaluation.",
    );

    let outcome = persist_semantic_design_claim(
        &mut db,
        &judge,
        "decision",
        2,
        "We made a decision to validate the Maya implementation before evaluation.",
    );

    assert!(outcome.support_edges().is_empty());
    assert_eq!(outcome.compressible_edges().len(), 1);
    assert!(outcome.conflict_edges().is_empty());
    assert!(outcome.conflict_ids().is_empty());
    assert!(outcome.compressible_edges()[0].contains(existing_row));
    assert!(outcome.compressible_edges()[0].contains(outcome.row_id()));

    let compressible_edges = db.compressible_edges("design_claims").unwrap();
    assert_eq!(compressible_edges.len(), 1);
    let compressible_edge = compressible_edges.iter().next().unwrap();
    assert!(compressible_edge.contains(existing_row));
    assert!(compressible_edge.contains(outcome.row_id()));
}

#[test]
fn semantic_persist_can_prefilter_candidates_by_normal_columns() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let judge = FixedSemanticJudge::default().with_pair_text(
        "claim=Candidate row",
        "claim=Existing row",
        SemanticRelation::support(),
    );

    let existing_row =
        persist_design_claim(&mut db, "existing", 1, "Existing row");

    let outcome = db
        .persist_semantic_with_filters(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("other-project".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("candidate".to_owned()))
                .with_value("source_turn", Value::Int64(2))
                .with_value("claim", Value::Text("Candidate row".to_owned())),
            &[ExactFilter::new("project", Value::Text("other-project".to_owned())).unwrap()],
            &judge,
        )
        .unwrap();

    assert_eq!(outcome.support_edges().len(), 0);
    assert_eq!(outcome.conflict_edges().len(), 0);
    assert_eq!(db.support_edges("design_claims").unwrap().len(), 0);
    assert!(db.row("design_claims", existing_row).is_ok());
}

#[test]
fn binding_sets_can_bind_remove_and_list_rows() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    );
    let row_c = persist_design_claim(
        &mut db,
        "design-decision",
        3,
        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
    );

    let binding_id = db.create_binding_set("design_claims").unwrap();
    assert_eq!(binding_id.value(), 1);

    assert!(db.bind_row(binding_id, row_a).unwrap());
    assert!(db.bind_row(binding_id, row_b).unwrap());
    assert!(db.bind_row(binding_id, row_c).unwrap());
    assert!(!db.bind_row(binding_id, row_b).unwrap());

    let binding_set = db.binding_set(binding_id).unwrap();
    assert_eq!(binding_set.table_name(), "design_claims");
    assert_eq!(binding_set.rows().len(), 3);
    assert!(binding_set.contains(row_a));
    assert!(binding_set.contains(row_b));
    assert!(binding_set.contains(row_c));

    assert!(db.unbind_row(binding_id, row_b).unwrap());
    assert!(!db.unbind_row(binding_id, row_b).unwrap());

    let updated_binding_set = db.binding_set(binding_id).unwrap();
    assert_eq!(updated_binding_set.rows().len(), 2);
    assert!(updated_binding_set.contains(row_a));
    assert!(!updated_binding_set.contains(row_b));
    assert!(updated_binding_set.contains(row_c));
}

#[test]
fn binding_sets_require_existing_table_and_rows() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let missing_table_error = db.create_binding_set("unknown_table").unwrap_err();
    assert_eq!(missing_table_error, DbError::TableNotFound("unknown_table".to_owned()));

    let binding_id = db.create_binding_set("design_claims").unwrap();
    let missing_row_error = db.bind_row(binding_id, crate::row::RowId::new(999)).unwrap_err();
    assert_eq!(
        missing_row_error,
        DbError::RowNotFound {
            table: "design_claims".to_owned(),
            row_id: crate::row::RowId::new(999),
        }
    );
}

#[test]
fn binding_set_lookup_requires_known_id() {
    let db = Database::new();

    let error = db.binding_set(crate::binding::BindingId::new(999)).unwrap_err();
    assert_eq!(
        error,
        DbError::BindingSetNotFound(crate::binding::BindingId::new(999))
    );
}

#[test]
fn conservative_projection_suppresses_weaker_conflict_rows() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let (row_a, row_b, row_c, row_d, row_e, row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let binding_id = db.create_binding_set("design_claims").unwrap();
    db.bind_row(binding_id, row_b).unwrap();
    db.bind_row(binding_id, row_c).unwrap();
    db.bind_row(binding_id, row_d).unwrap();
    db.bind_row(binding_id, row_e).unwrap();
    db.bind_row(binding_id, row_f).unwrap();

    let projection = db
        .project_binding(binding_id, ProjectionPolicy::conservative())
        .unwrap();

    assert_eq!(projection.binding_id().value(), binding_id.value());
    assert!(projection.considered_rows().contains(&row_a));
    assert!(projection.considered_rows().contains(&row_b));
    assert!(projection.considered_rows().contains(&row_c));
    assert!(projection.considered_rows().contains(&row_d));
    assert!(projection.considered_rows().contains(&row_e));
    assert!(projection.considered_rows().contains(&row_f));

    assert_eq!(projection.accepted_rows(), &[row_b, row_c, row_d, row_e, row_f]);
    assert_eq!(projection.suppressed_rows(), &[row_a]);
    assert!(projection.ambiguous_rows().is_empty());
    assert_eq!(projection.consulted_conflicts().len(), 1);
}

#[test]
fn conservative_projection_suppresses_compressible_rows_deterministically() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "todo",
        1,
        "We have a todo to validate the Maya implementation before evaluation.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "decision",
        2,
        "We made a decision to validate the Maya implementation before evaluation.",
    );
    let row_c = persist_design_claim(
        &mut db,
        "finding",
        3,
        "The current Maya implementation still needs evaluation-focused tests.",
    );

    db.add_compressible_edge("design_claims", row_a, row_b).unwrap();
    db.add_support_edge("design_claims", row_b, row_c).unwrap();

    let binding_id = db.create_binding_set("design_claims").unwrap();
    db.bind_row(binding_id, row_a).unwrap();
    db.bind_row(binding_id, row_b).unwrap();

    let projection = db
        .project_binding(binding_id, ProjectionPolicy::conservative())
        .unwrap();

    assert_eq!(projection.considered_rows(), &[row_a, row_b, row_c]);
    assert_eq!(projection.accepted_rows(), &[row_b, row_c]);
    assert_eq!(projection.suppressed_rows(), &[row_a]);
    assert!(projection.ambiguous_rows().is_empty());
    assert!(projection.consulted_conflicts().is_empty());
}

#[test]
fn conservative_projection_ignores_resolved_conflicts() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let (row_a, row_b, row_c, ..) = populate_v1_design_claims_semantic_state(&mut db);

    let conflict_id = *db
        .conflict_records("design_claims")
        .unwrap()
        .keys()
        .next()
        .expect("expected one conflict record");
    db.set_conflict_status("design_claims", conflict_id, ConflictStatus::Resolved)
        .unwrap();

    let binding_id = db.create_binding_set("design_claims").unwrap();
    db.bind_row(binding_id, row_b).unwrap();

    let projection = db
        .project_binding(binding_id, ProjectionPolicy::conservative())
        .unwrap();

    assert!(projection.considered_rows().contains(&row_a));
    assert!(projection.considered_rows().contains(&row_b));
    assert!(projection.considered_rows().contains(&row_c));
    assert_eq!(projection.accepted_rows(), &[row_a, row_b, row_c]);
    assert!(projection.suppressed_rows().is_empty());
    assert!(projection.ambiguous_rows().is_empty());
    assert!(projection.consulted_conflicts().is_empty());
}

#[test]
fn traversal_helpers_return_support_and_conflict_neighbors() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let (row_a, row_b, row_c, row_d, _row_e, row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let conflict_neighbors_b = db.conflict_neighbors("design_claims", row_b).unwrap();
    assert_eq!(conflict_neighbors_b, std::collections::BTreeSet::from([row_a]));

    let support_neighbors_c = db.support_neighbors("design_claims", row_c).unwrap();
    assert_eq!(
        support_neighbors_c,
        std::collections::BTreeSet::from([row_b, row_d, row_f])
    );

    let support_neighbors_d = db.support_neighbors("design_claims", row_d).unwrap();
    assert_eq!(
        support_neighbors_d,
        std::collections::BTreeSet::from([row_c, _row_e, row_f])
    );
}

#[test]
fn traversal_helpers_return_compressible_neighbors() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "todo",
        1,
        "We have a todo to validate the Maya implementation before evaluation.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "decision",
        2,
        "We made a decision to validate the Maya implementation before evaluation.",
    );

    db.add_compressible_edge("design_claims", row_a, row_b).unwrap();

    let compressible_neighbors = db.compressible_neighbors("design_claims", row_b).unwrap();
    assert_eq!(compressible_neighbors, std::collections::BTreeSet::from([row_a]));
}

#[test]
fn traversal_helpers_return_generic_compatible_neighbors() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "finding",
        1,
        "The Maya implementation still needs paper-aligned verification before evaluation.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "question",
        2,
        "We need to clarify which output checks belong in the Maya validation suite.",
    );

    db.add_compatible_edge("design_claims", row_a, row_b).unwrap();

    let compatible_neighbors = db.compatible_neighbors("design_claims", row_b).unwrap();
    assert_eq!(compatible_neighbors, std::collections::BTreeSet::from([row_a]));
}

#[test]
fn traversal_helpers_require_existing_rows() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let error = db
        .support_neighbors("design_claims", crate::row::RowId::new(999))
        .unwrap_err();
    assert_eq!(
        error,
        DbError::RowNotFound {
            table: "design_claims".to_owned(),
            row_id: crate::row::RowId::new(999),
        }
    );
}

#[test]
fn stores_support_and_conflict_edges_as_first_class_relation_state() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );

    let row_b = persist_design_claim(
        &mut db,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    );

    let row_c = persist_design_claim(
        &mut db,
        "design-decision",
        3,
        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
    );

    db.add_conflict_edge("design_claims", row_a, row_b).unwrap();
    db.add_support_edge("design_claims", row_b, row_c).unwrap();

    let support_edges = db.support_edges("design_claims").unwrap();
    assert_eq!(support_edges.len(), 1);
    let support_edge = support_edges.iter().next().unwrap();
    assert_eq!(support_edge.first().value(), row_b.value());
    assert_eq!(support_edge.second().value(), row_c.value());

    let conflict_edges = db.conflict_edges("design_claims").unwrap();
    assert_eq!(conflict_edges.len(), 1);
    let conflict_edge = conflict_edges.iter().next().unwrap();
    assert!(conflict_edge.contains(row_a));
    assert!(conflict_edge.contains(row_b));
}

#[test]
fn stores_compressible_edges_as_first_class_relation_state() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "todo",
        1,
        "We have a todo to validate the Maya implementation before evaluation.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "decision",
        2,
        "We made a decision to validate the Maya implementation before evaluation.",
    );

    db.add_compressible_edge("design_claims", row_a, row_b).unwrap();

    let compressible_edges = db.compressible_edges("design_claims").unwrap();
    assert_eq!(compressible_edges.len(), 1);
    let compressible_edge = compressible_edges.iter().next().unwrap();
    assert!(compressible_edge.contains(row_a));
    assert!(compressible_edge.contains(row_b));
}

#[test]
fn stores_generic_compatible_edges_as_first_class_relation_state() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "finding",
        1,
        "The Maya implementation still needs paper-aligned verification before evaluation.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "question",
        2,
        "We need to clarify which output checks belong in the Maya validation suite.",
    );

    db.add_compatible_edge("design_claims", row_a, row_b).unwrap();

    let compatible_edges = db.compatible_edges("design_claims").unwrap();
    assert_eq!(compatible_edges.len(), 1);
    let compatible_edge = compatible_edges.iter().next().unwrap();
    assert!(compatible_edge.contains(row_a));
    assert!(compatible_edge.contains(row_b));
    assert!(db.support_edges("design_claims").unwrap().is_empty());
    assert!(db.compressible_edges("design_claims").unwrap().is_empty());
}

#[test]
fn creates_and_updates_conflict_records() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );

    let row_b = persist_design_claim(
        &mut db,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    );

    let conflict_id = db
        .create_conflict_record("design_claims", [row_a, row_b])
        .unwrap();
    let conflict = db.conflict_record("design_claims", conflict_id).unwrap();

    assert_eq!(conflict.id().value(), 1);
    assert_eq!(conflict.status(), ConflictStatus::Open);
    assert!(conflict.rows().contains(&row_a));
    assert!(conflict.rows().contains(&row_b));

    db.set_conflict_status("design_claims", conflict_id, ConflictStatus::Resolved)
        .unwrap();

    let updated_conflict = db.conflict_record("design_claims", conflict_id).unwrap();
    assert_eq!(updated_conflict.status(), ConflictStatus::Resolved);
}

#[test]
fn rejects_relation_edges_for_missing_or_identical_rows() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let row_a = persist_design_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );

    let identical_error = db
        .add_support_edge("design_claims", row_a, row_a)
        .unwrap_err();
    assert_eq!(
        identical_error,
        DbError::InvalidSemanticEdge {
            table: "design_claims".to_owned(),
            left: row_a,
            right: row_a,
        }
    );

    let missing_error = db
        .add_conflict_edge("design_claims", row_a, crate::row::RowId::new(999))
        .unwrap_err();
    assert_eq!(
        missing_error,
        DbError::InvalidSemanticEdge {
            table: "design_claims".to_owned(),
            left: row_a,
            right: crate::row::RowId::new(999),
        }
    );
}

#[test]
fn rejects_missing_columns_on_insert() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    let error = db
        .persist(
            "project_notes",
            RowInput::new()
                .with_value("project_id", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("schema".to_owned()))
                .with_value("archived", Value::Bool(false)),
        )
        .unwrap_err();

    assert_eq!(
        error,
        DbError::MissingColumn {
            table: "project_notes".to_owned(),
            column: "statement".to_owned(),
        }
    );
}

#[test]
fn rejects_type_mismatch_on_insert() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    let error = db
        .persist(
            "project_notes",
            RowInput::new()
                .with_value("project_id", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("schema".to_owned()))
                .with_value("statement", Value::Bool(true))
                .with_value("archived", Value::Bool(false)),
        )
        .unwrap_err();

    assert_eq!(
        error,
        DbError::TypeMismatch {
            table: "project_notes".to_owned(),
            column: "statement".to_owned(),
            expected: DataType::Text,
            found: DataType::Bool,
        }
    );
}

#[test]
fn scans_rows_by_normal_columns() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    persist_project_note(&mut db, "agentdb", "schema", "bootstrap kernel", false);
    persist_project_note(&mut db, "agentdb", "semantic", "not active yet", true);
    persist_project_note(&mut db, "other", "schema", "other project", false);

    let rows = db
        .scan_exact(
            "project_notes",
            &[
                ExactFilter::new("project_id", Value::Text("agentdb".to_owned())).unwrap(),
                ExactFilter::new("archived", Value::Bool(false)).unwrap(),
            ],
        )
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].get("statement"),
        Some(&Value::Text("bootstrap kernel".to_owned()))
    );
}

#[test]
fn rejects_exact_scan_on_semantic_columns() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    let error = db
        .scan_exact(
            "project_notes",
            &[ExactFilter::new("topic", Value::Text("schema".to_owned())).unwrap()],
        )
        .unwrap_err();

    assert_eq!(
        error,
        DbError::ExactFilterOnNonNormalColumn {
            table: "project_notes".to_owned(),
            column: "topic".to_owned(),
            role: ColumnRole::Semantic,
        }
    );
}

#[test]
fn row_ids_are_stable_and_can_be_fetched_directly() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    let first_row_id = persist_project_note(&mut db, "agentdb", "schema", "first row", false);
    let second_row_id = persist_project_note(&mut db, "agentdb", "semantic", "second row", false);

    assert_eq!(first_row_id.value(), 1);
    assert_eq!(second_row_id.value(), 2);

    let first_row = db.row("project_notes", first_row_id).unwrap();
    let second_row = db.row("project_notes", second_row_id).unwrap();
    let schema = db.table("project_notes").unwrap().schema();

    assert_eq!(
        first_row.get(schema, "statement"),
        Some(&Value::Text("first row".to_owned()))
    );
    assert_eq!(
        second_row.get(schema, "statement"),
        Some(&Value::Text("second row".to_owned()))
    );
}

#[test]
fn semantic_policy_inserts_when_no_conflict_exists() {
    let action = semantic_insert_action(&[
        SemanticRelation::support(),
        SemanticRelation::compressible(),
        SemanticRelation::none(),
    ]);

    assert_eq!(action, SemanticAction::Insert);
}

#[test]
fn semantic_policy_blocks_when_any_conflict_exists() {
    let action = semantic_insert_action(&[
        SemanticRelation::none(),
        SemanticRelation::conflict(),
        SemanticRelation::support(),
    ]);

    assert_eq!(action, SemanticAction::Block);
}

#[test]
fn persist_returns_row_id_and_no_semantic_effects_for_plain_write() {
    let mut db = Database::new();
    db.create_table(project_table_schema()).unwrap();

    let outcome = db
        .persist(
            "project_notes",
            RowInput::new()
                .with_value("project_id", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("schema".to_owned()))
                .with_value("statement", Value::Text("persist surface".to_owned()))
                .with_value("archived", Value::Bool(false)),
        )
        .unwrap();

    assert_eq!(outcome.row_id().value(), 1);
    assert!(!outcome.has_semantic_effects());
    assert!(outcome.support_edges().is_empty());
    assert!(outcome.conflict_edges().is_empty());
    assert!(outcome.conflict_ids().is_empty());
}

#[test]
fn persist_semantic_returns_same_outcome_shape_as_plain_persist_plus_relations() {
    let input = RowInput::new()
        .with_value("project", Value::Text("agentdb".to_owned()))
        .with_value("topic", Value::Text("semantic-core".to_owned()))
        .with_value("kind", Value::Text("draft-conclusion".to_owned()))
        .with_value("source_turn", Value::Int64(1))
        .with_value(
            "claim",
            Value::Text("Strict atomicity should be a hard invariant for semantic rows.".to_owned()),
        );
    let judge = FixedSemanticJudge::default();

    let mut db_from_persist = Database::new();
    db_from_persist
        .create_table(design_claims_table_schema())
        .unwrap();
    let persist_outcome = db_from_persist
        .persist_semantic("design_claims", input.clone(), &judge)
        .unwrap();
    let plain_row = db_from_persist
        .persist("design_claims", input)
        .unwrap()
        .row_id();

    assert_eq!(persist_outcome.row_id().value(), 1);
    assert_eq!(plain_row.value(), 2);
    assert!(persist_outcome.support_edges().is_empty());
    assert!(persist_outcome.conflict_edges().is_empty());
    assert!(persist_outcome.conflict_ids().is_empty());
}

#[test]
fn rebuilds_semantic_index_and_searches_canonical_scope_text() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let (_row_a, _row_b, row_c, row_d, row_e, row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let embedder = v1_design_claims_embedder();
    db.rebuild_semantic_index("design_claims", &embedder).unwrap();

    let index = db.semantic_index("design_claims").unwrap();
    assert_eq!(index.len(), 6);
    assert_eq!(index.dimension(), 3);

    let hits = db
        .search_semantic(
            "design_claims",
            "claim=Projection-time resolution should create a coherent task-local view over preserved semantic state.",
            4,
            &embedder,
        )
        .unwrap();

    assert_eq!(hits.len(), 4);
    assert_eq!(hits[0].row_id(), row_d);
    assert_eq!(hits[1].row_id(), row_f);
    assert_eq!(hits[2].row_id(), row_c);
    assert_eq!(hits[3].row_id(), row_e);
    assert!(hits[0].score() >= hits[1].score());
    assert!(hits[1].score() >= hits[2].score());
    assert!(hits[2].score() >= hits[3].score());
}

#[test]
fn semantic_neighbors_query_existing_indexed_row_and_exclude_self() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let (_row_a, _row_b, row_c, row_d, row_e, row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let embedder = v1_design_claims_embedder();
    db.rebuild_semantic_index("design_claims", &embedder).unwrap();

    let neighbors = db.semantic_neighbors("design_claims", row_d, 3).unwrap();

    assert_eq!(neighbors.len(), 3);
    assert_eq!(neighbors[0].row_id(), row_c);
    assert_eq!(neighbors[1].row_id(), row_f);
    assert_eq!(neighbors[2].row_id(), row_e);
    assert!(neighbors.iter().all(|hit| hit.row_id() != row_d));
}

#[test]
fn semantic_search_rows_limits_hits_to_explicit_candidate_subset() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let (_row_a, _row_b, row_c, _row_d, _row_e, row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let embedder = v1_design_claims_embedder();
    db.rebuild_semantic_index("design_claims", &embedder).unwrap();

    let hits = db
        .search_semantic_rows(
            "design_claims",
            &[row_c, row_f],
            "claim=Vector retrieval is an execution strategy for semantic access, not the core data model.",
            4,
            &embedder,
        )
        .unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].row_id(), row_f);
    assert_eq!(hits[1].row_id(), row_c);
}

#[test]
fn semantic_search_requires_built_index() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();

    let error = db.semantic_index("design_claims").unwrap_err();
    assert_eq!(
        error,
        DbError::SemanticIndexNotBuilt("design_claims".to_owned())
    );
}

#[test]
fn fixed_embedder_reports_missing_embedding() {
    let embedder = FixedSemanticEmbedder::new();

    let error = db_search_error_for_missing_embedding(embedder);
    assert_eq!(
        error,
        DbError::SemanticIndex(SemanticIndexError::MissingEmbedding(
            "claim=unknown".to_owned()
        ))
    );
}

fn db_search_error_for_missing_embedding(embedder: FixedSemanticEmbedder) -> DbError {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let _ = populate_v1_design_claims_semantic_state(&mut db);
    let index_embedder = v1_design_claims_embedder();
    db.rebuild_semantic_index("design_claims", &index_embedder)
        .unwrap();

    db.search_semantic("design_claims", "claim=unknown", 3, &embedder)
        .unwrap_err()
}

#[test]
fn plain_persist_invalidates_existing_semantic_index() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let _ = populate_v1_design_claims_semantic_state(&mut db);

    let embedder = v1_design_claims_embedder();
    db.rebuild_semantic_index("design_claims", &embedder).unwrap();
    assert!(db.semantic_index("design_claims").is_ok());

    let _ = persist_design_claim(
        &mut db,
        "follow-up-note",
        7,
        "A plain persist should invalidate an existing semantic index until it is refreshed.",
    );

    let error = db.semantic_index("design_claims").unwrap_err();
    assert_eq!(
        error,
        DbError::SemanticIndexNotBuilt("design_claims".to_owned())
    );
}

#[test]
fn persist_semantic_with_index_refreshes_semantic_index() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let (_row_a, _row_b, row_c, row_d, row_e, row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let embedder = v1_design_claims_embedder().with_text_embedding(
        "claim=Semantic index refresh should be explicit on indexed persist operations.",
        vec![0.0, 0.92, 0.08],
    );
    db.rebuild_semantic_index("design_claims", &embedder).unwrap();

    let outcome = db
        .persist_semantic_with_index(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("design-decision".to_owned()))
                .with_value("source_turn", Value::Int64(7))
                .with_value(
                    "claim",
                    Value::Text(
                        "Semantic index refresh should be explicit on indexed persist operations."
                            .to_owned(),
                    ),
                ),
            &FixedSemanticJudge::default(),
            &embedder,
        )
        .unwrap();

    let new_row = outcome.row_id();
    let index = db.semantic_index("design_claims").unwrap();
    assert_eq!(index.len(), 7);
    assert!(index.contains_row(new_row));

    let hits = db
        .search_semantic(
            "design_claims",
            "claim=Semantic index refresh should be explicit on indexed persist operations.",
            4,
            &embedder,
        )
        .unwrap();
    assert_eq!(hits[0].row_id(), new_row);

    let neighbors = db.semantic_neighbors("design_claims", new_row, 3).unwrap();
    assert_eq!(neighbors.len(), 3);
    assert_eq!(neighbors[0].row_id(), row_f);
    assert_eq!(neighbors[1].row_id(), row_d);
    assert_eq!(neighbors[2].row_id(), row_c);
    assert!(neighbors.iter().all(|hit| hit.row_id() != row_e));
}

#[test]
fn persist_semantic_with_candidates_only_judges_explicit_rows() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let row_a = persist_design_claim(
        &mut db,
        "design-decision",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );
    let row_d = persist_design_claim(
        &mut db,
        "design-decision",
        2,
        "Projection-time resolution should create a task-local view without mutating base storage by default.",
    );
    let judge = v1_design_claims_judge();

    let outcome = db
        .persist_semantic_with_candidates(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("design-decision".to_owned()))
                .with_value("source_turn", Value::Int64(3))
                .with_value(
                    "claim",
                    Value::Text(
                        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling."
                            .to_owned(),
                    ),
                ),
            &[row_d],
            &judge,
        )
        .unwrap();

    assert_eq!(outcome.support_edges().len(), 0);
    assert_eq!(outcome.conflict_edges().len(), 0);
    assert!(db.conflict_edges("design_claims").unwrap().is_empty());
    assert!(db
        .row("design_claims", row_a)
        .unwrap()
        .get(db.table("design_claims").unwrap().schema(), "claim")
        .is_some());
}

#[test]
fn enrich_row_semantic_adds_missing_relations_once() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let row_a = persist_design_claim(
        &mut db,
        "design-decision",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    );
    let row_b = persist_design_claim(
        &mut db,
        "design-decision",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    );
    let judge = v1_design_claims_judge();

    let first = db
        .enrich_row_semantic("design_claims", row_b, &[row_a], &judge)
        .unwrap();
    assert_eq!(first.conflict_edges().len(), 1);
    assert_eq!(first.conflict_ids().len(), 1);

    let second = db
        .enrich_row_semantic("design_claims", row_b, &[row_a], &judge)
        .unwrap();
    assert!(second.support_edges().is_empty());
    assert!(second.conflict_edges().is_empty());
    assert!(second.conflict_ids().is_empty());
    assert_eq!(db.conflict_edges("design_claims").unwrap().len(), 1);
    assert_eq!(db.conflict_records("design_claims").unwrap().len(), 1);
}

#[test]
fn save_and_load_round_trips_state_but_not_semantic_indexes() {
    let mut db = Database::new();
    db.create_table(design_claims_table_schema()).unwrap();
    let (row_a, row_b, row_c, _row_d, _row_e, _row_f) =
        populate_v1_design_claims_semantic_state(&mut db);

    let binding_id = db.create_binding_set("design_claims").unwrap();
    db.bind_row(binding_id, row_b).unwrap();
    db.bind_row(binding_id, row_c).unwrap();

    let conflict_id = *db
        .conflict_records("design_claims")
        .unwrap()
        .keys()
        .next()
        .unwrap();
    db.set_conflict_status("design_claims", conflict_id, ConflictStatus::Resolved)
        .unwrap();

    let embedder = v1_design_claims_embedder();
    db.rebuild_semantic_index("design_claims", &embedder).unwrap();

    let snapshot_path = unique_snapshot_path("agentdb-roundtrip");
    db.save_to_path(&snapshot_path).unwrap();

    let loaded = Database::load_from_path(&snapshot_path).unwrap();
    let _ = std::fs::remove_file(&snapshot_path);

    assert_eq!(
        loaded.named_rows("design_claims").unwrap(),
        db.named_rows("design_claims").unwrap()
    );
    assert_eq!(
        loaded.support_edges("design_claims").unwrap(),
        db.support_edges("design_claims").unwrap()
    );
    assert_eq!(
        loaded.conflict_edges("design_claims").unwrap(),
        db.conflict_edges("design_claims").unwrap()
    );
    assert_eq!(
        loaded.conflict_record("design_claims", conflict_id).unwrap().status(),
        ConflictStatus::Resolved
    );
    assert_eq!(
        loaded.binding_set(binding_id).unwrap().rows(),
        db.binding_set(binding_id).unwrap().rows()
    );
    assert_eq!(
        loaded.semantic_index("design_claims").unwrap_err(),
        DbError::SemanticIndexNotBuilt("design_claims".to_owned())
    );
    assert_eq!(loaded.row("design_claims", row_a).unwrap().id(), row_a);

    let mut loaded = loaded;
    let new_row = persist_design_claim(
        &mut loaded,
        "loaded-follow-up",
        7,
        "Loaded database should continue row ids without reusing previous ones.",
    );
    assert_eq!(new_row.value(), 7);

    let new_binding = loaded.create_binding_set("design_claims").unwrap();
    assert_eq!(new_binding.value(), 2);
}

fn unique_snapshot_path(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}.json", std::process::id()))
}
