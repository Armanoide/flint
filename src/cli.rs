use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start a service by formula
    Start { formula: String },
    /// Stop a service by formula
    Stop { formula: String },
    /// Query status
    Status { formula: Option<String> },
}
