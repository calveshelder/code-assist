use anyhow::Result;
use ignore::Walk;
use regex::Regex;
use std::path::{Path, PathBuf};

pub struct CodeSearch;

impl CodeSearch {
    pub fn new() -> Self {
        Self
    }
    
    pub fn find_relevant_files(&self, base_path: &Path, keywords: &[String]) -> Result<Vec<PathBuf>> {
        let mut relevant_files = Vec::new();
        
        if keywords.is_empty() {
            return Ok(relevant_files);
        }
        
        for entry in Walk::new(base_path) {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                // Skip non-files
                if !path.is_file() {
                    continue;
                }
                
                // Skip binary files and large files
                if self.is_binary_or_large_file(path)? {
                    continue;
                }
                
                // Read file content
                if let Ok(content) = std::fs::read_to_string(path) {
                    // Check if any keyword matches
                    let relevance = self.calculate_relevance(&content, keywords);
                    
                    if relevance > 0 {
                        relevant_files.push(path.to_owned());
                    }
                }
            }
        }
        
        // Sort by relevance (most relevant first)
        relevant_files.sort_by(|a, b| {
            if let (Ok(content_a), Ok(content_b)) = (std::fs::read_to_string(a), std::fs::read_to_string(b)) {
                let relevance_a = self.calculate_relevance(&content_a, keywords);
                let relevance_b = self.calculate_relevance(&content_b, keywords);
                relevance_b.cmp(&relevance_a)
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        Ok(relevant_files)
    }
    
    pub fn search_in_files(&self, base_path: &Path, pattern: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let regex = Regex::new(pattern)?;
        
        for entry in Walk::new(base_path) {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                // Skip non-files
                if !path.is_file() {
                    continue;
                }
                
                // Skip binary files and large files
                if self.is_binary_or_large_file(path)? {
                    continue;
                }
                
                // Read file content
                if let Ok(content) = std::fs::read_to_string(path) {
                    // Find all matches
                    for (line_idx, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            results.push(SearchResult {
                                file_path: path.to_path_buf(),
                                line_number: line_idx + 1,
                                line_content: line.to_string(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    fn is_binary_or_large_file(&self, path: &Path) -> Result<bool> {
        // Get file extension
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        // Skip common binary file extensions
        let binary_extensions = [
            "exe", "dll", "obj", "bin", "so", "dylib", "a", "o", "class",
            "pyc", "pyd", "jpg", "jpeg", "png", "gif", "bmp", "ico", "svg",
            "pdf", "zip", "tar", "gz", "tgz", "rar", "7z", "jar", "war",
        ];
        
        if binary_extensions.contains(&extension) {
            return Ok(true);
        }
        
        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > 1024 * 1024 {  // Skip files larger than 1MB
            return Ok(true);
        }
        
        Ok(false)
    }
    
    fn calculate_relevance(&self, content: &str, keywords: &[String]) -> usize {
        let mut score = 0;
        let content_lower = content.to_lowercase();
        
        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            let count = content_lower.matches(&keyword_lower).count();
            score += count;
        }
        
        score
    }
}

#[derive(Debug)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
}
