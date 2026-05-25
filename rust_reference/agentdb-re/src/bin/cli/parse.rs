use std::path::PathBuf;

use agentdb::{ColumnRole, DataType};

use crate::cli::command::{ColumnSpec, Command};
use crate::cli::usage::CliError;

pub fn parse_args(args: Vec<String>) -> Result<Command, CliError> {
    if args.is_empty() {
        return Err(CliError::with_usage("missing command"));
    }

    match args[0].as_str() {
        "help" | "--help" | "-h" => Ok(Command::Help),
        "init" => parse_init(&args[1..]),
        "create-table" => parse_create_table(&args[1..]),
        "persist" => parse_persist(&args[1..]),
        "show-table" => parse_show_table(&args[1..]),
        "bind-create" => parse_bind_create(&args[1..]),
        "bind-add" => parse_bind_add(&args[1..]),
        "bind-remove" => parse_bind_remove(&args[1..]),
        "show-binding" => parse_show_binding(&args[1..]),
        "project" => parse_project(&args[1..]),
        other => Err(CliError::with_usage(format!("unknown command `{other}`"))),
    }
}

fn parse_init(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    ensure_no_extra_flags(args, &["--db"])?;
    Ok(Command::Init { db_path })
}

fn parse_create_table(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let table_name = require_string_option(args, "--table")?;
    let column_values = collect_option_values(args, "--column");
    if column_values.is_empty() {
        return Err(CliError::with_usage(
            "create-table requires at least one --column NAME:TYPE:ROLE",
        ));
    }
    let column_specs = column_values
        .into_iter()
        .map(parse_column_spec)
        .collect::<Result<Vec<_>, _>>()?;
    let scope = optional_string_option(args, "--scope")
        .map(|value| value.split(',').map(|item| item.trim().to_owned()).collect());
    ensure_no_extra_flags(args, &["--db", "--table", "--column", "--scope"])?;

    Ok(Command::CreateTable {
        db_path,
        table_name,
        column_specs,
        scope,
    })
}

fn parse_persist(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let table_name = require_string_option(args, "--table")?;
    let set_values = collect_option_values(args, "--set");
    if set_values.is_empty() {
        return Err(CliError::with_usage(
            "persist requires at least one --set KEY=VALUE pair",
        ));
    }
    let raw_values = set_values
        .into_iter()
        .map(parse_key_value)
        .collect::<Result<Vec<_>, _>>()?;
    ensure_no_extra_flags(args, &["--db", "--table", "--set"])?;

    Ok(Command::Persist {
        db_path,
        table_name,
        raw_values,
    })
}

fn parse_show_table(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let table_name = require_string_option(args, "--table")?;
    ensure_no_extra_flags(args, &["--db", "--table"])?;
    Ok(Command::ShowTable { db_path, table_name })
}

fn parse_bind_create(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let table_name = require_string_option(args, "--table")?;
    ensure_no_extra_flags(args, &["--db", "--table"])?;
    Ok(Command::BindCreate { db_path, table_name })
}

fn parse_bind_add(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let binding_id = require_u64_option(args, "--binding")?;
    let row_id = require_u64_option(args, "--row")?;
    ensure_no_extra_flags(args, &["--db", "--binding", "--row"])?;
    Ok(Command::BindAdd {
        db_path,
        binding_id,
        row_id,
    })
}

fn parse_bind_remove(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let binding_id = require_u64_option(args, "--binding")?;
    let row_id = require_u64_option(args, "--row")?;
    ensure_no_extra_flags(args, &["--db", "--binding", "--row"])?;
    Ok(Command::BindRemove {
        db_path,
        binding_id,
        row_id,
    })
}

fn parse_show_binding(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let binding_id = require_u64_option(args, "--binding")?;
    ensure_no_extra_flags(args, &["--db", "--binding"])?;
    Ok(Command::ShowBinding { db_path, binding_id })
}

fn parse_project(args: &[String]) -> Result<Command, CliError> {
    let db_path = require_path_option(args, "--db")?;
    let binding_id = require_u64_option(args, "--binding")?;
    ensure_no_extra_flags(args, &["--db", "--binding"])?;
    Ok(Command::Project { db_path, binding_id })
}

fn parse_column_spec(raw: String) -> Result<ColumnSpec, CliError> {
    let mut parts = raw.split(':');
    let name = parts
        .next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            CliError::new(format!(
                "invalid column spec `{raw}`, expected NAME:TYPE:ROLE"
            ))
        })?
        .trim()
        .to_owned();
    let data_type = parse_data_type(parts.next().unwrap_or_default().trim())?;
    let role = parse_role(parts.next().unwrap_or_default().trim())?;
    if parts.next().is_some() {
        return Err(CliError::new(format!(
            "invalid column spec `{raw}`, expected NAME:TYPE:ROLE"
        )));
    }

    Ok(ColumnSpec {
        name,
        data_type,
        role,
    })
}

fn parse_data_type(raw: &str) -> Result<DataType, CliError> {
    match raw {
        "text" => Ok(DataType::Text),
        "int64" => Ok(DataType::Int64),
        "bool" => Ok(DataType::Bool),
        _ => Err(CliError::new(format!(
            "unknown data type `{raw}`, expected text|int64|bool"
        ))),
    }
}

fn parse_role(raw: &str) -> Result<ColumnRole, CliError> {
    match raw {
        "normal" => Ok(ColumnRole::Normal),
        "semantic" => Ok(ColumnRole::Semantic),
        _ => Err(CliError::new(format!(
            "unknown column role `{raw}`, expected normal|semantic"
        ))),
    }
}

fn parse_key_value(raw: String) -> Result<(String, String), CliError> {
    let Some((key, value)) = raw.split_once('=') else {
        return Err(CliError::new(format!(
            "invalid --set value `{raw}`, expected KEY=VALUE"
        )));
    };
    if key.trim().is_empty() {
        return Err(CliError::new(format!(
            "invalid --set value `{raw}`, key must be non-empty"
        )));
    }
    Ok((key.trim().to_owned(), value.to_owned()))
}

fn require_path_option(args: &[String], flag: &str) -> Result<PathBuf, CliError> {
    Ok(PathBuf::from(require_string_option(args, flag)?))
}

fn require_string_option(args: &[String], flag: &str) -> Result<String, CliError> {
    optional_string_option(args, flag)
        .ok_or_else(|| CliError::with_usage(format!("missing required option `{flag}`")))
}

fn require_u64_option(args: &[String], flag: &str) -> Result<u64, CliError> {
    let raw = require_string_option(args, flag)?;
    raw.parse::<u64>().map_err(|error| {
        CliError::new(format!("invalid numeric value for `{flag}`: `{raw}` ({error})"))
    })
}

fn optional_string_option(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

fn collect_option_values(args: &[String], flag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == flag && index + 1 < args.len() {
            values.push(args[index + 1].clone());
            index += 2;
        } else {
            index += 1;
        }
    }
    values
}

fn ensure_no_extra_flags(args: &[String], allowed_flags: &[&str]) -> Result<(), CliError> {
    let mut index = 0;
    while index < args.len() {
        let item = &args[index];
        if item.starts_with("--") {
            if !allowed_flags.contains(&item.as_str()) {
                return Err(CliError::with_usage(format!("unknown option `{item}`")));
            }
            if index + 1 >= args.len() {
                return Err(CliError::with_usage(format!(
                    "missing value for option `{item}`"
                )));
            }
            index += 2;
        } else {
            return Err(CliError::with_usage(format!(
                "unexpected positional argument `{item}`"
            )));
        }
    }
    Ok(())
}
