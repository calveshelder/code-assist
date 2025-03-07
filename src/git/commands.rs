use anyhow::{Result, Context};
use std::path::Path;
use std::process::Command;

pub struct GitCommands;

impl GitCommands {
    pub fn status(repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(&["status"])
            .output()
            .context("Failed to execute git status")?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Git status failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    pub fn commit(repo_path: &Path, message: &str) -> Result<String> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(&["commit", "-m", message])
            .output()
            .context("Failed to execute git commit")?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Git commit failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    pub fn add(repo_path: &Path, files: &[&str]) -> Result<String> {
        let mut args = vec!["add"];
        args.extend(files);
        
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(&args)
            .output()
            .context("Failed to execute git add")?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Git add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    // Add more git commands as needed...
}
