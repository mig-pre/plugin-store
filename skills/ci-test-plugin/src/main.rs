use clap::Parser;

#[derive(Parser)]
#[command(name = "ci-test-plugin", version = env!("CARGO_PKG_VERSION"), about = "CI test plugin for plugin-store pipeline validation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Say hello
    Hello { name: Option<String> },
    /// Check health
    Health,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Hello { name } => {
            let name = name.unwrap_or_else(|| "world".to_string());
            println!("{}", serde_json::json!({"message": format!("Hello, {}!", name)}));
        }
        Commands::Health => {
            println!("{}", serde_json::json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}));
        }
    }
}
