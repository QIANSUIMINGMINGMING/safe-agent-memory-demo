use agentdb::{
    ColumnRole, ColumnSchema, DataType, Database, FixedSemanticJudge, ProjectionPolicy, RowInput,
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

    let row_e = db
        .persist_semantic(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("architecture-note".to_owned()))
                .with_value("source_turn", Value::Int64(5))
                .with_value(
                    "claim",
                    Value::Text("Vector retrieval is an execution strategy for semantic access, not the core data model.".to_owned()),
                ),
            &judge,
        )?
        .row_id();

    let row_f = db
        .persist_semantic(
            "design_claims",
            RowInput::new()
                .with_value("project", Value::Text("agentdb".to_owned()))
                .with_value("topic", Value::Text("semantic-core".to_owned()))
                .with_value("kind", Value::Text("supporting-note".to_owned()))
                .with_value("source_turn", Value::Int64(6))
                .with_value(
                    "claim",
                    Value::Text("Persisted support and conflict relations should be first-class state in the kernel.".to_owned()),
                ),
            &judge,
        )?
        .row_id();

    let binding_id = db.create_binding_set("design_claims")?;
    db.bind_row(binding_id, row_b)?;
    db.bind_row(binding_id, row_c)?;
    db.bind_row(binding_id, row_d)?;
    db.bind_row(binding_id, row_e)?;
    db.bind_row(binding_id, row_f)?;

    let projection = db.project_binding(binding_id, ProjectionPolicy::conservative())?;

    println!("reference outdated row: {}", row_a.value());
    println!("considered rows: {:#?}", projection.considered_rows());
    println!("accepted rows: {:#?}", projection.accepted_rows());
    println!("suppressed rows: {:#?}", projection.suppressed_rows());
    println!("ambiguous rows: {:#?}", projection.ambiguous_rows());
    println!("consulted conflicts: {:#?}", projection.consulted_conflicts());

    Ok(())
}
