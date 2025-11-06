use crate::cli::{Cli, Commands};
use crate::error::Result;
use crate::service_manager::ServiceManager;
use clap::Parser;
mod cli;
mod error;
mod launchd_config;
mod service_manager;
mod services;

#[tokio::main]
async fn main() {
    // init tracing
    tracing_subscriber::fmt::init();

    if let Err(err) = try_main().await {
        eprintln!("❌ Error: {}", err);
        std::process::exit(1);
    }
}

fn get_manager(service_name: String) -> Result<ServiceManager> {
    ServiceManager::new(service_name)
}

async fn try_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { formula } => get_manager(formula)?.start()?,
        Commands::Stop { formula } => get_manager(formula)?.stop()?,
        Commands::Status { formula } => match formula {
            Some(name) => {
                get_manager(name)?.state()?;
            }
            None => {
                ServiceManager::states()?;
            }
        },
    }

    Ok(())
}
