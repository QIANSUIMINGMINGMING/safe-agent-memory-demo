use agentdb::{
    ColumnRole, ColumnSchema, DataType, Database, FixedSemanticEmbedder, FixedSemanticJudge,
    RowId, RowInput, SemanticRelation, SemanticScope, TableSchema, Value,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = TableSchema::new(
        "design_claims",
        vec![
            ColumnSchema::new("project", DataType::Text, ColumnRole::Normal)?,
            ColumnSchema::new("topic", DataType::Text, ColumnRole::Normal)?,
            ColumnSchema::new("kind", DataType::Text, ColumnRole::Normal)?,
            ColumnSchema::new("source_turn", DataType::Int64, ColumnRole::Normal)?,
            ColumnSchema::new("claim", DataType::Text, ColumnRole::Semantic)?,
        ],
        Some(SemanticScope::new(["claim"])?),
    )?;

    let judge = FixedSemanticJudge::default()
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
        );

    let embedder = FixedSemanticEmbedder::new()
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
        );

    let mut db = Database::new();
    db.create_table(schema)?;

    persist_claim(
        &mut db,
        "draft-conclusion",
        1,
        "Strict atomicity should be a hard invariant for semantic rows.",
    )?;
    let _row_b = persist_claim_semantic(
        &mut db,
        &judge,
        "revised-conclusion",
        2,
        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.",
    )?;
    let row_c = persist_claim_semantic(
        &mut db,
        &judge,
        "design-decision",
        3,
        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
    )?;
    let row_d = persist_claim_semantic(
        &mut db,
        &judge,
        "design-decision",
        4,
        "Projection-time resolution should create a task-local view without mutating base storage by default.",
    )?;
    let _row_e = persist_claim_semantic(
        &mut db,
        &judge,
        "architecture-note",
        5,
        "Vector retrieval is an execution strategy for semantic access, not the core data model.",
    )?;
    let _row_f = persist_claim_semantic(
        &mut db,
        &judge,
        "supporting-note",
        6,
        "Persisted support and conflict relations should be first-class state in the kernel.",
    )?;

    db.rebuild_semantic_index("design_claims", &embedder)?;

    println!(
        "semantic query hits: {:#?}",
        db.search_semantic(
            "design_claims",
            "claim=Projection-time resolution should create a coherent task-local view over preserved semantic state.",
            4,
            &embedder,
        )?
    );
    println!(
        "semantic neighbors of row {}: {:#?}",
        row_d.value(),
        db.semantic_neighbors("design_claims", row_d, 3)?
    );
    println!(
        "support neighbors of row {}: {:#?}",
        row_c.value(),
        db.support_neighbors("design_claims", row_c)?
    );

    Ok(())
}

fn persist_claim(
    db: &mut Database,
    kind: &str,
    source_turn: i64,
    claim: &str,
) -> Result<RowId, Box<dyn std::error::Error>> {
    Ok(db
        .persist(
        "design_claims",
        RowInput::new()
            .with_value("project", Value::Text("agentdb".to_owned()))
            .with_value("topic", Value::Text("semantic-core".to_owned()))
            .with_value("kind", Value::Text(kind.to_owned()))
            .with_value("source_turn", Value::Int64(source_turn))
            .with_value("claim", Value::Text(claim.to_owned())),
    )?
        .row_id())
}

fn persist_claim_semantic(
    db: &mut Database,
    judge: &FixedSemanticJudge,
    kind: &str,
    source_turn: i64,
    claim: &str,
) -> Result<RowId, Box<dyn std::error::Error>> {
    Ok(db
        .persist_semantic(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text(kind.to_owned()))
                .with_value("source_turn", Value::Int64(source_turn))
                .with_value("claim", Value::Text(claim.to_owned())),
            judge,
        )?
        .row_id())
}
