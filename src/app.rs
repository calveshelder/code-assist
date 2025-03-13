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
            let input_trimmed = input.trim();

            if input_trimmed.to_lowercase() == "exit" {
                break;
            }

            // Handle special commands
            if input_trimmed == "/init" {
                let cwd = std::env::current_dir()?;
                let memory = crate::memory::ProjectMemory::new();
                memory.init_caulk_file(&cwd)?;
                continue;
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
        let context = self.gather_context(command)?;
        
        // Send to LLM for interpretation
        let llm_response = self.llm_client.process_command(command, &context).await
            .context("Failed to process command with LLM")?;
        
        // Execute the interpreted command
        self.command_executor.execute(&llm_response).await?;
        
        Ok(())
    }
    
    // New method to gather context with project memory
    fn gather_context(&self, command: &str) -> Result<String> {
        // Load project memory (returns a new instance without modifying self)
        let loaded_memory = self.context_manager.project_memory.load()?;
        
        // Start building context
        let mut context = String::new();
        
        // Add project memory if available
        let memory = loaded_memory.get_memory();
        if !memory.is_empty() {
            context.push_str("# Project Memory\n");
            context.push_str(memory);
            context.push_str("\n\n");
        }
        
        // Get the regular code context
        let code_context = self.context_manager.gather_context(command)?;
        context.push_str(&code_context);
        
        Ok(context)
    }
}
