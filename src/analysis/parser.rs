use anyhow::Result;
use std::path::Path;

pub struct CodeParser;

impl CodeParser {
    pub fn analyze_file_structure(&self, file_path: &Path) -> Result<FileStructure> {
        let content = std::fs::read_to_string(file_path)?;
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let structure = match extension {
            "rs" => self.analyze_rust_file(&content),
            "py" => self.analyze_python_file(&content),
            "js" | "ts" => self.analyze_javascript_file(&content),
            _ => self.analyze_generic_file(&content),
        }?;
        
        Ok(structure)
    }
    
    fn analyze_rust_file(&self, content: &str) -> Result<FileStructure> {
        // A simple analysis - in a real implementation, use tree-sitter for proper parsing
        let mut modules = Vec::new();
        let mut structs = Vec::new();
        let mut functions = Vec::new();
        
        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            
            if line.starts_with("mod ") && line.ends_with(';') {
                let name = line.strip_prefix("mod ").unwrap().strip_suffix(';').unwrap();
                modules.push(CodeElement {
                    name: name.to_string(),
                    kind: "module".to_string(),
                    line: line_idx + 1,
                });
            } else if line.starts_with("struct ") && line.contains('{') {
                let name = line.strip_prefix("struct ").unwrap().split_whitespace().next().unwrap();
                structs.push(CodeElement {
                    name: name.to_string(),
                    kind: "struct".to_string(),
                    line: line_idx + 1,
                });
            } else if line.starts_with("fn ") {
                if let Some(name) = line.strip_prefix("fn ").unwrap().split('(').next() {
                    let name = name.trim();
                    functions.push(CodeElement {
                        name: name.to_string(),
                        kind: "function".to_string(),
                        line: line_idx + 1,
                    });
                }
            }
        }
        
        Ok(FileStructure {
            elements: {
                let mut combined = Vec::new();
                combined.extend(modules);
                combined.extend(structs);
                combined.extend(functions);
                combined
            },
        })
    }
    
    fn analyze_python_file(&self, content: &str) -> Result<FileStructure> {
        // Simplified Python file analysis
        let mut classes = Vec::new();
        let mut functions = Vec::new();
        
        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            
            if line.starts_with("class ") {
                if let Some(name) = line.strip_prefix("class ").unwrap().split('(').next() {
                    let name = name.split(':').next().unwrap_or(name).trim();
                    classes.push(CodeElement {
                        name: name.to_string(),
                        kind: "class".to_string(),
                        line: line_idx + 1,
                    });
                }
            } else if line.starts_with("def ") {
                if let Some(name) = line.strip_prefix("def ").unwrap().split('(').next() {
                    let name = name.trim();
                    functions.push(CodeElement {
                        name: name.to_string(),
                        kind: "function".to_string(),
                        line: line_idx + 1,
                    });
                }
            }
        }
        
        Ok(FileStructure {
            elements: {
                let mut combined = Vec::new();
                combined.extend(classes);
                combined.extend(functions);
                combined
            },
        })
    }
    
    fn analyze_javascript_file(&self, content: &str) -> Result<FileStructure> {
        // Simplified JavaScript file analysis
        let mut classes = Vec::new();
        let mut functions = Vec::new();
        
        for (line_idx, line) in content.lines().enumerate() {
            let line = line.trim();
            
            if line.starts_with("class ") {
                if let Some(name) = line.strip_prefix("class ").unwrap().split(' ').next() {
                    let name = name.split('{').next().unwrap_or(name).trim();
                    classes.push(CodeElement {
                        name: name.to_string(),
                        kind: "class".to_string(),
                        line: line_idx + 1,
                    });
                }
            } else if line.starts_with("function ") {
                if let Some(name) = line.strip_prefix("function ").unwrap().split('(').next() {
                    let name = name.trim();
                    functions.push(CodeElement {
                        name: name.to_string(),
                        kind: "function".to_string(),
                        line: line_idx + 1,
                    });
                }
            }
        }
        
        Ok(FileStructure {
            elements: {
                let mut combined = Vec::new();
                combined.extend(classes);
                combined.extend(functions);
                combined
            },
        })
    }
    
    fn analyze_generic_file(&self, content: &str) -> Result<FileStructure> {
        // Very basic analysis for unknown file types
        Ok(FileStructure {
            elements: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub struct FileStructure {
    pub elements: Vec<CodeElement>,
}

#[derive(Debug)]
pub struct CodeElement {
    pub name: String,
    pub kind: String,
    pub line: usize,
}
