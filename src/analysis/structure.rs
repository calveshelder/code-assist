use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;
use regex::Regex;
use glob::glob;

pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    /// Analyzes the structure of a project to determine its type and organize files
    pub fn analyze_project_structure(&self, project_path: &Path) -> Result<ProjectStructure> {
        let mut directories = Vec::new();
        let mut files_by_type = HashMap::new();
        
        // Detect project structure by scanning files and directories
        let project_features = self.scan_project_features(project_path, &mut directories, &mut files_by_type)?;
        
        // Determine project type based on detected features
        let (project_type, modules) = self.determine_project_type(project_path, &project_features, &files_by_type)?;
        
        // Gather specific details for the detected project type
        let specific_info = match project_type {
            ProjectType::DrupalModule => SpecificProjectInfo::Drupal(
                self.gather_drupal_module_info(project_path, &files_by_type)?
            ),
            ProjectType::Rust => SpecificProjectInfo::Rust(
                self.gather_rust_project_info(project_path, &files_by_type)?
            ),
            ProjectType::Angular => SpecificProjectInfo::Angular(
                self.gather_angular_project_info(project_path, &files_by_type)?
            ),
            ProjectType::React => SpecificProjectInfo::React(
                self.gather_react_project_info(project_path, &files_by_type)?
            ),
            ProjectType::Python => SpecificProjectInfo::Python(
                self.gather_python_project_info(project_path, &files_by_type)?
            ),
            _ => SpecificProjectInfo::None,
        };
        
        Ok(ProjectStructure {
            directories,
            files_by_type,
            project_type: Some(project_type),
            specific_info,
            modules,
        })
    }
    
    /// Scans project directories and files to detect project features
    fn scan_project_features(&self, project_path: &Path, 
                            directories: &mut Vec<PathBuf>,
                            files_by_type: &mut HashMap<String, Vec<PathBuf>>) -> Result<ProjectFeatures> {
        let mut features = ProjectFeatures::default();
        
        for entry in WalkDir::new(project_path)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok()) {
                
            let path = entry.path();
            
            if path.is_dir() {
                if !self.should_ignore_dir(path) {
                    directories.push(path.strip_prefix(project_path)?.to_path_buf());
                    
                    // Check for key directories
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        match dir_name {
                            "core" => features.has_drupal_core = true,
                            "src" => features.has_src_dir = true,
                            "node_modules" => features.has_node_modules = true,
                            ".git" => features.has_git = true,
                            "target" => features.has_rust_target = true,
                            "Plugin" => {
                                if path.starts_with(project_path.join("src")) {
                                    features.has_drupal_plugin_dir = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } else if path.is_file() {
                if !self.should_ignore_file(path) {
                    // Check for specific files by name/extension
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        match file_name {
                            "Cargo.toml" => features.has_cargo_toml = true,
                            "package.json" => features.has_package_json = true,
                            "angular.json" => features.has_angular_json = true,
                            "composer.json" => features.has_composer_json = true,
                            "pyproject.toml" => features.has_pyproject_toml = true,
                            "requirements.txt" => features.has_requirements_txt = true,
                            "setup.py" => features.has_setup_py = true,
                            "go.mod" => features.has_go_mod = true,
                            _ => {
                                if file_name.ends_with(".info.yml") {
                                    features.has_info_yml = true;
                                    
                                    // Check if file contains Drupal module info
                                    if let Ok(content) = std::fs::read_to_string(path) {
                                        if content.contains("type: module") {
                                            features.has_drupal_module_file = true;
                                        }
                                    }
                                } else if file_name.ends_with(".module") {
                                    features.has_drupal_module_extension = true;
                                }
                            }
                        }
                    }
                    
                    // Check for language-specific indicators
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        // Check for key language file extensions
                        match ext {
                            "php" => {
                                features.has_php_files = true;
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    if content.contains("Drupal\\") || 
                                       content.contains("function") && content.contains("_hook_") ||
                                       content.contains("implements") && content.contains("Hook") {
                                        features.has_drupal_php_code = true;
                                    }
                                }
                            },
                            "rs" => features.has_rust_files = true,
                            "py" => features.has_python_files = true,
                            "js" => features.has_js_files = true,
                            "ts" => features.has_ts_files = true,
                            "jsx" => features.has_jsx_files = true,
                            "tsx" => features.has_tsx_files = true,
                            "go" => features.has_go_files = true,
                            _ => {}
                        }
                        
                        // Add file to files_by_type
                        let entry = files_by_type
                            .entry(ext.to_string())
                            .or_insert_with(Vec::new);
                            
                        entry.push(path.strip_prefix(project_path)?.to_path_buf());
                    }
                }
            }
        }
        
        // Additional directory-based checks
        features.has_drupal_modules_dir = project_path.join("web/modules").exists() || 
                                          project_path.join("modules").exists();
                                       
        Ok(features)
    }
    
    /// Determines the project type based on detected features
    fn determine_project_type(&self, project_path: &Path, 
                             features: &ProjectFeatures, 
                             files_by_type: &HashMap<String, Vec<PathBuf>>) -> Result<(ProjectType, Vec<(String, PathBuf)>)> {
        // Initialize an empty list for modules
        let mut drupal_modules = Vec::new();
        
        // Check for Drupal projects first
        let is_drupal_site = features.has_drupal_core || features.has_drupal_modules_dir;
        
        if is_drupal_site || (features.has_info_yml && (features.has_drupal_module_file || features.has_drupal_php_code)) {
            // Find all modules in the project if it's a Drupal project
            drupal_modules = self.find_all_drupal_modules(project_path)?;
            
            if !drupal_modules.is_empty() {
                // Determine if the current directory is itself a module
                let is_module = self.is_drupal_module(project_path)?;
                
                if is_module {
                    return Ok((ProjectType::DrupalModule, drupal_modules));
                } else {
                    return Ok((ProjectType::Drupal, drupal_modules));
                }
            }
        }
        
        // Check for other project types
        if features.has_cargo_toml {
            return Ok((ProjectType::Rust, Vec::new()));
        } else if features.has_angular_json && features.has_package_json {
            return Ok((ProjectType::Angular, Vec::new()));
        } else if features.has_package_json && (features.has_jsx_files || features.has_tsx_files || 
                                              (files_by_type.get("js").map_or(false, |files| 
                                                files.iter().any(|p| p.to_string_lossy().contains("react"))))) {
            return Ok((ProjectType::React, Vec::new()));
        } else if features.has_pyproject_toml || features.has_requirements_txt || features.has_setup_py {
            return Ok((ProjectType::Python, Vec::new()));
        } else if features.has_go_mod || features.has_go_files {
            return Ok((ProjectType::Go, Vec::new()));
        } else if features.has_js_files || features.has_ts_files {
            return Ok((ProjectType::JavaScript, Vec::new()));
        } else if features.has_php_files {
            return Ok((ProjectType::PHP, Vec::new()));
        }
        
        // Default to Generic if no specific type is detected
        Ok((ProjectType::Generic, Vec::new()))
    }
    
    fn should_ignore_dir(&self, path: &Path) -> bool {
        let ignore_dirs = [
            ".git", "node_modules", "target", "build", "dist", "venv",
            "__pycache__", ".idea", ".vscode", "vendor", ".next", "out",
        ];
        
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            ignore_dirs.contains(&dir_name) || dir_name.starts_with('.')
        } else {
            false
        }
    }
    
    fn should_ignore_file(&self, path: &Path) -> bool {
        let ignore_extensions = [
            "pyc", "exe", "dll", "so", "o", "obj", "class", "jpg", "png", 
            "gif", "pdf", "bin", "lock", "woff", "woff2", "ttf",
        ];
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            ignore_extensions.contains(&ext)
        } else {
            false
        }
    }
    
    /// Determines if a project is a Drupal module by checking for module structure
    fn is_drupal_module(&self, project_path: &Path) -> Result<bool> {
        // Check for patterns specific to Drupal modules
        let path = project_path.to_str().unwrap_or("");
        
        // Check if path contains "modules/custom" which is the standard Drupal module location
        if path.contains("modules/custom") || path.contains("modules/contrib") {
            return Ok(true);
        }
        
        // Look for module.info.yml file at root
        let info_yml_path = project_path.join("*.info.yml");
        let info_yml_glob = glob(info_yml_path.to_str().unwrap_or(""))?;
        let has_info_yml_at_root = info_yml_glob.count() > 0;
        
        // Look for *.module file at root
        let module_file_path = project_path.join("*.module");
        let module_file_glob = glob(module_file_path.to_str().unwrap_or(""))?;
        let has_module_file_at_root = module_file_glob.count() > 0;
        
        // Look for src/Plugin directory - common in Drupal modules
        let has_plugin_dir = project_path.join("src/Plugin").exists();
        
        // Look for a composer.json that depends on drupal/core
        let composer_path = project_path.join("composer.json");
        let has_drupal_dependency = if composer_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&composer_path) {
                content.contains("drupal/core")
            } else {
                false
            }
        } else {
            false
        };
        
        // If we're at the Drupal root, also check for modules in web/modules/custom
        let web_modules_custom = project_path.join("web/modules/custom");
        if web_modules_custom.exists() {
            // We're in a Drupal root, this directory contains modules
            if project_path.join("core").exists() && project_path.join("composer.json").exists() {
                return Ok(false); // This is the Drupal root, not a module
            }
        }
        
        // If multiple Drupal module indicators are present, it's likely a module
        Ok(has_info_yml_at_root && (has_module_file_at_root || has_plugin_dir || has_drupal_dependency))
    }
    
    /// Finds all Drupal modules in a project
    pub fn find_all_drupal_modules(&self, project_path: &Path) -> Result<Vec<(String, PathBuf)>> {
        let mut modules = Vec::new();
        
        // First check if this is a Drupal site with modules
        let web_modules_custom = project_path.join("web/modules/custom");
        let modules_custom = project_path.join("modules/custom");
        
        // Paths to check for modules
        let module_dirs = [
            web_modules_custom,
            modules_custom,
            project_path.join("web/modules/contrib"),
            project_path.join("modules/contrib"),
            // Also check the current directory as it might be a module
            project_path.to_path_buf(),
        ];
        
        for dir in module_dirs.iter() {
            if dir.exists() && dir.is_dir() {
                // If it's a nested modules directory, check each subdirectory
                if dir != &project_path.to_path_buf() {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let path = entry.path();
                            if path.is_dir() && self.is_drupal_module(&path)? {
                                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                    modules.push((name.to_string(), path));
                                }
                            }
                        }
                    }
                } else {
                    // Check if the current directory itself is a module
                    if self.is_drupal_module(dir)? {
                        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                            modules.push((name.to_string(), dir.clone()));
                        }
                    }
                }
            }
        }
        
        Ok(modules)
    }
    
    /// Gathers detailed information about a Drupal module
    fn gather_drupal_module_info(&self, project_path: &Path, files_by_type: &HashMap<String, Vec<PathBuf>>) -> Result<Option<DrupalModuleInfo>> {
        // Find the .info.yml file for the module
        let info_yml_files: Vec<PathBuf> = if let Some(yml_files) = files_by_type.get("yml") {
            yml_files.iter()
                .filter(|p| p.to_string_lossy().ends_with(".info.yml"))
                .map(|p| project_path.join(p))
                .collect()
        } else {
            vec![]
        };
        
        if info_yml_files.is_empty() {
            return Ok(None);
        }
        
        // Find the main info.yml file (typically at the root of the module)
        let info_file = info_yml_files.iter()
            .find(|p| p.parent() == Some(project_path))
            .or_else(|| info_yml_files.first())
            .cloned();
        
        // Extract module name and description from info.yml
        let mut module_name = String::new();
        let mut module_description = String::new();
        
        if let Some(info_path) = &info_file {
            if let Ok(content) = std::fs::read_to_string(info_path) {
                for line in content.lines() {
                    if line.starts_with("name:") {
                        module_name = line.trim_start_matches("name:").trim().trim_matches('"').trim_matches('\'').to_string();
                    } else if line.starts_with("description:") {
                        module_description = line.trim_start_matches("description:").trim().trim_matches('"').trim_matches('\'').to_string();
                    }
                }
            }
        }
        
        // Find the module file (.module)
        let module_file_name = if let Some(info_path) = &info_file {
            if let Some(stem) = info_path.file_stem() {
                if let Some(name) = stem.to_str() {
                    if name.ends_with(".info") {
                        Some(name.trim_end_matches(".info"))
                    } else {
                        Some(name)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        let module_file = if let Some(name) = module_file_name {
            let module_path = project_path.join(format!("{}.module", name));
            if module_path.exists() {
                Some(module_path)
            } else {
                None
            }
        } else {
            None
        };
        
        // Find config schema files
        let config_schemas: Vec<PathBuf> = if let Some(yml_files) = files_by_type.get("yml") {
            yml_files.iter()
                .filter(|p| p.to_string_lossy().contains("/config/schema/"))
                .map(|p| project_path.join(p))
                .collect()
        } else {
            vec![]
        };
        
        // Check for plugins
        let has_plugins = project_path.join("src/Plugin").exists() || 
                         module_file.as_ref().map_or(false, |path| {
                             if let Ok(content) = std::fs::read_to_string(path) {
                                 content.contains("Plugin") || content.contains("plugin")
                             } else {
                                 false
                             }
                         });
        
        // Check for services
        let has_services = {
            let services_path = if let Some(name) = module_file_name {
                project_path.join(format!("{}.services.yml", name))
            } else {
                PathBuf::new()
            };
            
            services_path.exists() || files_by_type.get("yml").map_or(false, |yml_files| {
                yml_files.iter().any(|p| p.to_string_lossy().ends_with(".services.yml"))
            })
        };
        
        // Find implemented hooks
        let mut hooks = Vec::new();
        
        // Check for hooks in .module file
        if let Some(module_path) = &module_file {
            if let Ok(content) = std::fs::read_to_string(module_path) {
                // Find hook implementations using regex
                let hook_regex = Regex::new(r"function\s+([a-zA-Z0-9_]+)_hook_([a-zA-Z0-9_]+)")?;
                
                for cap in hook_regex.captures_iter(&content) {
                    if cap.len() >= 3 {
                        let hook_name = format!("hook_{}", &cap[2]);
                        hooks.push(hook_name);
                    }
                }
            }
        }
        
        // Search for hooks in all PHP files
        if let Some(php_files) = files_by_type.get("php") {
            for file_path in php_files {
                let full_path = project_path.join(file_path);
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    // Regex to find hook implementations
                    let hook_regex = Regex::new(r"function\s+([a-zA-Z0-9_]+)_hook_([a-zA-Z0-9_]+)")?;
                    
                    for cap in hook_regex.captures_iter(&content) {
                        if cap.len() >= 3 {
                            let hook_name = format!("hook_{}", &cap[2]);
                            if !hooks.contains(&hook_name) {
                                hooks.push(hook_name);
                            }
                        }
                    }
                    
                    // Look for implements hook annotations
                    let annotation_regex = Regex::new(r"@(Implements|implements)\s+hook_([a-zA-Z0-9_]+)")?;
                    
                    for cap in annotation_regex.captures_iter(&content) {
                        if cap.len() >= 3 {
                            let hook_name = format!("hook_{}", &cap[2]);
                            if !hooks.contains(&hook_name) {
                                hooks.push(hook_name);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(Some(DrupalModuleInfo {
            name: if module_name.is_empty() { 
                module_file_name.unwrap_or("unknown").to_string() 
            } else { 
                module_name 
            },
            description: module_description,
            module_file: module_file.map(|p| p.strip_prefix(project_path).unwrap_or(&p).to_path_buf()),
            info_file: info_file.map(|p| p.strip_prefix(project_path).unwrap_or(&p).to_path_buf()),
            config_schemas: config_schemas.iter()
                .map(|p| p.strip_prefix(project_path).unwrap_or(p).to_path_buf())
                .collect(),
            has_plugins,
            has_services,
            hooks,
        }))
    }
    
    /// Gathers information about a Rust project
    fn gather_rust_project_info(&self, project_path: &Path, files_by_type: &HashMap<String, Vec<PathBuf>>) -> Result<Option<RustProjectInfo>> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(None);
        }
        
        let mut package_name = String::new();
        let mut version = String::new();
        
        if let Ok(content) = std::fs::read_to_string(&cargo_toml_path) {
            // Extract basic information from Cargo.toml
            for line in content.lines() {
                if line.trim().starts_with("name") {
                    package_name = line.split('=').nth(1)
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .unwrap_or_default();
                } else if line.trim().starts_with("version") {
                    version = line.split('=').nth(1)
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .unwrap_or_default();
                }
            }
        }
        
        // Count modules and structs
        let mut module_count = 0;
        let mut struct_count = 0;
        
        if let Some(rs_files) = files_by_type.get("rs") {
            for file_path in rs_files {
                if let Ok(content) = std::fs::read_to_string(project_path.join(file_path)) {
                    // Count mod declarations
                    module_count += content.matches("mod ").count();
                    // Count struct declarations
                    struct_count += content.matches("struct ").count();
                }
            }
        }
        
        Ok(Some(RustProjectInfo {
            name: package_name,
            version,
            module_count,
            struct_count,
            has_lib: project_path.join("src/lib.rs").exists(),
            has_bin: project_path.join("src/main.rs").exists() || project_path.join("src/bin").exists(),
        }))
    }
    
    /// Gathers information about an Angular project
    fn gather_angular_project_info(&self, project_path: &Path, files_by_type: &HashMap<String, Vec<PathBuf>>) -> Result<Option<AngularProjectInfo>> {
        let angular_json_path = project_path.join("angular.json");
        if !angular_json_path.exists() {
            return Ok(None);
        }
        
        let mut project_name = String::new();
        
        if let Ok(content) = std::fs::read_to_string(&angular_json_path) {
            // Try to extract project name from angular.json
            if let Some(start) = content.find("\"projects\"") {
                if let Some(project_start) = content[start..].find('{') {
                    if let Some(name_start) = content[start + project_start + 1..].find('"') {
                        let name_end = content[start + project_start + 1 + name_start + 1..].find('"').unwrap_or(0);
                        if name_end > 0 {
                            project_name = content[start + project_start + 1 + name_start + 1..start + project_start + 1 + name_start + 1 + name_end].to_string();
                        }
                    }
                }
            }
        }
        
        // Count components and services
        let mut component_count = 0;
        let mut service_count = 0;
        
        if let Some(ts_files) = files_by_type.get("ts") {
            for file_path in ts_files {
                let path_str = file_path.to_string_lossy().to_string();
                if path_str.ends_with(".component.ts") {
                    component_count += 1;
                } else if path_str.ends_with(".service.ts") {
                    service_count += 1;
                }
                
                if let Ok(content) = std::fs::read_to_string(project_path.join(file_path)) {
                    if content.contains("@Component") {
                        component_count += 1;
                    } else if content.contains("@Injectable") {
                        service_count += 1;
                    }
                }
            }
        }
        
        Ok(Some(AngularProjectInfo {
            name: project_name,
            component_count,
            service_count,
            has_routing: files_by_type.get("ts").map_or(false, |files| 
                files.iter().any(|p| p.to_string_lossy().contains("routing") || 
                                    p.to_string_lossy().contains("routes"))),
            has_ngrx: files_by_type.get("ts").map_or(false, |files| 
                files.iter().any(|p| p.to_string_lossy().contains("reducer") || 
                                    p.to_string_lossy().contains("action") || 
                                    p.to_string_lossy().contains("effect"))),
        }))
    }
    
    /// Gathers information about a React project
    fn gather_react_project_info(&self, project_path: &Path, files_by_type: &HashMap<String, Vec<PathBuf>>) -> Result<Option<ReactProjectInfo>> {
        let package_json_path = project_path.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }
        
        let mut project_name = String::new();
        let mut has_redux = false;
        
        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
            // Extract project name from package.json
            if let Some(name_start) = content.find("\"name\"") {
                if let Some(colon) = content[name_start..].find(':') {
                    let start_idx = name_start + colon + 1;
                    if let Some(quote_start) = content[start_idx..].find('"') {
                        let value_start = start_idx + quote_start + 1;
                        if let Some(quote_end) = content[value_start..].find('"') {
                            project_name = content[value_start..value_start + quote_end].to_string();
                        }
                    }
                }
            }
            
            // Check for Redux dependencies
            has_redux = content.contains("\"redux\"") || 
                        content.contains("\"@reduxjs/toolkit\"") || 
                        content.contains("\"react-redux\"");
        }
        
        // Count components
        let mut component_count = 0;
        
        // Count .jsx and .tsx files as components
        component_count += files_by_type.get("jsx").map_or(0, |files| files.len());
        component_count += files_by_type.get("tsx").map_or(0, |files| files.len());
        
        // Check .js and .ts files for React components
        for ext in &["js", "ts"] {
            if let Some(files) = files_by_type.get(*ext) {
                for file_path in files {
                    if let Ok(content) = std::fs::read_to_string(project_path.join(file_path)) {
                        if content.contains("React") && (content.contains("class ") && content.contains("extends") || 
                                                         content.contains("function ") && content.contains("return")) {
                            component_count += 1;
                        }
                    }
                }
            }
        }
        
        // Determine if Next.js project
        let is_nextjs = project_path.join("pages").exists() || 
                        project_path.join("src/pages").exists() || 
                        project_path.join(".next").exists();
        
        Ok(Some(ReactProjectInfo {
            name: project_name,
            component_count,
            has_redux,
            is_nextjs,
            has_typescript: files_by_type.get("tsx").is_some() || files_by_type.get("ts").is_some(),
        }))
    }
    
    /// Gathers information about a Python project
    fn gather_python_project_info(&self, project_path: &Path, files_by_type: &HashMap<String, Vec<PathBuf>>) -> Result<Option<PythonProjectInfo>> {
        // Check for either pyproject.toml, setup.py, or requirements.txt
        let mut project_name = String::new();
        let mut has_django = false;
        let mut has_flask = false;
        let mut has_fastapi = false;
        
        // Try to determine project name from common Python project files
        if project_path.join("pyproject.toml").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("pyproject.toml")) {
                if let Some(name_pos) = content.find("name = ") {
                    if let Some(quote_start) = content[name_pos + 7..].find('"') {
                        if let Some(quote_end) = content[name_pos + 7 + quote_start + 1..].find('"') {
                            project_name = content[name_pos + 7 + quote_start + 1..name_pos + 7 + quote_start + 1 + quote_end].to_string();
                        }
                    }
                }
            }
        } else if project_path.join("setup.py").exists() {
            if let Ok(content) = std::fs::read_to_string(project_path.join("setup.py")) {
                if let Some(name_pos) = content.find("name=") {
                    if let Some(quote_start) = content[name_pos + 5..].find('"') {
                        if let Some(quote_end) = content[name_pos + 5 + quote_start + 1..].find('"') {
                            project_name = content[name_pos + 5 + quote_start + 1..name_pos + 5 + quote_start + 1 + quote_end].to_string();
                        }
                    }
                }
            }
        }
        
        // If project name still not found, use directory name
        if project_name.is_empty() {
            if let Some(dir_name) = project_path.file_name().and_then(|n| n.to_str()) {
                project_name = dir_name.to_string();
            }
        }
        
        // Check for popular Python frameworks
        if let Some(py_files) = files_by_type.get("py") {
            for file_path in py_files {
                if let Ok(content) = std::fs::read_to_string(project_path.join(file_path)) {
                    if content.contains("django") {
                        has_django = true;
                    }
                    if content.contains("flask") {
                        has_flask = true;
                    }
                    if content.contains("fastapi") {
                        has_fastapi = true;
                    }
                }
            }
        }
        
        // Check for Django-specific directories
        has_django = has_django || project_path.join("manage.py").exists();
        
        // Count class and function definitions
        let mut class_count = 0;
        let mut function_count = 0;
        
        if let Some(py_files) = files_by_type.get("py") {
            for file_path in py_files {
                if let Ok(content) = std::fs::read_to_string(project_path.join(file_path)) {
                    // Count class definitions
                    class_count += content.matches("class ").count();
                    // Count function definitions
                    function_count += content.matches("def ").count();
                }
            }
        }
        
        Ok(Some(PythonProjectInfo {
            name: project_name,
            class_count,
            function_count,
            has_django,
            has_flask,
            has_fastapi,
        }))
    }
}

#[derive(Debug, PartialEq)]
pub enum ProjectType {
    Drupal,
    DrupalModule,
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    PHP,
    Angular,
    React,
    Generic,
}

#[derive(Debug, Default)]
pub struct ProjectFeatures {
    // Drupal-specific
    pub has_drupal_core: bool,
    pub has_drupal_modules_dir: bool,
    pub has_info_yml: bool,
    pub has_drupal_module_file: bool,  // .info.yml with "type: module"
    pub has_drupal_module_extension: bool,  // .module file
    pub has_drupal_php_code: bool,
    pub has_drupal_plugin_dir: bool,
    
    // General directories
    pub has_src_dir: bool,
    pub has_node_modules: bool,
    pub has_git: bool,
    pub has_rust_target: bool,
    
    // Language-specific files
    pub has_php_files: bool,
    pub has_rust_files: bool,
    pub has_python_files: bool,
    pub has_js_files: bool,
    pub has_ts_files: bool,
    pub has_jsx_files: bool,
    pub has_tsx_files: bool,
    pub has_go_files: bool,
    
    // Project definition files
    pub has_cargo_toml: bool,
    pub has_package_json: bool,
    pub has_angular_json: bool,
    pub has_composer_json: bool,
    pub has_pyproject_toml: bool,
    pub has_requirements_txt: bool,
    pub has_setup_py: bool,
    pub has_go_mod: bool,
}

// Specific project information types
#[derive(Debug)]
pub enum SpecificProjectInfo {
    Drupal(Option<DrupalModuleInfo>),
    Rust(Option<RustProjectInfo>),
    Angular(Option<AngularProjectInfo>),
    React(Option<ReactProjectInfo>),
    Python(Option<PythonProjectInfo>),
    None,
}

#[derive(Debug)]
pub struct ProjectStructure {
    pub directories: Vec<PathBuf>,
    pub files_by_type: HashMap<String, Vec<PathBuf>>,
    pub project_type: Option<ProjectType>,
    pub specific_info: SpecificProjectInfo,
    pub modules: Vec<(String, PathBuf)>, // List of (module_name, module_path)
}

#[derive(Debug)]
pub struct DrupalModuleInfo {
    pub name: String,
    pub description: String, 
    pub module_file: Option<PathBuf>,
    pub info_file: Option<PathBuf>,
    pub config_schemas: Vec<PathBuf>,
    pub has_plugins: bool,
    pub has_services: bool,
    pub hooks: Vec<String>,
}

#[derive(Debug)]
pub struct RustProjectInfo {
    pub name: String,
    pub version: String,
    pub module_count: usize,
    pub struct_count: usize,
    pub has_lib: bool,
    pub has_bin: bool,
}

#[derive(Debug)]
pub struct AngularProjectInfo {
    pub name: String,
    pub component_count: usize,
    pub service_count: usize,
    pub has_routing: bool,
    pub has_ngrx: bool,
}

#[derive(Debug)]
pub struct ReactProjectInfo {
    pub name: String,
    pub component_count: usize,
    pub has_redux: bool,
    pub is_nextjs: bool,
    pub has_typescript: bool,
}

#[derive(Debug)]
pub struct PythonProjectInfo {
    pub name: String,
    pub class_count: usize,
    pub function_count: usize,
    pub has_django: bool,
    pub has_flask: bool,
    pub has_fastapi: bool,
}

// End of file
