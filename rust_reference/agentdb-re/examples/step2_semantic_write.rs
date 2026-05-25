use agentdb::{
    ColumnRole, ColumnSchema, ConflictStatus, DataType, Database, FixedSemanticJudge, RowInput,
    SemanticRelation, SemanticScope, TableSchema, Value,
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
        );

    let mut db = Database::new();
    db.create_table(schema)?;

    let row_a = db
        .persist(
        "design_claims",
        RowInput::new()
            .with_value("project", Value::Text("agentdb".to_owned()))
            .with_value("topic", Value::Text("semantic-core".to_owned()))
            .with_value("kind", Value::Text("draft-conclusion".to_owned()))
            .with_value("source_turn", Value::Int64(1))
            .with_value(
                "claim",
                Value::Text("Strict atomicity should be a hard invariant for semantic rows.".to_owned()),
            ),
    )?
        .row_id();

    let row_b = db
        .persist_semantic(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("revised-conclusion".to_owned()))
                .with_value("source_turn", Value::Int64(2))
                .with_value(
                    "claim",
                    Value::Text("Strict atomicity should not be a hard invariant; judgment coherence is enough for practical modeling.".to_owned()),
                ),
            &judge,
        )?
        .row_id();

    let row_c = db
        .persist_semantic(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("design-decision".to_owned()))
                .with_value("source_turn", Value::Int64(3))
                .with_value(
                    "claim",
                    Value::Text("Conflicts should be preserved in base storage instead of being blocked or destructively resolved at write time.".to_owned()),
                ),
            &judge,
        )?
        .row_id();

    let row_d = db
        .persist_semantic(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("design-decision".to_owned()))
                .with_value("source_turn", Value::Int64(4))
                .with_value(
                    "claim",
                    Value::Text("Projection-time resolution should create a task-local view without mutating base storage by default.".to_owned()),
                ),
            &judge,
        )?
        .row_id();

    println!("row ids: a={}, b={}, c={}, d={}", row_a.value(), row_b.value(), row_c.value(), row_d.value());
    println!("rows: {:#?}", db.named_rows("design_claims")?);
    println!("support edges: {:#?}", db.support_edges("design_claims")?);
    println!("conflict edges: {:#?}", db.conflict_edges("design_claims")?);
    println!("conflict records: {:#?}", db.conflict_records("design_claims")?);

    let first_conflict_id = *db
        .conflict_records("design_claims")?
        .keys()
        .next()
        .expect("expected one conflict record");
    db.set_conflict_status("design_claims", first_conflict_id, ConflictStatus::Resolved)?;
    println!(
        "updated first conflict: {:#?}",
        db.conflict_record("design_claims", first_conflict_id)?
    );

    Ok(())
}
