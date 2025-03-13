use crate::fs::search::CodeSearch;
use anyhow::Result;
use std::path::Path;
use crate::memory::ProjectMemory;

pub struct ContextManager {
    code_search: CodeSearch,
    pub project_memory: ProjectMemory,  // Made public
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            code_search: CodeSearch::new(),
            project_memory: ProjectMemory::new(),
        }
    }
    
    pub fn gather_context(&self, command: &str) -> Result<String> {
        // No longer trying to load project memory here
        // That's now handled in App::gather_context
        
        // Get relevant file content based on keywords
        let mut context = String::new();
        
        // Analyze the command to determine what context is needed
        let keywords = self.extract_keywords(command);
        
        // Get the current working directory
        let cwd = std::env::current_dir()?;

        // Add workspace information
        context.push_str(&format!("Working directory: {}\n\n", cwd.display()));
        
        // Find relevant files
        let relevant_files = self.code_search.find_relevant_files(&cwd, &keywords)?;
        
        // Add file contents or summaries to context
        for file_path in relevant_files.iter().take(3) {  // Limit to top 3 files to avoid context explosion
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let relative_path = file_path.strip_prefix(&cwd).unwrap_or(file_path);
                context.push_str(&format!("File: {}\n", relative_path.display()));
                
                // Include only first ~500 chars to avoid overly large contexts
                let preview = if content.len() > 500 {
                    format!("{}... (truncated)", &content[..500])
                } else {
                    content
                };
                
                context.push_str(&format!("{}\n\n", preview));
            }
        }
        
        // Add git status if relevant
        if command.contains("git") || command.contains("commit") || command.contains("merge") {
            if let Ok(git_status) = self.get_git_status(&cwd) {
                context.push_str(&format!("Git status:\n{}\n\n", git_status));
            }
        }
        
        Ok(context)
    }
    
    fn extract_keywords(&self, command: &str) -> Vec<String> {
        // Simple keyword extraction - in a real implementation this would be more sophisticated
        command
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .collect()
    }
    
    fn get_git_status(&self, path: &Path) -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("git")
            .current_dir(path)
            .args(&["status", "--short"])
            .output()?;
        
        if output.status.success() {
            let git_status = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(git_status)
        } else {
            Ok("Not a git repository or git command failed".to_string())
        }
    }
}
