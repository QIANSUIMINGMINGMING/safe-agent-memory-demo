use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use agentdb::{
    BindingId, ColumnRole, ColumnSchema, DataType, Database, ProjectionPolicy, RowId, RowInput,
    SemanticScope, TableSchema, Value,
};

use crate::cli::usage::CliError;

#[derive(Debug, Clone)]
pub enum Command {
    Help,
    Init { db_path: PathBuf },
    CreateTable {
        db_path: PathBuf,
        table_name: String,
        column_specs: Vec<ColumnSpec>,
        scope: Option<Vec<String>>,
    },
    Persist {
        db_path: PathBuf,
        table_name: String,
        raw_values: Vec<(String, String)>,
    },
    ShowTable {
        db_path: PathBuf,
        table_name: String,
    },
    BindCreate {
        db_path: PathBuf,
        table_name: String,
    },
    BindAdd {
        db_path: PathBuf,
        binding_id: u64,
        row_id: u64,
    },
    BindRemove {
        db_path: PathBuf,
        binding_id: u64,
        row_id: u64,
    },
    ShowBinding {
        db_path: PathBuf,
        binding_id: u64,
    },
    Project {
        db_path: PathBuf,
        binding_id: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSpec {
    pub name: String,
    pub data_type: DataType,
    pub role: ColumnRole,
}

impl Command {
    pub fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Help => println!("{}", crate::cli::usage::usage_text()),
            Self::Init { db_path } => execute_init(&db_path)?,
            Self::CreateTable {
                db_path,
                table_name,
                column_specs,
                scope,
            } => execute_create_table(&db_path, &table_name, &column_specs, scope.as_deref())?,
            Self::Persist {
                db_path,
                table_name,
                raw_values,
            } => execute_persist(&db_path, &table_name, &raw_values)?,
            Self::ShowTable { db_path, table_name } => execute_show_table(&db_path, &table_name)?,
            Self::BindCreate { db_path, table_name } => {
                execute_bind_create(&db_path, &table_name)?
            }
            Self::BindAdd {
                db_path,
                binding_id,
                row_id,
            } => execute_bind_add(&db_path, binding_id, row_id)?,
            Self::BindRemove {
                db_path,
                binding_id,
                row_id,
            } => execute_bind_remove(&db_path, binding_id, row_id)?,
            Self::ShowBinding { db_path, binding_id } => {
                execute_show_binding(&db_path, binding_id)?
            }
            Self::Project { db_path, binding_id } => execute_project(&db_path, binding_id)?,
        }

        Ok(())
    }
}

fn execute_init(db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if db_path.exists() {
        return Err(Box::new(CliError::new(format!(
            "database file already exists: {}",
            db_path.display()
        ))));
    }

    let db = Database::new();
    db.save_to_path(db_path)?;
    println!("initialized database: {}", db_path.display());
    Ok(())
}

fn execute_create_table(
    db_path: &Path,
    table_name: &str,
    column_specs: &[ColumnSpec],
    scope: Option<&[String]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut db = load_db(db_path)?;
    let columns = column_specs
        .iter()
        .map(|spec| ColumnSchema::new(spec.name.clone(), spec.data_type, spec.role))
        .collect::<Result<Vec<_>, _>>()?;
    let semantic_scope = match scope {
        Some(columns) => Some(SemanticScope::new(columns.iter().cloned())?),
        None => None,
    };
    let schema = TableSchema::new(table_name.to_owned(), columns, semantic_scope)?;
    db.create_table(schema)?;
    db.save_to_path(db_path)?;
    println!("created table `{table_name}` in {}", db_path.display());
    Ok(())
}

fn execute_persist(
    db_path: &Path,
    table_name: &str,
    raw_values: &[(String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut db = load_db(db_path)?;
    let schema = db.table(table_name)?.schema().clone();
    let input = parse_row_input(&schema, raw_values)?;
    let outcome = db.persist(table_name, input)?;
    db.save_to_path(db_path)?;

    println!("persisted row {}", outcome.row_id().value());
    if outcome.has_semantic_effects() {
        println!("semantic effects: {:#?}", outcome);
    }
    Ok(())
}

fn execute_show_table(
    db_path: &Path,
    table_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = load_db(db_path)?;
    let table = db.table(table_name)?;

    println!("table: {}", table.schema().name());
    println!("columns:");
    for column in table.schema().columns() {
        println!(
            "  - {}: {} ({})",
            column.name(),
            data_type_name(column.data_type()),
            role_name(column.role())
        );
    }
    if let Some(scope) = table.schema().semantic_scope() {
        println!("semantic scope: {}", scope.columns().join(", "));
    } else {
        println!("semantic scope: <none>");
    }
    println!("rows: {:#?}", db.named_rows(table_name)?);
    println!("compatible edges: {:#?}", db.compatible_edges(table_name)?);
    println!("support edges: {:#?}", db.support_edges(table_name)?);
    println!("compressible edges: {:#?}", db.compressible_edges(table_name)?);
    println!("conflict edges: {:#?}", db.conflict_edges(table_name)?);
    println!("conflict records: {:#?}", db.conflict_records(table_name)?);
    Ok(())
}

fn execute_bind_create(
    db_path: &Path,
    table_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut db = load_db(db_path)?;
    let binding_id = db.create_binding_set(table_name)?;
    db.save_to_path(db_path)?;
    println!("created binding set {}", binding_id.value());
    Ok(())
}

fn execute_bind_add(
    db_path: &Path,
    binding_id: u64,
    row_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut db = load_db(db_path)?;
    let changed = db.bind_row(BindingId::from_value(binding_id), RowId::from_value(row_id))?;
    db.save_to_path(db_path)?;
    println!("binding updated: changed={changed}");
    Ok(())
}

fn execute_bind_remove(
    db_path: &Path,
    binding_id: u64,
    row_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut db = load_db(db_path)?;
    let changed = db.unbind_row(BindingId::from_value(binding_id), RowId::from_value(row_id))?;
    db.save_to_path(db_path)?;
    println!("binding updated: changed={changed}");
    Ok(())
}

fn execute_show_binding(
    db_path: &Path,
    binding_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = load_db(db_path)?;
    let binding = db.binding_set(BindingId::from_value(binding_id))?;
    println!("binding set: {:#?}", binding);
    Ok(())
}

fn execute_project(
    db_path: &Path,
    binding_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = load_db(db_path)?;
    let binding_id = BindingId::from_value(binding_id);
    let binding = db.binding_set(binding_id)?;
    let table = db.table(binding.table_name())?;
    let projection = db.project_binding(binding_id, ProjectionPolicy::conservative())?;

    println!("projection: {:#?}", projection);
    println!("accepted rows:");
    for row_id in projection.accepted_rows() {
        let row = db.row(binding.table_name(), *row_id)?;
        println!("  - {}: {:#?}", row_id.value(), row.as_named_values(table.schema()));
    }
    Ok(())
}

fn load_db(path: &Path) -> Result<Database, Box<dyn std::error::Error>> {
    Ok(Database::load_from_path(path)?)
}

fn parse_row_input(
    schema: &TableSchema,
    raw_values: &[(String, String)],
) -> Result<RowInput, Box<dyn std::error::Error>> {
    if raw_values.is_empty() {
        return Err(Box::new(CliError::new(
            "persist requires at least one --set KEY=VALUE pair",
        )));
    }

    let mut latest_values = BTreeMap::new();
    for (column, raw_value) in raw_values {
        latest_values.insert(column.clone(), raw_value.clone());
    }

    for provided_column in latest_values.keys() {
        if schema.column(provided_column).is_none() {
            return Err(Box::new(CliError::new(format!(
                "unknown column `{provided_column}` for table `{}`",
                schema.name()
            ))));
        }
    }

    let mut input = RowInput::new();
    for column in schema.columns() {
        let Some(raw_value) = latest_values.get(column.name()) else {
            continue;
        };

        let value = parse_value(column.data_type(), raw_value)?;
        input = input.with_value(column.name(), value);
    }

    Ok(input)
}

fn parse_value(
    data_type: DataType,
    raw_value: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    match data_type {
        DataType::Text => Ok(Value::Text(raw_value.to_owned())),
        DataType::Int64 => {
            let value = raw_value.parse::<i64>().map_err(|error| {
                CliError::new(format!("invalid int64 value `{raw_value}`: {error}"))
            })?;
            Ok(Value::Int64(value))
        }
        DataType::Bool => match raw_value {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(Box::new(CliError::new(format!(
                "invalid bool value `{raw_value}`, expected true or false"
            )))),
        },
    }
}

fn data_type_name(data_type: DataType) -> &'static str {
    match data_type {
        DataType::Text => "text",
        DataType::Int64 => "int64",
        DataType::Bool => "bool",
    }
}

fn role_name(role: ColumnRole) -> &'static str {
    match role {
        ColumnRole::Normal => "normal",
        ColumnRole::Semantic => "semantic",
    }
}
