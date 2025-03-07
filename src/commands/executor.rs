use anyhow::{Result, Context};
use crate::fs::edit::{FileEditor, FileEdit};
use crate::git::commands::GitCommands;
use serde_json::{Value, from_str};
use std::path::PathBuf;
use std::process::Command;
use colored::Colorize;

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn execute(&self, llm_response: &str) -> Result<()> {
        // Parse the LLM response as JSON
        let action: Value = from_str(llm_response)
            .context("Failed to parse LLM response as JSON")?;
        
        match action.get("action").and_then(|a| a.as_str()) {
            Some("edit_file") => self.handle_edit_file(&action["details"])?,
            Some("answer_question") => self.handle_answer_question(&action["details"])?,
            Some("execute_command") => self.handle_execute_command(&action["details"]).await?,
            Some("git_operation") => self.handle_git_operation(&action["details"])?,
            _ => return Err(anyhow::anyhow!("Unknown or invalid action in LLM response")),
        }
        
        Ok(())
    }
    
    fn handle_edit_file(&self, details: &Value) -> Result<()> {
        let file_path = PathBuf::from(
            details.get("file_path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing file_path in edit_file action"))?
        );
        
        let edit_type = details.get("edit_type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing edit_type in edit_file action"))?;
        
        match edit_type {
            "replace" => {
                let start_line = details.get("start_line")
                    .and_then(|l| l.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing start_line in replace edit"))?;
                
                let end_line = details.get("end_line")
                    .and_then(|l| l.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing end_line in replace edit"))?;
                
                let new_text = details.get("new_text")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing new_text in replace edit"))?;
                
                let edit = FileEdit::Replace {
                    start_line: start_line as usize,
                    end_line: end_line as usize,
                    new_text: new_text.to_string(),
                };
                
                FileEditor::apply_edit(&file_path, &edit)?;
                
                println!("{} Replaced lines {}-{} in {}", 
                    "✓".bright_green(),
                    start_line,
                    end_line,
                    file_path.display()
                );
            },
            "insert" => {
                let line = details.get("line")
                    .and_then(|l| l.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing line in insert edit"))?;
                
                let text = details.get("text")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing text in insert edit"))?;
                
                let edit = FileEdit::Insert {
                    line: line as usize,
                    text: text.to_string(),
                };
                
                FileEditor::apply_edit(&file_path, &edit)?;
                
                println!("{} Inserted at line {} in {}", 
                    "✓".bright_green(),
                    line,
                    file_path.display()
                );
            },
            "delete" => {
                let start_line = details.get("start_line")
                    .and_then(|l| l.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing start_line in delete edit"))?;
                
                let end_line = details.get("end_line")
                    .and_then(|l| l.as_u64())
                    .ok_or_else(|| anyhow::anyhow!("Missing end_line in delete edit"))?;
                
                let edit = FileEdit::Delete {
                    start_line: start_line as usize,
                    end_line: end_line as usize,
                };
                
                FileEditor::apply_edit(&file_path, &edit)?;
                
                println!("{} Deleted lines {}-{} in {}", 
                    "✓".bright_green(),
                    start_line,
                    end_line,
                    file_path.display()
                );
            },
            _ => return Err(anyhow::anyhow!("Unknown edit_type: {}", edit_type)),
        }
        
        Ok(())
    }
    
    fn handle_answer_question(&self, details: &Value) -> Result<()> {
        let answer = details.get("answer")
            .and_then(|a| a.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing answer in answer_question action"))?;
        
        println!("\n{}", answer);
        
        Ok(())
    }
    
    async fn handle_execute_command(&self, details: &Value) -> Result<()> {
        let command_str = details.get("command")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing command in execute_command action"))?;
        
        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "bash"
        };
        
        let shell_arg = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };
        
        println!("{} Executing: {}", "▶".bright_blue(), command_str);
        
        let output = Command::new(shell)
            .arg(shell_arg)
            .arg(command_str)
            .output()
            .context("Failed to execute command")?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !stdout.is_empty() {
            println!("\n{}", stdout);
        }
        
        if !stderr.is_empty() {
            eprintln!("{} {}", "Error:".bright_red(), stderr);
        }
        
        if output.status.success() {
            println!("{} Command executed successfully", "✓".bright_green());
        } else {
            println!("{} Command failed with exit code: {:?}", 
                "✗".bright_red(),
                output.status.code()
            );
        }
        
        Ok(())
    }
    
    fn handle_git_operation(&self, details: &Value) -> Result<()> {
        let operation = details.get("operation")
            .and_then(|o| o.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing operation in git_operation action"))?;
        
        let current_dir = std::env::current_dir()?;
        
        match operation {
            "status" => {
                let status = GitCommands::status(&current_dir)?;
                println!("\n{}", status);
            },
            "commit" => {
                let message = details.get("message")
                    .and_then(|m| m.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing message in git commit operation"))?;
                
                let result = GitCommands::commit(&current_dir, message)?;
                println!("{} Successfully committed: {}", "✓".bright_green(), result);
            },
            "add" => {
                let files = details.get("files")
                    .and_then(|f| f.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing files in git add operation"))?;
                
                let file_strs: Vec<&str> = files
                    .iter()
                    .filter_map(|f| f.as_str())
                    .collect();
                
                let result = GitCommands::add(&current_dir, &file_strs)?;
                println!("{} Files added to staging area", "✓".bright_green());
            },
            _ => return Err(anyhow::anyhow!("Unknown git operation: {}", operation)),
        }
        
        Ok(())
    }
}
