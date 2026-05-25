mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty()
        || matches!(args.first().map(String::as_str), Some("help" | "--help" | "-h"))
    {
        println!("{}", cli::usage_text());
        return Ok(());
    }

    cli::run(args)
}
