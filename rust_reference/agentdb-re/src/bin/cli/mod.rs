mod command;
mod parse;
mod usage;

use std::error::Error;

pub fn usage_text() -> &'static str {
    usage::usage_text()
}

pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = parse::parse_args(args)?;
    command.execute()?;
    Ok(())
}
