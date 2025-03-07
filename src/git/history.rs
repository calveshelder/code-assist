use git2::{Repository, Commit, Error as Git2Error};
use anyhow::{Result, Context, anyhow};
use std::path::Path;

pub struct GitHistory;

impl GitHistory {
    pub fn get_commit_history(repo_path: &Path, max_count: usize) -> Result<Vec<CommitInfo>> {
        let repo = Repository::open(repo_path)
            .context("Failed to open git repository")?;
        
        let mut revwalk = repo.revwalk()
            .context("Failed to create revision walker")?;
        
        revwalk.push_head()
            .context("Failed to push HEAD to revision walker")?;
        
        let mut commits = Vec::new();
        
        for (i, oid_result) in revwalk.enumerate() {
            if i >= max_count {
                break;
            }
            
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            
            let commit_info = CommitInfo {
                id: commit.id().to_string(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
                email: commit.author().email().unwrap_or("").to_string(),
                time: commit.time().seconds(),
                message: commit.message().unwrap_or("").to_string(),
            };
            
            commits.push(commit_info);
        }
        
        Ok(commits)
    }
    
    pub fn search_commits(repo_path: &Path, query: &str) -> Result<Vec<CommitInfo>> {
        let repo = Repository::open(repo_path)
            .context("Failed to open git repository")?;
        
        let mut revwalk = repo.revwalk()
            .context("Failed to create revision walker")?;
        
        revwalk.push_head()
            .context("Failed to push HEAD to revision walker")?;
        
        let query_lower = query.to_lowercase();
        let mut matching_commits = Vec::new();
        
        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            
            let message = commit.message().unwrap_or("").to_lowercase();
            let author = commit.author().name().unwrap_or("").to_lowercase();
            
            if message.contains(&query_lower) || author.contains(&query_lower) {
                let commit_info = CommitInfo {
                    id: commit.id().to_string(),
                    author: commit.author().name().unwrap_or("Unknown").to_string(),
                    email: commit.author().email().unwrap_or("").to_string(),
                    time: commit.time().seconds(),
                    message: commit.message().unwrap_or("").to_string(),
                };
                
                matching_commits.push(commit_info);
            }
        }
        
        Ok(matching_commits)
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub message: String,
}
