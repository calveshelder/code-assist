use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    pub fn analyze_project_structure(&self, project_path: &Path) -> Result<ProjectStructure> {
        let mut directories = Vec::new();
        let mut files_by_type = HashMap::new();
        
        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
                
            let path = entry.path();
            
            if path.is_dir() {
                if !self.should_ignore_dir(path) {
                    directories.push(path.strip_prefix(project_path)?.to_path_buf());
                }
            } else if path.is_file() {
                if !self.should_ignore_file(path) {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let entry = files_by_type
                            .entry(ext.to_string())
                            .or_insert_with(Vec::new);
                            
                        entry.push(path.strip_prefix(project_path)?.to_path_buf());
                    }
                }
            }
        }
        
        Ok(ProjectStructure {
            directories,
            files_by_type,
        })
    }
    
    fn should_ignore_dir(&self, path: &Path) -> bool {
        let ignore_dirs = [
            ".git", "node_modules", "target", "build", "dist", "venv",
            "__pycache__", ".idea", ".vscode",
        ];
        
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            ignore_dirs.contains(&dir_name) || dir_name.starts_with('.')
        } else {
            false
        }
    }
    
    fn should_ignore_file(&self, path: &Path) -> bool {
        let ignore_extensions = [
            "pyc", "exe", "dll", "so", "o", "obj", "class",
        ];
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            ignore_extensions.contains(&ext)
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct ProjectStructure {
    pub directories: Vec<PathBuf>,
    pub files_by_type: HashMap<String, Vec<PathBuf>>,
}
