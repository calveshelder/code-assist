use git2::{Repository, Diff, Error as Git2Error};
use anyhow::{Result, Context};
use std::path::Path;

pub struct GitDiff;

impl GitDiff {
    pub fn get_working_diff(repo_path: &Path) -> Result<String> {
        let repo = Repository::open(repo_path)
            .context("Failed to open git repository")?;
        
        let diff = repo.diff_index_to_workdir(None, None)
            .context("Failed to get diff between index and working directory")?;
        
        let mut diff_output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content()).unwrap_or("");
            diff_output.push_str(content);
            true
        })?;
        
        Ok(diff_output)
    }
    
    pub fn resolve_merge_conflict(
        repo_path: &Path,
        file_path: &Path,
        resolution: &str
    ) -> Result<()> {
        use std::fs;
        
        // Write the resolution to the file
        fs::write(file_path, resolution)
            .context("Failed to write resolution to file")?;
        
        // Mark as resolved with git add
        let relative_path = file_path.strip_prefix(repo_path)
            .unwrap_or(file_path)
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))?;
        
        let output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(&["add", relative_path])
            .output()
            .context("Failed to execute git add")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to mark file as resolved: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        Ok(())
    }
}
