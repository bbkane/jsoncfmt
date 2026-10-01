use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

#[derive(Args)]
struct HelloArgs {
    #[arg(long)]
    name: String,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Print version")]
    Version,

    #[command(about = "Say hello")]
    Hello(HelloArgs),

    #[command(about = "Print shell completion scripts")]
    Completion { shell: Shell },
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Version => {
            println!("v{}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Hello(args) => {
            println!("Hello there {}!", args.name);
        }
        Commands::Completion { shell } => {
            generate(
                *shell,
                &mut Cli::command(),
                env!("CARGO_PKG_NAME"),
                &mut std::io::stdout(),
            );
        }
    }
}
