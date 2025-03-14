use crate::fs::search::CodeSearch;
use anyhow::Result;
use std::path::Path;
use crate::memory::ProjectMemory;
use crate::analysis::structure::{ProjectAnalyzer, ProjectType, ProjectStructure, SpecificProjectInfo};

pub struct ContextManager {
    code_search: CodeSearch,
    pub project_memory: ProjectMemory,  // Made public
    project_analyzer: ProjectAnalyzer,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            code_search: CodeSearch::new(),
            project_memory: ProjectMemory::new(),
            project_analyzer: ProjectAnalyzer {},
        }
    }
    
    /// Add file count information for all supported languages
    fn add_file_count_info(&self, context: &mut String, project_structure: &ProjectStructure) {
        // Add counts for each language
        let language_extensions = [
            ("rs", "Rust"),
            ("py", "Python"),
            ("js", "JavaScript"),
            ("ts", "TypeScript"),
            ("jsx", "React JSX"),
            ("tsx", "React TSX"),
            ("go", "Go"),
            ("php", "PHP"),
            ("java", "Java"),
            ("cpp", "C++"),
            ("h", "C/C++ header")
        ];
        
        for (ext, lang) in &language_extensions {
            if let Some(files) = project_structure.files_by_type.get(*ext) {
                if !files.is_empty() {
                    context.push_str(&format!("{} files count: {}\n", lang, files.len()));
                }
            }
        }
    }
    
    /// Add Rust project information to context
    fn add_rust_project_info(&self, context: &mut String, project_structure: &ProjectStructure) -> Result<()> {
        if let SpecificProjectInfo::Rust(Some(rust_info)) = &project_structure.specific_info {
            context.push_str(&format!("Rust package: {}\n", rust_info.name));
            if !rust_info.version.is_empty() {
                context.push_str(&format!("Version: {}\n", rust_info.version));
            }
            
            context.push_str(&format!("Contains {} modules, {} structs\n", 
                              rust_info.module_count, 
                              rust_info.struct_count));
            
            if rust_info.has_lib {
                context.push_str("Has library target (lib.rs)\n");
            }
            
            if rust_info.has_bin {
                context.push_str("Has binary target (main.rs or bin/)\n");
            }
        }
        Ok(())
    }
    
    /// Add Python project information to context
    fn add_python_project_info(&self, context: &mut String, project_structure: &ProjectStructure) -> Result<()> {
        if let SpecificProjectInfo::Python(Some(python_info)) = &project_structure.specific_info {
            context.push_str(&format!("Python project: {}\n", python_info.name));
            context.push_str(&format!("Contains {} classes, {} functions\n", 
                              python_info.class_count, 
                              python_info.function_count));
            
            if python_info.has_django {
                context.push_str("Django framework detected\n");
            }
            
            if python_info.has_flask {
                context.push_str("Flask framework detected\n");
            }
            
            if python_info.has_fastapi {
                context.push_str("FastAPI framework detected\n");
            }
        }
        Ok(())
    }
    
    /// Add Angular project information to context
    fn add_angular_project_info(&self, context: &mut String, project_structure: &ProjectStructure) -> Result<()> {
        if let SpecificProjectInfo::Angular(Some(angular_info)) = &project_structure.specific_info {
            context.push_str(&format!("Angular project: {}\n", angular_info.name));
            context.push_str(&format!("Contains {} components, {} services\n", 
                              angular_info.component_count, 
                              angular_info.service_count));
            
            if angular_info.has_routing {
                context.push_str("Uses Angular routing\n");
            }
            
            if angular_info.has_ngrx {
                context.push_str("Uses NgRx state management\n");
            }
        }
        Ok(())
    }
    
    /// Add React project information to context
    fn add_react_project_info(&self, context: &mut String, project_structure: &ProjectStructure) -> Result<()> {
        if let SpecificProjectInfo::React(Some(react_info)) = &project_structure.specific_info {
            context.push_str(&format!("React project: {}\n", react_info.name));
            context.push_str(&format!("Contains approximately {} components\n", react_info.component_count));
            
            if react_info.has_redux {
                context.push_str("Uses Redux state management\n");
            }
            
            if react_info.is_nextjs {
                context.push_str("Next.js framework detected\n");
            }
            
            if react_info.has_typescript {
                context.push_str("Uses TypeScript\n");
            }
        }
        Ok(())
    }
    
    /// Add Drupal project information to context
    fn add_drupal_project_info(&self, context: &mut String, project_structure: &ProjectStructure, cwd: &Path) -> Result<()> {
        // Count PHP files
        if let Some(php_files) = project_structure.files_by_type.get("php") {
            context.push_str(&format!("PHP files count: {}\n", php_files.len()));
        }
        
        // Note if .info.yml files exist
        if let Some(yml_files) = project_structure.files_by_type.get("yml") {
            let info_files: Vec<_> = yml_files.iter()
                .filter(|p| p.to_string_lossy().ends_with(".info.yml"))
                .collect();
            
            if !info_files.is_empty() {
                context.push_str(&format!("Drupal module info files: {}\n", info_files.len()));
            }
        }
        
        // List all detected Drupal modules
        if !project_structure.modules.is_empty() {
            context.push_str(&format!("\nDetected Drupal modules ({}):\n", project_structure.modules.len()));
            for (module_name, module_path) in &project_structure.modules {
                let relative_path = module_path.strip_prefix(cwd).unwrap_or(module_path);
                context.push_str(&format!("- {}: {}\n", module_name, relative_path.display()));
            }
        }
        
        Ok(())
    }
    
    /// Add Drupal module information to context
    fn add_drupal_module_project_info(&self, context: &mut String, project_structure: &ProjectStructure, cwd: &Path) -> Result<()> {
        // Add detailed info about a Drupal module
        if let SpecificProjectInfo::Drupal(Some(module_info)) = &project_structure.specific_info {
            context.push_str(&format!("Drupal Module: {}\n", module_info.name));
            if !module_info.description.is_empty() {
                context.push_str(&format!("Description: {}\n", module_info.description));
            }
            
            // Add info about module files
            if let Some(module_file) = &module_info.module_file {
                context.push_str(&format!("Module file: {}\n", module_file.display()));
            }
            
            if let Some(info_file) = &module_info.info_file {
                context.push_str(&format!("Info file: {}\n", info_file.display()));
            }
            
            // Add hooks implemented
            if !module_info.hooks.is_empty() {
                context.push_str("Implements hooks:\n");
                for hook in &module_info.hooks {
                    context.push_str(&format!("- {}\n", hook));
                }
            }
            
            // Add plugin and service info
            if module_info.has_plugins {
                context.push_str("Contains plugins: Yes\n");
            }
            
            if module_info.has_services {
                context.push_str("Contains services: Yes\n");
            }
            
            // Add config schema info
            if !module_info.config_schemas.is_empty() {
                context.push_str("Config schemas:\n");
                for schema in &module_info.config_schemas {
                    context.push_str(&format!("- {}\n", schema.display()));
                }
            }
            
            // Analyze module structure in more depth
            self.add_drupal_module_analysis(context, cwd, &module_info.name)?;
        }
        
        // List other detected Drupal modules if we're in a Drupal site with modules
        if !project_structure.modules.is_empty() && project_structure.modules.len() > 1 {
            let current_module_name = if let SpecificProjectInfo::Drupal(Some(info)) = &project_structure.specific_info {
                Some(&info.name)
            } else {
                None
            };
            
            let other_modules = project_structure.modules.iter()
                .filter(|(name, _)| Some(name) != current_module_name)
                .collect::<Vec<_>>();
            
            if !other_modules.is_empty() {
                context.push_str(&format!("\nOther Drupal modules in this project ({}):\n", other_modules.len()));
                
                for (module_name, module_path) in other_modules {
                    let relative_path = module_path.strip_prefix(cwd).unwrap_or(module_path);
                    context.push_str(&format!("- {}: {}\n", module_name, relative_path.display()));
                }
            }
        }
        
        Ok(())
    }
    
    /// Add detailed Drupal module analysis to context
    fn add_drupal_module_analysis(&self, context: &mut String, project_path: &Path, module_name: &str) -> Result<()> {
        // This function does deeper analysis of a Drupal module structure
        
        // Try multiple possible paths for the module
        let possible_paths = [
            // Standard Drupal 8+ module paths
            project_path.join(format!("web/modules/custom/{}", module_name)),
            project_path.join(format!("web/modules/contrib/{}", module_name)),
            project_path.join(format!("modules/custom/{}", module_name)),
            project_path.join(format!("modules/contrib/{}", module_name)),
            // The module might be in the current directory
            project_path.to_path_buf(),
        ];
        
        // Find the real module path
        let mut module_path = None;
        for path in &possible_paths {
            // Check if this is a valid module path
            if path.join(format!("{}.info.yml", module_name)).exists() || 
               path.join(format!("{}.module", module_name)).exists() ||
               path.join("src").exists() {
                module_path = Some(path);
                break;
            }
        }
        
        // If we can't find the module, just return
        let module_path = match module_path {
            Some(path) => path,
            None => return Ok(()),
        };
        
        // Look for key Drupal module directories like src/Plugin, src/Form, etc.
        let src_path = module_path.join("src");
        if src_path.exists() {
            context.push_str("\nModule Structure:\n");
            
            // Check for common Drupal plugin/component directories
            let key_directories = [
                ("Plugin", "Contains plugins"),
                ("Form", "Contains form definitions"),
                ("Entity", "Contains entity definitions"),
                ("Controller", "Contains route controllers"),
                ("EventSubscriber", "Contains event subscribers"),
                ("Access", "Contains access control"),
                ("Element", "Contains render elements"),
            ];
            
            for (dir, description) in key_directories.iter() {
                let dir_path = src_path.join(dir);
                if dir_path.exists() {
                    context.push_str(&format!("- {}: {}\n", dir, description));
                    
                    // For Plugin directory, go deeper to categorize plugin types
                    if *dir == "Plugin" {
                        if let Ok(entries) = std::fs::read_dir(dir_path) {
                            for entry in entries.filter_map(|e| e.ok()) {
                                if entry.path().is_dir() {
                                    if let Some(name) = entry.file_name().to_str() {
                                        let plugin_description = match name {
                                            "Block" => "Block plugins (content blocks)",
                                            "Field" => "Field types/widgets/formatters",
                                            "Action" => "Action plugins",
                                            "Condition" => "Condition plugins",
                                            "Filter" => "Text format filters",
                                            "Queue" => "Queue workers",
                                            "Views" => "Views plugins",
                                            _ => "Custom plugin type",
                                        };
                                        context.push_str(&format!("  - {}: {}\n", name, plugin_description));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Look for module-specific template files
        let templates_path = module_path.join("templates");
        if templates_path.exists() {
            context.push_str("\nModule Templates:\n");
            if let Ok(entries) = std::fs::read_dir(templates_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".html.twig") {
                            context.push_str(&format!("- {}\n", name));
                        }
                    }
                }
            }
        }
        
        // Look for any JavaScript/CSS files
        let js_path = module_path.join("js");
        if js_path.exists() {
            context.push_str("\nJavaScript files present\n");
            
            // Count JS files to help correct JS bias
            let mut js_count = 0;
            if let Ok(entries) = std::fs::read_dir(js_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".js") {
                            js_count += 1;
                        }
                    }
                }
            }
            
            if js_count > 0 {
                context.push_str(&format!("JavaScript file count: {}\n", js_count));
            }
        }
        
        // Check for Angular apps within the module
        let app_path = module_path.join("app");
        if app_path.exists() && app_path.join("angular.json").exists() {
            context.push_str("\nContains Angular application\n");
        }
        
        let css_path = module_path.join("css");
        if css_path.exists() {
            context.push_str("CSS files present\n");
        }
        
        // Report PHP file count to balance JS bias
        let php_count = self.count_php_files_in_module(&module_path)?;
        if php_count > 0 {
            context.push_str(&format!("PHP file count: {}\n", php_count));
        }
        
        Ok(())
    }
    
    /// Count PHP files in a module
    fn count_php_files_in_module(&self, module_path: &Path) -> Result<usize> {
        let mut count = 0;
        
        // Use walkdir to count all PHP files
        for entry in walkdir::WalkDir::new(module_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            
            if entry.path().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "php" {
                        count += 1;
                    }
                }
            }
        }
        
        Ok(count)
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
        context.push_str(&format!("Working directory: {}\n", cwd.display()));
        
        // Analyze project structure to detect project type
        if let Ok(project_structure) = self.project_analyzer.analyze_project_structure(&cwd) {
            if let Some(project_type) = &project_structure.project_type {
                let type_str = match project_type {
                    ProjectType::Drupal => "Drupal site",
                    ProjectType::DrupalModule => "Drupal module",
                    ProjectType::Rust => "Rust project",
                    ProjectType::Python => "Python project",
                    ProjectType::JavaScript => "JavaScript project",
                    ProjectType::TypeScript => "TypeScript project",
                    ProjectType::Go => "Go project",
                    ProjectType::PHP => "PHP project",
                    ProjectType::Angular => "Angular application",
                    ProjectType::React => "React application",
                    ProjectType::Generic => "Generic project",
                };
                context.push_str(&format!("\nProject type: {}\n", type_str));
                
                // Add language-specific file counts
                self.add_file_count_info(&mut context, &project_structure);
                
                // Add more specific information based on project type
                match project_type {
                    ProjectType::Rust => {
                        self.add_rust_project_info(&mut context, &project_structure)?;
                    },
                    ProjectType::Python => {
                        self.add_python_project_info(&mut context, &project_structure)?;
                    },
                    ProjectType::Angular => {
                        self.add_angular_project_info(&mut context, &project_structure)?;
                    },
                    ProjectType::React => {
                        self.add_react_project_info(&mut context, &project_structure)?;
                    },
                    ProjectType::Drupal => {
                        self.add_drupal_project_info(&mut context, &project_structure, &cwd)?;
                    },
                    ProjectType::DrupalModule => {
                        self.add_drupal_module_project_info(&mut context, &project_structure, &cwd)?;
                    },
                    _ => {
                        // For other project types, add generic info about the directory structure
                        let directories_count = project_structure.directories.len();
                        if directories_count > 0 {
                            context.push_str(&format!("Project contains {} directories\n", directories_count));
                            // List top-level directories
                            let top_level = project_structure.directories.iter()
                                .filter(|p| p.components().count() == 1)
                                .take(5)
                                .collect::<Vec<_>>();
                            
                            if !top_level.is_empty() {
                                context.push_str("Top-level directories:\n");
                                for dir in top_level {
                                    context.push_str(&format!("- {}\n", dir.display()));
                                }
                            }
                        }
                    }
                }
            }
            
            context.push_str("\n");
        }
        
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
