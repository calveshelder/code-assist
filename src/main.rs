use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

mod app;
mod config;
mod ui;
mod llm;
mod git;
mod fs;
mod analysis;
mod commands;
mod memory;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn on verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure the application
    Config {
        /// Set the LLM API endpoint URL
        #[arg(long)]
        api_url: Option<String>,
        
        /// Set the API key for the LLM
        #[arg(long)]
        api_key: Option<String>,
        
        /// Set the LLM model to use
        #[arg(long)]
        model: Option<String>,
    },
    
    /// Execute a one-off command without entering interactive mode
    Exec {
        /// The natural language command to execute
        #[arg(required = true)]
        command: Vec<String>,
    },

    /// Initialize a CAULK.md file in the current directory
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    // Load configuration
    let config_path = cli.config.unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("code-assist/config.toml")
    });
    
    let config = config::load_or_create_config(&config_path)?;
    
    // Handle subcommands
    match &cli.command {
        Some(Commands::Config { api_url, api_key, model }) => {
            config::update_config(&config_path, api_url, api_key, model)?;
            println!("Configuration updated successfully.");
            return Ok(());
        }
        Some(Commands::Exec { command }) => {
            let command_str = command.join(" ");
            let mut app = app::App::new(config)?;  // Made app mutable
            app.execute_command(&command_str).await?;
            return Ok(());
        }
        Some(Commands::Init) => {
            let cwd = std::env::current_dir()?;
            let memory = memory::ProjectMemory::new();
            memory.init_caulk_file(&cwd)?;
            return Ok(());
        }
        None => {
            // No subcommand, enter interactive mode
            let mut app = app::App::new(config)?;
            app.run().await?;
        }
    }
    
    Ok(())
}
