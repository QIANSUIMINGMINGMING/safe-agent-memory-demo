use agentdb::{ColumnRole, ColumnSchema, DataType, Database, RowInput, SemanticScope, TableSchema, Value};

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
        .persist(
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
    )?
        .row_id();

    let row_c = db
        .persist(
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
    )?
        .row_id();

    let binding_id = db.create_binding_set("design_claims")?;
    db.bind_row(binding_id, row_a)?;
    db.bind_row(binding_id, row_b)?;
    db.bind_row(binding_id, row_c)?;

    println!("binding set after add: {:#?}", db.binding_set(binding_id)?);

    db.unbind_row(binding_id, row_b)?;
    println!("binding set after remove: {:#?}", db.binding_set(binding_id)?);

    Ok(())
}
