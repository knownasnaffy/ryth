mod commands;

use clap::Parser;
use ryth::cli::{Cli, Command};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", serde_json::json!({ "error": e.to_string() }));
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Status { watch } => commands::status(watch).await,
        Command::List { watch } => commands::list(watch).await,
        Command::Scan => commands::scan().await,
        Command::Connect { ssid, password } => commands::connect(ssid, password).await,
        Command::Disconnect => commands::disconnect().await,
        Command::Autoconnect { ssid, state } => commands::autoconnect(ssid, state.into()).await,
        Command::Forget { ssid } => commands::forget(ssid).await,
        Command::Known => commands::known().await,
        Command::Power { state } => commands::power(state.into()).await,
    }
}
