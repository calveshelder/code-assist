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
        
        // Use a map to store path and relevance for sorting
        let mut path_relevance: Vec<(PathBuf, usize)> = Vec::new();
        
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
                        path_relevance.push((path.to_owned(), relevance));
                    }
                }
            }
        }
        
        // Sort by relevance (most relevant first)
        path_relevance.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Extract sorted paths
        relevant_files = path_relevance.into_iter().map(|(path, _)| path).collect();
        
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
        
        // Detect file language and framework signatures
        let lang_signatures = self.detect_language_signatures(&content_lower);
        
        // Calculate basic keyword score
        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            let count = content_lower.matches(&keyword_lower).count();
            
            // Check if this keyword corresponds to a file's language
            let language_match = self.get_language_boost(&keyword_lower, &lang_signatures);
            
            // Apply language-specific boost if the keyword matches the file language
            if language_match > 1 {
                score += count * language_match;
            } else {
                // Default scoring for non-language-specific matches
                score += count;
            }
        }
        
        // Apply general boosts based on search keywords
        let search_language = self.detect_search_language(keywords);
        
        // Boost score if file language matches the search language
        if let Some(lang) = search_language {
            match lang {
                SearchLanguage::Rust => {
                    if lang_signatures.is_rust {
                        score += 25;
                    }
                },
                SearchLanguage::Python => {
                    if lang_signatures.is_python {
                        score += 25;
                    }
                },
                SearchLanguage::JavaScript => {
                    if lang_signatures.is_javascript {
                        score += 20;
                    }
                    if lang_signatures.is_angular {
                        score += 25;
                    }
                    if lang_signatures.is_react {
                        score += 25;
                    }
                },
                SearchLanguage::PHP => {
                    if lang_signatures.is_php {
                        score += 25;
                    }
                },
                SearchLanguage::Drupal => {
                    if lang_signatures.is_drupal {
                        score += 30;
                    }
                    if lang_signatures.is_drupal_info {
                        score += 35;
                    }
                    if lang_signatures.is_drupal_services {
                        score += 35;
                    }
                    if lang_signatures.is_drupal_template {
                        score += 25;
                    }
                    
                    // Special handling for Drupal component searches
                    let component_search = keywords.iter().any(|k| {
                        let kl = k.to_lowercase();
                        kl.contains("plugin") || kl.contains("block") || kl.contains("field") || 
                        kl.contains("form") || kl.contains("controller") || kl.contains("entity")
                    });
                    
                    if component_search {
                        if content_lower.contains("\\plugin\\") {
                            score += 40;
                        }
                        if content_lower.contains("\\form\\") {
                            score += 40;
                        }
                        if content_lower.contains("\\entity\\") {
                            score += 40;
                        }
                    }
                },
                SearchLanguage::Go => {
                    if lang_signatures.is_go {
                        score += 25;
                    }
                },
                SearchLanguage::Generic => {
                    // No specific boost for generic searches
                }
            }
            
            // Penalize mismatches between search language and file language
            if let SearchLanguage::Drupal = lang {
                if lang_signatures.is_javascript && !content_lower.contains("drupal") {
                    score = score / 2;
                }
            }
        }
        
        score
    }
    
    /// Detects language signatures from file content
    fn detect_language_signatures(&self, content: &str) -> LanguageSignatures {
        let mut signatures = LanguageSignatures::default();
        
        // Rust signatures
        signatures.is_rust = content.contains("fn ") && 
                            (content.contains("struct ") || 
                             content.contains("impl ") || 
                             content.contains("pub ") ||
                             content.contains("use std::") ||
                             content.contains("mod "));
        
        // Python signatures
        signatures.is_python = content.contains("def ") && 
                              (content.contains("import ") || 
                               content.contains("class ") ||
                               content.contains("if __name__ == ") ||
                               content.contains("self."));
                               
        // PHP signatures
        signatures.is_php = content.contains("<?php") || 
                           (content.contains("namespace") && content.contains(";")) || 
                           (content.contains("use ") && content.contains("\\") && content.contains(";"));
        
        // JavaScript signatures
        signatures.is_javascript = content.contains("function") && 
                                  (content.contains("var ") || 
                                   content.contains("let ") || 
                                   content.contains("const ") ||
                                   content.contains("import ") ||
                                   content.contains("export "));
        
        // Angular signatures
        signatures.is_angular = content.contains("@component") || 
                               content.contains("@injectable") || 
                               content.contains("@ngmodule");
        
        // React signatures
        signatures.is_react = content.contains("react") &&
                             (content.contains("component") ||
                              content.contains("render") ||
                              content.contains("jsx") ||
                              content.contains("</>"));
        
        // Go signatures
        signatures.is_go = content.contains("package ") && 
                          (content.contains("func ") || 
                           content.contains("import (") ||
                           content.contains("type ") && content.contains("struct {"));
        
        // Drupal signatures
        signatures.is_drupal = content.contains("drupal") || 
                              content.contains("hook_") || 
                              content.contains("module_implements") ||
                              content.contains("@plugin") ||
                              content.contains("pluginbase") ||
                              content.contains("\\plugin\\") ||
                              content.contains("\\form\\") ||
                              content.contains("\\entity\\") ||
                              content.contains("drupalconsole") ||
                              content.contains("@implements");
        
        // Drupal info file
        signatures.is_drupal_info = content.contains("type: module") ||
                                   content.contains("core_version_requirement") ||
                                   content.contains("core: ");
        
        // Drupal services file
        signatures.is_drupal_services = content.contains("services:") && 
                                       content.contains("class:");
        
        // Drupal template
        signatures.is_drupal_template = content.contains("{{ content }}") || 
                                       content.contains("{{ attach_library") ||
                                       content.contains("{{ 'drupal");
        
        signatures
    }
    
    /// Gets a language-specific boost factor for a keyword
    fn get_language_boost(&self, keyword: &str, signatures: &LanguageSignatures) -> usize {
        if signatures.is_rust && (keyword == "rust" || keyword.contains("fn ") || keyword.contains("struct ") || keyword.contains("impl ")) {
            3
        } else if signatures.is_python && (keyword == "python" || keyword.contains("def ") || keyword.contains("import ") || keyword.contains("class ")) {
            3
        } else if signatures.is_php && (keyword == "php" || keyword.contains("php")) {
            3
        } else if signatures.is_drupal && (keyword == "drupal" || keyword.contains("drupal") || keyword.contains("module")) {
            4
        } else if signatures.is_drupal_info && (keyword.contains("info") || keyword.contains("configuration")) {
            5
        } else if signatures.is_drupal_services && (keyword.contains("service") || keyword.contains("dependency")) {
            5
        } else if signatures.is_drupal_template && (keyword.contains("template") || keyword.contains("twig")) {
            5
        } else if signatures.is_javascript && (keyword.contains("js") || keyword.contains("javascript")) {
            3
        } else if signatures.is_angular && (keyword.contains("angular") || keyword.contains("component") || keyword.contains("service")) {
            4
        } else if signatures.is_react && (keyword.contains("react") || keyword.contains("component") || keyword.contains("jsx")) {
            4
        } else if signatures.is_go && (keyword == "go" || keyword.contains("golang") || keyword.contains("func ")) {
            3
        } else {
            1 // No boost
        }
    }
    
    /// Detects the primary language being searched for based on keywords
    fn detect_search_language(&self, keywords: &[String]) -> Option<SearchLanguage> {
        // Count language-specific keywords
        let mut rust_count = 0;
        let mut python_count = 0;
        let mut php_count = 0;
        let mut drupal_count = 0;
        let mut js_count = 0;
        let mut go_count = 0;
        
        for keyword in keywords {
            let k = keyword.to_lowercase();
            
            // Rust
            if k.contains("rust") || k.contains("cargo") || k.contains("crate") || k.contains("fn ") {
                rust_count += 1;
            }
            
            // Python
            if k.contains("python") || k.contains("django") || k.contains("flask") || k.contains("def ") {
                python_count += 1;
            }
            
            // PHP (basic)
            if k.contains("php") {
                php_count += 1;
            }
            
            // Drupal (specific PHP framework)
            if k.contains("drupal") || k.contains("hook_") || k.contains("module") || 
               k.contains("block") || k.contains("entity") || k.contains("field") || 
               k.contains("form") || k.contains("plugin") || k.contains("token") {
                drupal_count += 1;
            }
            
            // JavaScript and related
            if k.contains("js") || k.contains("javascript") || k.contains("angular") || 
               k.contains("react") || k.contains("node") || k.contains("component") || 
               k.contains("directive") {
                js_count += 1;
            }
            
            // Go
            if k.contains("go") || k.contains("golang") || k.contains("func ") {
                go_count += 1;
            }
        }
        
        // Determine which language has the most keyword matches
        let max_count = [rust_count, python_count, php_count, drupal_count, js_count, go_count]
            .iter()
            .max()
            .cloned()
            .unwrap_or(0);
        
        // Only return a language if we have at least one keyword match
        if max_count > 0 {
            if drupal_count == max_count { // Prioritize more specific frameworks
                Some(SearchLanguage::Drupal)
            } else if rust_count == max_count {
                Some(SearchLanguage::Rust)
            } else if python_count == max_count {
                Some(SearchLanguage::Python)
            } else if js_count == max_count {
                Some(SearchLanguage::JavaScript)
            } else if php_count == max_count {
                Some(SearchLanguage::PHP)
            } else if go_count == max_count {
                Some(SearchLanguage::Go)
            } else {
                Some(SearchLanguage::Generic)
            }
        } else {
            Some(SearchLanguage::Generic)
        }
    }
}

#[derive(Debug, Default)]
struct LanguageSignatures {
    is_rust: bool,
    is_python: bool,
    is_php: bool,
    is_javascript: bool,
    is_go: bool,
    is_angular: bool,
    is_react: bool,
    is_drupal: bool,
    is_drupal_info: bool,
    is_drupal_services: bool,
    is_drupal_template: bool,
}

enum SearchLanguage {
    Rust,
    Python,
    JavaScript,
    PHP,
    Drupal,
    Go,
    Generic,
}

#[derive(Debug)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
}
