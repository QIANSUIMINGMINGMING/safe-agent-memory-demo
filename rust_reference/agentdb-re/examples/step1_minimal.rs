use agentdb::{
    semantic_insert_action, ColumnRole, ColumnSchema, DataType, Database, ExactFilter, RowInput,
    SemanticRelation, SemanticScope, TableSchema, Value,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = TableSchema::new(
        "project_notes",
        vec![
            ColumnSchema::new("project_id", DataType::Text, ColumnRole::Normal)?,
            ColumnSchema::new("topic", DataType::Text, ColumnRole::Semantic)?,
            ColumnSchema::new("statement", DataType::Text, ColumnRole::Semantic)?,
            ColumnSchema::new("archived", DataType::Bool, ColumnRole::Normal)?,
        ],
        Some(SemanticScope::new(["topic", "statement"])?),
    )?;

    let mut db = Database::new();
    db.create_table(schema)?;
    let first_row_id = db
        .persist(
        "project_notes",
        RowInput::new()
            .with_value("project_id", Value::Text("agentdb".to_owned()))
            .with_value("topic", Value::Text("bootstrapping".to_owned()))
            .with_value("statement", Value::Text("step 1 only stores schema and rows".to_owned()))
            .with_value("archived", Value::Bool(false)),
    )?
        .row_id();
    let second_row_id = db
        .persist(
        "project_notes",
        RowInput::new()
            .with_value("project_id", Value::Text("agentdb".to_owned()))
            .with_value("topic", Value::Text("semantic".to_owned()))
            .with_value("statement", Value::Text("semantic behavior is not implemented yet".to_owned()))
            .with_value("archived", Value::Bool(true)),
    )?
        .row_id();
    let third_row_id = db
        .persist(
        "project_notes",
        RowInput::new()
            .with_value("project_id", Value::Text("other-project".to_owned()))
            .with_value("topic", Value::Text("bootstrapping".to_owned()))
            .with_value("statement", Value::Text("another row in another project".to_owned()))
            .with_value("archived", Value::Bool(false)),
    )?
        .row_id();

    println!("table: project_notes");
    println!("rows: {:#?}", db.named_rows("project_notes")?);
    println!(
        "row ids: first={}, second={}, third={}",
        first_row_id.value(),
        second_row_id.value(),
        third_row_id.value()
    );
    println!();
    println!("exact scan: project_id = agentdb AND archived = false");
    println!(
        "matches: {:#?}",
        db.scan_exact(
            "project_notes",
            &[
                ExactFilter::new("project_id", Value::Text("agentdb".to_owned()))?,
                ExactFilter::new("archived", Value::Bool(false))?,
            ],
        )?
    );
    println!();
    println!(
        "semantic policy demo: {:?}",
        semantic_insert_action(&[
            SemanticRelation::support(),
            SemanticRelation::none(),
        ])
    );

    Ok(())
}
