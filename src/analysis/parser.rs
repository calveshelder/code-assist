use anyhow::Result;
use std::path::Path;
use regex::Regex;

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
            "js" | "ts" | "jsx" | "tsx" => self.analyze_javascript_file(&content),
            "php" => self.analyze_php_file(&content),
            "go" => self.analyze_go_file(&content),
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
                    description: None,
                    metadata: None,
                });
            } else if line.starts_with("struct ") && line.contains('{') {
                let name = line.strip_prefix("struct ").unwrap().split_whitespace().next().unwrap();
                structs.push(CodeElement {
                    name: name.to_string(),
                    kind: "struct".to_string(),
                    line: line_idx + 1,
                    description: None,
                    metadata: None,
                });
            } else if line.starts_with("fn ") {
                if let Some(name) = line.strip_prefix("fn ").unwrap().split('(').next() {
                    let name = name.trim();
                    functions.push(CodeElement {
                        name: name.to_string(),
                        kind: "function".to_string(),
                        line: line_idx + 1,
                        description: None,
                        metadata: None,
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
            is_drupal: false,
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
                        description: None,
                        metadata: None,
                    });
                }
            } else if line.starts_with("def ") {
                if let Some(name) = line.strip_prefix("def ").unwrap().split('(').next() {
                    let name = name.trim();
                    functions.push(CodeElement {
                        name: name.to_string(),
                        kind: "function".to_string(),
                        line: line_idx + 1,
                        description: None,
                        metadata: None,
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
            is_drupal: false,
        })
    }
    
    fn analyze_javascript_file(&self, content: &str) -> Result<FileStructure> {
        // Enhanced JavaScript/TypeScript file analysis with React and Angular support
        let mut classes = Vec::new();
        let mut functions = Vec::new();
        let mut components = Vec::new();
        let mut hooks = Vec::new();
        
        // Check for React or Angular signatures
        let is_react = content.contains("import React") || 
                       content.contains("from 'react'") || 
                       content.contains("extends React.Component") ||
                       content.contains("<React.") ||
                       content.contains("<>");
                   
        let is_angular = content.contains("@Component") || 
                         content.contains("@NgModule") || 
                         content.contains("@Injectable") ||
                         content.contains("@Input(") ||
                         content.contains("@Output(");
        
        let lines: Vec<&str> = content.lines().collect();
        
        for line_idx in 0..lines.len() {
            let line = lines[line_idx].trim();
            
            // React and Angular detection
            if is_angular && line.contains("@Component") {
                // Look ahead for component class name
                for i in line_idx..std::cmp::min(line_idx + 5, lines.len()) {
                    let l = lines[i].trim();
                    if l.starts_with("class ") {
                        if let Some(name) = l.strip_prefix("class ").unwrap().split(' ').next() {
                            let name = name.split('{').next().unwrap_or(name).trim();
                            components.push(CodeElement {
                                name: name.to_string(),
                                kind: "angular_component".to_string(),
                                line: line_idx + 1,
                                description: None,
                                metadata: Some(ElementMetadata {
                                    is_plugin: false,
                                    plugin_type: None,
                                    is_service: false,
                                    service_tags: Vec::new(),
                                    is_hook: false,
                                    hook_name: None,
                                    annotations: vec!["@Component".to_string()],
                                    namespace: None,
                                }),
                            });
                            break;
                        }
                    }
                }
            } else if is_angular && line.contains("@Injectable") {
                // Look ahead for service class name
                for i in line_idx..std::cmp::min(line_idx + 5, lines.len()) {
                    let l = lines[i].trim();
                    if l.starts_with("class ") {
                        if let Some(name) = l.strip_prefix("class ").unwrap().split(' ').next() {
                            let name = name.split('{').next().unwrap_or(name).trim();
                            components.push(CodeElement {
                                name: name.to_string(),
                                kind: "angular_service".to_string(),
                                line: line_idx + 1,
                                description: None,
                                metadata: Some(ElementMetadata {
                                    is_plugin: false,
                                    plugin_type: None,
                                    is_service: true,
                                    service_tags: Vec::new(),
                                    is_hook: false,
                                    hook_name: None,
                                    annotations: vec!["@Injectable".to_string()],
                                    namespace: None,
                                }),
                            });
                            break;
                        }
                    }
                }
            } else if is_react && line.contains("class ") && line.contains("extends React.Component") {
                // React class component
                if let Some(name) = line.strip_prefix("class ").unwrap().split(' ').next() {
                    let name = name.split('{').next().unwrap_or(name).trim();
                    components.push(CodeElement {
                        name: name.to_string(),
                        kind: "react_component".to_string(),
                        line: line_idx + 1,
                        description: None,
                        metadata: None,
                    });
                }
            } else if is_react && (line.contains("function ") || line.contains("const ")) && content[line_idx..].contains("return (") {
                // React functional component (simple heuristic)
                let name = if line.starts_with("function ") {
                    line.strip_prefix("function ").unwrap().split('(').next().map(|s| s.trim())
                } else if line.starts_with("const ") && line.contains(" = ") {
                    line.strip_prefix("const ").unwrap().split(" = ").next().map(|s| s.trim())
                } else {
                    None
                };
                
                if let Some(name) = name {
                    // Check if it's actually a component by seeing if it returns JSX
                    let mut has_jsx = false;
                    for i in line_idx..std::cmp::min(line_idx + 20, lines.len()) {
                        let check_line = lines[i].trim();
                        if check_line.contains("return (") && (
                            check_line.contains("<") || 
                            i + 1 < lines.len() && lines[i+1].contains("<")) {
                            has_jsx = true;
                            break;
                        }
                    }
                    
                    if has_jsx {
                        components.push(CodeElement {
                            name: name.to_string(),
                            kind: "react_component".to_string(),
                            line: line_idx + 1,
                            description: None,
                            metadata: None,
                        });
                    }
                }
            } else if is_react && line.contains("use") && line.starts_with("function ") {
                // React hook
                if let Some(name) = line.strip_prefix("function ").unwrap().split('(').next() {
                    let name = name.trim();
                    if name.starts_with("use") {
                        hooks.push(CodeElement {
                            name: name.to_string(),
                            kind: "react_hook".to_string(),
                            line: line_idx + 1,
                            description: None,
                            metadata: None,
                        });
                    }
                }
            } else if line.starts_with("class ") {
                // Regular class
                if let Some(name) = line.strip_prefix("class ").unwrap().split(' ').next() {
                    let name = name.split('{').next().unwrap_or(name).trim();
                    // Skip if already added as a component
                    if !components.iter().any(|c| c.name == name) {
                        classes.push(CodeElement {
                            name: name.to_string(),
                            kind: "class".to_string(),
                            line: line_idx + 1,
                            description: None,
                            metadata: None,
                        });
                    }
                }
            } else if line.starts_with("function ") {
                // Regular function
                if let Some(name) = line.strip_prefix("function ").unwrap().split('(').next() {
                    let name = name.trim();
                    // Skip if already added as a component or hook
                    if !components.iter().any(|c| c.name == name) && 
                       !hooks.iter().any(|h| h.name == name) {
                        functions.push(CodeElement {
                            name: name.to_string(),
                            kind: "function".to_string(),
                            line: line_idx + 1,
                            description: None,
                            metadata: None,
                        });
                    }
                }
            } else if line.starts_with("const ") && line.contains(" = (") && line.contains("=>") {
                // Arrow function
                if let Some(name) = line.strip_prefix("const ").unwrap().split(" = ").next() {
                    let name = name.trim();
                    // Skip if already added as a component
                    if !components.iter().any(|c| c.name == name) {
                        functions.push(CodeElement {
                            name: name.to_string(),
                            kind: "function".to_string(),
                            line: line_idx + 1,
                            description: None,
                            metadata: None,
                        });
                    }
                }
            }
        }
        
        // Combine all elements
        let mut elements = Vec::new();
        elements.extend(components);
        elements.extend(hooks);
        elements.extend(classes);
        elements.extend(functions);
        
        Ok(FileStructure {
            elements,
            is_drupal: false,
        })
    }
    
    fn analyze_php_file(&self, content: &str) -> Result<FileStructure> {
        // Enhanced PHP file analysis for Drupal
        let mut elements = Vec::new();
        let mut current_namespace = None;
        let mut doc_comment_buffer = String::new();
        let mut in_doc_comment = false;
        let mut annotation_buffer = Vec::new();
        
        // Check if it's a Drupal file by common indicators
        let is_drupal_module = content.contains("Drupal\\") || 
                              content.contains("function") && content.contains("_hook_") || 
                              content.contains("@Implements") || content.contains("@implements") ||
                              content.contains("\\Plugin\\") || content.contains("services.yml");
        
        // Extract namespace from the file
        let namespace_regex = Regex::new(r"namespace\s+([^;]+);")?;
        if let Some(cap) = namespace_regex.captures(content) {
            if cap.len() >= 2 {
                current_namespace = Some(cap[1].to_string());
            }
        }
        
        // Lines to analyze
        let lines: Vec<&str> = content.lines().collect();
        
        for line_idx in 0..lines.len() {
            let line = lines[line_idx].trim();
            
            // Handle doc comments and annotations
            if line.starts_with("/**") {
                in_doc_comment = true;
                doc_comment_buffer.clear();
                annotation_buffer.clear();
                doc_comment_buffer.push_str(line);
            } else if in_doc_comment && line.contains("*/") {
                in_doc_comment = false;
                doc_comment_buffer.push_str(line);
            } else if in_doc_comment {
                doc_comment_buffer.push_str(line);
                
                // Extract annotations
                if line.trim().starts_with("@") {
                    let annotation = line.trim().to_string();
                    annotation_buffer.push(annotation);
                }
            }
            
            // Look for PHP classes
            else if line.starts_with("class ") {
                if let Some(class_def) = self.extract_class_definition(line, line_idx + 1, &lines, &doc_comment_buffer, &annotation_buffer, &current_namespace) {
                    elements.push(class_def);
                }
            } 
            // Look for PHP interfaces
            else if line.starts_with("interface ") {
                if let Some(name) = line.strip_prefix("interface ").unwrap().split(' ').next() {
                    let name = name.split('{').next().unwrap_or(name).trim();
                    elements.push(CodeElement {
                        name: name.to_string(),
                        kind: "interface".to_string(),
                        line: line_idx + 1,
                        description: self.extract_doc_comment_description(&doc_comment_buffer),
                        metadata: Some(ElementMetadata {
                            is_plugin: false,
                            plugin_type: None,
                            is_service: false,
                            service_tags: Vec::new(),
                            is_hook: false,
                            hook_name: None,
                            annotations: annotation_buffer.clone(),
                            namespace: current_namespace.clone(),
                        }),
                    });
                }
            }
            // Look for PHP functions
            else if line.starts_with("function ") {
                if let Some(function_def) = self.extract_function_definition(line, line_idx + 1, &lines, &doc_comment_buffer, &annotation_buffer, &current_namespace, is_drupal_module) {
                    elements.push(function_def);
                }
            }
            
            // If not in a doc comment, reset the buffer
            if !in_doc_comment && !line.starts_with("/**") {
                doc_comment_buffer.clear();
                annotation_buffer.clear();
            }
        }
        
        Ok(FileStructure {
            elements,
            is_drupal: is_drupal_module,
        })
    }
    
    /// Extracts class definition with Drupal-specific metadata
    fn extract_class_definition(&self, line: &str, line_idx: usize, lines: &[&str], 
                               doc_comment: &str, annotations: &[String], namespace: &Option<String>) -> Option<CodeElement> {
        if let Some(name) = line.strip_prefix("class ").unwrap().split(' ').next() {
            let name = name.split('{').next().unwrap_or(name).trim();
            
            // Check if this is a plugin by annotations
            let is_plugin = annotations.iter().any(|a| a.contains("@Plugin"));
            let plugin_type = if is_plugin {
                annotations.iter()
                    .find(|a| a.contains("@Plugin"))
                    .and_then(|a| {
                        let plugin_regex = Regex::new(r#"@Plugin\s*\(\s*id\s*=\s*["']([^"']+)["']"#).ok()?;
                        plugin_regex.captures(a).map(|cap| cap[1].to_string())
                    })
            } else {
                None
            };
            
            // Check if it's a service
            let is_service = doc_comment.contains("@Service") || doc_comment.contains("service");
            
            // Check class inheritance to determine if it's a plugin or service
            let is_plugin_by_inheritance = {
                // Look ahead for extends or implements lines
                for i in line_idx..std::cmp::min(line_idx + 5, lines.len()) {
                    let l = lines[i].trim();
                    if l.contains("extends") && (
                        l.contains("PluginBase") || 
                        l.contains("BlockBase") || 
                        l.contains("FieldItemBase") || 
                        l.contains("ConfigEntityBase")) {
                        return Some(CodeElement {
                            name: name.to_string(),
                            kind: "drupal_plugin".to_string(),
                            line: line_idx,
                            description: self.extract_doc_comment_description(doc_comment),
                            metadata: Some(ElementMetadata {
                                is_plugin: true,
                                plugin_type: Some(if l.contains("BlockBase") { 
                                    "Block".to_string() 
                                } else if l.contains("FieldItemBase") {
                                    "Field".to_string()
                                } else if l.contains("ConfigEntityBase") {
                                    "ConfigEntity".to_string()
                                } else {
                                    "Generic".to_string()
                                }),
                                is_service,
                                service_tags: Vec::new(),
                                is_hook: false,
                                hook_name: None,
                                annotations: annotations.to_vec(),
                                namespace: namespace.clone(),
                            }),
                        });
                    }
                }
                false
            };
            
            // Check if this class is in a Plugin namespace
            let is_plugin_by_namespace = namespace.as_ref().map_or(false, |ns| ns.contains("Plugin"));
            
            // Determine the kind based on all the checks
            let kind = if is_plugin || is_plugin_by_inheritance || is_plugin_by_namespace {
                "drupal_plugin"
            } else if is_service {
                "drupal_service"
            } else if namespace.as_ref().map_or(false, |ns| ns.contains("Drupal")) {
                "drupal_class"
            } else {
                "class"
            };
            
            return Some(CodeElement {
                name: name.to_string(),
                kind: kind.to_string(),
                line: line_idx,
                description: self.extract_doc_comment_description(doc_comment),
                metadata: Some(ElementMetadata {
                    is_plugin: is_plugin || is_plugin_by_inheritance || is_plugin_by_namespace,
                    plugin_type,
                    is_service,
                    service_tags: Vec::new(), // Could extract from service YML file but would need cross-file analysis
                    is_hook: false,
                    hook_name: None,
                    annotations: annotations.to_vec(),
                    namespace: namespace.clone(),
                }),
            });
        }
        None
    }
    
    /// Extracts function definition with Drupal-specific hook detection
    fn extract_function_definition(&self, line: &str, line_idx: usize, _lines: &[&str], 
                                  doc_comment: &str, annotations: &[String], namespace: &Option<String>,
                                  is_drupal_module: bool) -> Option<CodeElement> {
        if let Some(name) = line.strip_prefix("function ").unwrap().split('(').next() {
            let name = name.trim();
            
            // Check if this is a hook implementation
            let is_hook = name.contains("_hook_") || 
                         annotations.iter().any(|a| a.contains("@Implements") || a.contains("@implements"));
            
            // Extract the hook name if this is a hook implementation
            let hook_name = if is_hook {
                if name.contains("_hook_") {
                    // Extract from function name pattern: module_hook_name
                    let parts: Vec<&str> = name.split('_').collect();
                    if parts.len() >= 3 && parts[1] == "hook" {
                        Some(format!("hook_{}", parts[2..].join("_")))
                    } else {
                        None
                    }
                } else {
                    // Extract from @Implements annotation
                    annotations.iter()
                        .find(|a| a.contains("@Implements") || a.contains("@implements"))
                        .and_then(|a| {
                            let hook_regex = Regex::new(r"@(?:Implements|implements)\s+hook_([a-zA-Z0-9_]+)").ok()?;
                            hook_regex.captures(a).map(|cap| format!("hook_{}", &cap[1]))
                        })
                }
            } else {
                None
            };
            
            // Determine function type
            let kind = if is_hook {
                "drupal_hook"
            } else if is_drupal_module {
                "drupal_function"
            } else {
                "function"
            };
            
            return Some(CodeElement {
                name: name.to_string(),
                kind: kind.to_string(),
                line: line_idx,
                description: self.extract_doc_comment_description(doc_comment),
                metadata: Some(ElementMetadata {
                    is_plugin: false,
                    plugin_type: None,
                    is_service: false,
                    service_tags: Vec::new(),
                    is_hook,
                    hook_name,
                    annotations: annotations.to_vec(),
                    namespace: namespace.clone(),
                }),
            });
        }
        None
    }
    
    /// Extracts a readable description from a doc comment
    fn extract_doc_comment_description(&self, doc_comment: &str) -> Option<String> {
        if doc_comment.is_empty() {
            return None;
        }
        
        // Extract the description part (before any @annotations)
        let mut description = String::new();
        let lines = doc_comment.lines();
        
        for line in lines {
            let trimmed = line.trim().trim_start_matches("/**").trim_start_matches("*").trim();
            if trimmed.starts_with('@') {
                break;  // Stop at the first annotation
            }
            
            if !trimmed.is_empty() {
                description.push_str(trimmed);
                description.push(' ');
            }
        }
        
        let description = description.trim().to_string();
        if description.is_empty() {
            None
        } else {
            Some(description)
        }
    }
    
    fn analyze_go_file(&self, content: &str) -> Result<FileStructure> {
        // Basic Go file analysis
        let mut structs = Vec::new();
        let mut functions = Vec::new();
        let mut interfaces = Vec::new();
        let mut package_name = String::new();
        
        let lines: Vec<&str> = content.lines().collect();
        
        for line_idx in 0..lines.len() {
            let line = lines[line_idx].trim();
            
            // Extract package name
            if line.starts_with("package ") && package_name.is_empty() {
                if let Some(name) = line.strip_prefix("package ") {
                    package_name = name.trim().to_string();
                }
            }
            // Find struct definitions
            else if line.starts_with("type ") && line.contains("struct") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "type" {
                    let struct_name = parts[1].to_string();
                    structs.push(CodeElement {
                        name: struct_name,
                        kind: "struct".to_string(),
                        line: line_idx + 1,
                        description: None,
                        metadata: None,
                    });
                }
            }
            // Find interface definitions
            else if line.starts_with("type ") && line.contains("interface") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "type" {
                    let interface_name = parts[1].to_string();
                    interfaces.push(CodeElement {
                        name: interface_name,
                        kind: "interface".to_string(),
                        line: line_idx + 1,
                        description: None,
                        metadata: None,
                    });
                }
            }
            // Find function definitions
            else if line.starts_with("func ") {
                let func_parts: Vec<&str> = line.split('(').collect();
                if func_parts.len() >= 1 {
                    let func_name = func_parts[0].trim_start_matches("func ").trim();
                    // Check if it's a method (has a receiver)
                    let is_method = !func_name.is_empty() && func_parts.len() > 1;
                    
                    if !func_name.is_empty() {
                        functions.push(CodeElement {
                            name: func_name.to_string(),
                            kind: if is_method { "method".to_string() } else { "function".to_string() },
                            line: line_idx + 1,
                            description: None,
                            metadata: Some(ElementMetadata {
                                is_plugin: false,
                                plugin_type: None,
                                is_service: false,
                                service_tags: Vec::new(),
                                is_hook: false,
                                hook_name: None,
                                annotations: Vec::new(),
                                namespace: Some(package_name.clone()),
                            }),
                        });
                    }
                }
            }
        }
        
        // Combine all elements
        let mut elements = Vec::new();
        elements.extend(structs);
        elements.extend(interfaces);
        elements.extend(functions);
        
        Ok(FileStructure {
            elements,
            is_drupal: false,
        })
    }
    
    fn analyze_generic_file(&self, _content: &str) -> Result<FileStructure> {
        // Very basic analysis for unknown file types
        Ok(FileStructure {
            elements: Vec::new(),
            is_drupal: false,
        })
    }
}

#[derive(Debug)]
pub struct FileStructure {
    pub elements: Vec<CodeElement>,
    pub is_drupal: bool,
}

#[derive(Debug)]
pub struct CodeElement {
    pub name: String,
    pub kind: String,
    pub line: usize,
    pub description: Option<String>,
    pub metadata: Option<ElementMetadata>,
}

#[derive(Debug)]
pub struct ElementMetadata {
    pub is_plugin: bool,
    pub plugin_type: Option<String>,
    pub is_service: bool,
    pub service_tags: Vec<String>,
    pub is_hook: bool,
    pub hook_name: Option<String>,
    pub annotations: Vec<String>,
    pub namespace: Option<String>,
}
