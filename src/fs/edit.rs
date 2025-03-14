use anyhow::{Result, Context};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileEditor;

impl FileEditor {
    pub fn read_file(path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))
    }
    
    pub fn write_file(path: &Path, content: &str) -> Result<()> {
        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }
    
    pub fn find_and_replace(
        path: &Path,
        pattern: &str,
        replacement: &str,
        use_regex: bool
    ) -> Result<(String, usize)> {
        let content = Self::read_file(path)?;
        
        if use_regex {
            let regex = regex::Regex::new(pattern)
                .with_context(|| format!("Invalid regex pattern: {}", pattern))?;
            
            let new_content = regex.replace_all(&content, replacement)
                .into_owned();
            let count = regex.find_iter(&content).count();
            
            Ok((new_content, count))
        } else {
            let count = content.matches(pattern).count();
            let new_content = content.replace(pattern, replacement);
            
            Ok((new_content, count))
        }
    }
    
    pub fn apply_edit(path: &Path, edit: &FileEdit) -> Result<()> {
        let content = Self::read_file(path)?;
        
        let new_content = match edit {
            FileEdit::Replace { start_line, end_line, new_text } => {
                Self::replace_lines(&content, *start_line, *end_line, new_text)
            },
            FileEdit::Insert { line, text } => {
                Self::insert_at_line(&content, *line, text)
            },
            FileEdit::Delete { start_line, end_line } => {
                Self::delete_lines(&content, *start_line, *end_line)
            },
        }?;
        
        Self::write_file(path, &new_content)
    }
    
    fn replace_lines(content: &str, start_line: usize, end_line: usize, new_text: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        
        if start_line == 0 || start_line > lines.len() || end_line < start_line || end_line > lines.len() {
            return Err(anyhow::anyhow!("Invalid line range: {}-{}", start_line, end_line));
        }
        
        let mut result = String::new();
        
        // Add lines before the replacement
        for (_i, line) in lines.iter().enumerate().take(start_line - 1) {
            result.push_str(line);
            result.push('\n');
        }
        
        // Add the replacement text
        result.push_str(new_text);
        if !new_text.ends_with('\n') {
            result.push('\n');
        }
        
        // Add lines after the replacement
        for line in lines.iter().skip(end_line) {
            result.push_str(line);
            result.push('\n');
        }
        
        Ok(result)
    }
    
    fn insert_at_line(content: &str, line_num: usize, text: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        
        if line_num == 0 || line_num > lines.len() + 1 {
            return Err(anyhow::anyhow!("Invalid line number: {}", line_num));
        }
        
        let mut result = String::new();
        
        // Add lines before the insertion point
        for (_i, line) in lines.iter().enumerate().take(line_num - 1) {
            result.push_str(line);
            result.push('\n');
        }
        
        // Add the insertion text
        result.push_str(text);
        if !text.ends_with('\n') {
            result.push('\n');
        }
        
        // Add lines after the insertion point
        for line in lines.iter().skip(line_num - 1) {
            result.push_str(line);
            result.push('\n');
        }
        
        Ok(result)
    }
    
    fn delete_lines(content: &str, start_line: usize, end_line: usize) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        
        if start_line == 0 || start_line > lines.len() || end_line < start_line || end_line > lines.len() {
            return Err(anyhow::anyhow!("Invalid line range: {}-{}", start_line, end_line));
        }
        
        let mut result = String::new();
        
        // Add lines before the deletion
        for (_i, line) in lines.iter().enumerate().take(start_line - 1) {
            result.push_str(line);
            result.push('\n');
        }
        
        // Skip the deleted lines
        
        // Add lines after the deletion
        for line in lines.iter().skip(end_line) {
            result.push_str(line);
            result.push('\n');
        }
        
        Ok(result)
    }
}

pub enum FileEdit {
    Replace {
        start_line: usize,
        end_line: usize,
        new_text: String,
    },
    Insert {
        line: usize,
        text: String,
    },
    Delete {
        start_line: usize,
        end_line: usize,
    },
}
