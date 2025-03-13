use crate::commands::executor::CommandExecutor;
use crate::config::Config;
use crate::llm::client::LlmClient;
use crate::llm::context::ContextManager;
use crate::ui::prompt::Prompt;
use anyhow::{Context, Result};
use colored::Colorize;

pub struct App {
    config: Config,
    llm_client: LlmClient,
    context_manager: ContextManager,
    command_executor: CommandExecutor,
    prompt: Prompt,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let llm_client = LlmClient::new(&config)?;
        let context_manager = ContextManager::new();
        let command_executor = CommandExecutor::new();
        let prompt = Prompt::new();

        Ok(Self {
            config,
            llm_client,
            context_manager,
            command_executor,
            prompt,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "Welcome to CodeAssist!".bright_green().bold());
        println!("Type your natural language commands or 'exit' to quit");

        loop {
            let input = self.prompt.get_input()?;

            if input.trim().to_lowercase() == "exit" {
                break;
            }

            if let Err(e) = self.execute_command(&input).await {
                eprintln!("{} {}", "Error:".bright_red().bold(), e);
            }
        }

        println!("Goodbye!");
        Ok(())
    }

    pub async fn execute_command(&self, command: &str) -> Result<()> {
        println!("{}", "Analyzing request...".bright_blue());

        // Gather context from the codebase
        let context = self.context_manager.gather_context(command)?;

        // Send to LLM for interpretation
        let llm_response = self
            .llm_client
            .process_command(command, &context)
            .await
            .context("Failed to process command with LLM")?;

        // Add this line for debugging
        println!("Raw LLM response: {}", llm_response);

        // Execute the interpreted command
        self.command_executor.execute(&llm_response).await?;

        Ok(())
    }
}
