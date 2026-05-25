use agentdb::{
    ColumnRole, ColumnSchema, DataType, Database, FixedSemanticJudge, RowInput, SemanticRelation,
    SemanticScope, TableSchema, Value,
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

    let mut db = Database::new();
    db.create_table(schema)?;

    let plain = db.persist(
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
    )?;

    let semantic = db.persist_semantic(
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
    )?;

    let semantic_support = db.persist_semantic(
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
    )?;

    println!("plain persist outcome: {:#?}", plain);
    println!("semantic overlap outcome: {:#?}", semantic);
    println!("semantic support outcome: {:#?}", semantic_support);

    Ok(())
}
