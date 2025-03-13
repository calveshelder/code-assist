// src/memory/mod.rs
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use colored::Colorize;

#[derive(Default, Clone)]
pub struct ProjectMemory {
    // Stores the combined content of all relevant CAULK.md files
    combined_memory: String,
    // Tracks which files have been loaded
    loaded_files: Vec<PathBuf>,
}

impl ProjectMemory {
    /// Creates a new ProjectMemory instance
    pub fn new() -> Self {
        Self {
            combined_memory: String::new(),
            loaded_files: Vec::new(),
        }
    }

    /// Loads all relevant CAULK.md files for the current working directory
    /// Returns a new instance with the loaded memory (doesn't modify self)
    pub fn load(&self) -> Result<Self> {
        let mut result = Self::new();

        // 1. Try to load from ~/.caulk/CAULK.md (user-specific)
        if let Some(home_dir) = dirs::home_dir() {
            let user_caulk_path = home_dir.join(".caulk").join("CAULK.md");
            if user_caulk_path.exists() {
                result.load_file(&user_caulk_path)?;
            }
        }

        // 2. Load from current directory and any parent directories
        let cwd = std::env::current_dir()?;
        result.load_directory_and_parents(&cwd)?;

        // 3. Look for CAULK.md in subdirectories of current directory
        // (we don't automatically load these, but we track them for reference)
        result.find_subdirectory_files(&cwd)?;

        Ok(result)
    }

    /// Loads a specific CAULK.md file and adds its content to the memory
    fn load_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read memory file: {}", path.display()))?;
        
        // Add file info and content to the combined memory
        self.combined_memory.push_str(&format!("\n## From: {}\n\n", path.display()));
        self.combined_memory.push_str(&content);
        self.combined_memory.push_str("\n\n");
        
        // Track that we've loaded this file
        self.loaded_files.push(path.to_path_buf());
        
        Ok(())
    }

    /// Recursively loads CAULK.md from the current directory and all parent directories
    fn load_directory_and_parents(&mut self, dir: &Path) -> Result<()> {
        let caulk_path = dir.join("CAULK.md");
        if caulk_path.exists() {
            self.load_file(&caulk_path)?;
        }
        
        // Recursively check parent directories
        if let Some(parent) = dir.parent() {
            self.load_directory_and_parents(parent)?;
        }
        
        Ok(())
    }

    /// Finds but doesn't load CAULK.md files in subdirectories
    fn find_subdirectory_files(&mut self, dir: &Path) -> Result<()> {
        for entry in walkdir::WalkDir::new(dir)
            .min_depth(1) // Skip the root dir
            .max_depth(3) // Don't go too deep
            .into_iter()
            .filter_map(|e| e.ok()) {
                
            if entry.file_type().is_file() && entry.file_name() == "CAULK.md" {
                // Don't load, just track for reference
                self.loaded_files.push(entry.path().to_path_buf());
            }
        }
        
        Ok(())
    }

    /// Returns the combined memory content
    pub fn get_memory(&self) -> &str {
        &self.combined_memory
    }

    /// Returns a list of all tracked CAULK.md files
    pub fn get_loaded_files(&self) -> &[PathBuf] {
        &self.loaded_files
    }

    /// Initializes a new CAULK.md file in the specified directory
    pub fn init_caulk_file(&self, dir: &Path) -> Result<()> {
        let caulk_path = dir.join("CAULK.md");
        
        if caulk_path.exists() {
            println!("{} {} already exists", "!".yellow(), caulk_path.display());
            return Ok(());
        }
        
        let template = r#"# Project Memory for CodeAssist

## Project Overview
<!-- Provide a brief description of the project -->

## Frequently Used Commands
```
# Build the project
cargo build

# Run tests
cargo test

# Run linting
cargo clippy
```

## Code Conventions
<!-- Document your code style, naming conventions, etc. -->

## Architecture
<!-- Describe important architectural patterns in your project -->

## Important Notes
<!-- Any other information that would be helpful for working with this codebase -->
"#;
        
        fs::write(&caulk_path, template)
            .with_context(|| format!("Failed to create CAULK.md at {}", caulk_path.display()))?;
            
        println!("{} Created project memory file at {}", "✓".green(), caulk_path.display());
        
        Ok(())
    }
}
