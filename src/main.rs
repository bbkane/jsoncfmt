mod formatter;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use atomic_write_file::AtomicWriteFile;
use clap::{ArgAction, Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

#[derive(Args)]
struct FormatArgs {
    /// Path to the JSONC file to format
    file: PathBuf,

    /// Sort object keys recursively
    #[arg(short = 's', long, default_value_t = true, action = ArgAction::Set)]
    sort_keys: bool,

    /// Replace the input file instead of writing to stdout
    #[arg(short = 'i', long, default_value_t = false, action = ArgAction::Set)]
    in_place: bool,

    /// Number of spaces per indentation level
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..))]
    indent_width: u8,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Format a JSONC file")]
    Format(FormatArgs),

    #[command(about = "Print version")]
    Version,

    #[command(about = "Print shell completion scripts")]
    Completion { shell: Shell },
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::Format(args) => format_file(args),
        Commands::Version => {
            println!("v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Completion { shell } => {
            generate(
                shell,
                &mut Cli::command(),
                env!("CARGO_PKG_NAME"),
                &mut io::stdout(),
            );
            Ok(())
        }
    }
}

fn format_file(args: FormatArgs) -> Result<(), String> {
    let input = fs::read_to_string(&args.file)
        .map_err(|error| format!("failed to read '{}': {error}", args.file.display()))?;
    let formatted = formatter::format(&args.file, &input, args.sort_keys, args.indent_width)
        .map_err(|error| format!("failed to format '{}': {error}", args.file.display()))?;

    if args.in_place {
        let target = &args
            .file
            .canonicalize()
            .map_err(|error| format!("failed to resolve '{}': {error}", args.file.display()))?;
        let mut output = AtomicWriteFile::open(target)
            .map_err(|error| format!("failed to open '{}': {error}", target.display()))?;
        output
            .write_all(formatted.as_bytes())
            .map_err(|error| format!("failed to write '{}': {error}", target.display()))?;
        output
            .commit()
            .map_err(|error| format!("failed to replace '{}': {error}", target.display()))?;
        Ok(())
    } else {
        io::stdout()
            .lock()
            .write_all(formatted.as_bytes())
            .map_err(|error| format!("failed to write formatted output: {error}"))
    }
}
