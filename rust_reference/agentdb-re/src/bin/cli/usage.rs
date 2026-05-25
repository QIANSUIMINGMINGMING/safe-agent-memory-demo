use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CliError {
    message: String,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn with_usage(message: impl Into<String>) -> Self {
        Self::new(format!("{}\n\n{}", message.into(), usage_text()))
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CliError {}

pub fn usage_text() -> &'static str {
    r#"Usage:
  agentdb init --db PATH
  agentdb create-table --db PATH --table NAME --column NAME:TYPE:ROLE [--column ...] [--scope COL1,COL2]
  agentdb persist --db PATH --table NAME --set KEY=VALUE [--set ...]
  agentdb show-table --db PATH --table NAME
  agentdb bind-create --db PATH --table NAME
  agentdb bind-add --db PATH --binding ID --row ID
  agentdb bind-remove --db PATH --binding ID --row ID
  agentdb show-binding --db PATH --binding ID
  agentdb project --db PATH --binding ID

Type names:
  text | int64 | bool

Role names:
  normal | semantic
"#
}
