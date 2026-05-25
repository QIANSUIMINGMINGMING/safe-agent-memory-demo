use agentdb::{
    ColumnRole, ColumnSchema, DataType, Database, FixedSemanticEmbedder, FixedSemanticJudge,
    RowInput, SemanticRelation, SemanticScope, TableSchema, Value,
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
        );

    let mut db = Database::new();
    db.create_table(schema)?;

    let row_a = db
        .persist_with_index(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("draft-conclusion".to_owned()))
                .with_value("source_turn", Value::Int64(1))
                .with_value(
                    "claim",
                    Value::Text(
                        "Strict atomicity should be a hard invariant for semantic rows."
                            .to_owned(),
                    ),
                ),
            &embedder,
        )?
        .row_id();

    let row_b = db
        .persist_semantic_with_index(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("revised-conclusion".to_owned()))
                .with_value("source_turn", Value::Int64(2))
                .with_value(
                    "claim",
                    Value::Text(
                        "Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling."
                            .to_owned(),
                    ),
                ),
            &judge,
            &embedder,
        )?
        .row_id();

    let row_c = db
        .persist_semantic_with_index(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("design-decision".to_owned()))
                .with_value("source_turn", Value::Int64(3))
                .with_value(
                    "claim",
                    Value::Text(
                        "Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time."
                            .to_owned(),
                    ),
                ),
            &judge,
            &embedder,
        )?
        .row_id();

    let binding_id = db.create_binding_set("design_claims")?;
    db.bind_row(binding_id, row_b)?;
    db.bind_row(binding_id, row_c)?;

    let snapshot_path = std::env::temp_dir().join("agentdb-step8-storage.json");
    db.save_to_path(&snapshot_path)?;
    println!("saved snapshot: {}", snapshot_path.display());

    let mut loaded = Database::load_from_path(&snapshot_path)?;
    let _ = std::fs::remove_file(&snapshot_path);

    println!("rows after load: {:#?}", loaded.named_rows("design_claims")?);
    println!("support edges after load: {:#?}", loaded.support_edges("design_claims")?);
    println!("conflict records after load: {:#?}", loaded.conflict_records("design_claims")?);
    println!("binding after load: {:#?}", loaded.binding_set(binding_id)?);
    println!(
        "semantic index after load: {:#?}",
        loaded.semantic_index("design_claims").unwrap_err()
    );

    loaded.rebuild_semantic_index("design_claims", &embedder)?;
    println!(
        "semantic search after rebuild: {:#?}",
        loaded.search_semantic(
            "design_claims",
            "claim=Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.",
            3,
            &embedder,
        )?
    );

    println!(
        "row ids carried through snapshot: a={}, b={}, c={}",
        row_a.value(),
        row_b.value(),
        row_c.value()
    );

    Ok(())
}
