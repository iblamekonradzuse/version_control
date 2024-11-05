use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub files: HashMap<String, String>, // path -> content hash
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub commits: Vec<Commit>,
    pub staging: HashMap<String, String>, // path -> content hash
    pub working_dir: PathBuf,
}

impl Repository {
    pub fn new(working_dir: PathBuf) -> Self {
        Repository {
            commits: Vec::new(),
            staging: HashMap::new(),
            working_dir,
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let repo_dir = self.working_dir.join(".mini-git");
        fs::create_dir_all(&repo_dir)?;
        let repo_file = repo_dir.join("repository.json");
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(repo_file, serialized)?;
        Ok(())
    }

    pub fn load(working_dir: PathBuf) -> std::io::Result<Self> {
        let repo_file = working_dir.join(".mini-git/repository.json");
        if !repo_file.exists() {
            return Ok(Repository::new(working_dir));
        }
        let content = fs::read_to_string(repo_file)?;
        let repo: Repository = serde_json::from_str(&content)?;
        Ok(repo)
    }

    pub fn stage_file(&mut self, path: &Path) -> std::io::Result<()> {
        // Read file as bytes instead of string to handle binary files
        let content = fs::read(path)?;
        let hash = utils::calculate_hash_bytes(&content);
        
        // Get the canonical paths to handle path resolution correctly
        let working_dir = self.working_dir.canonicalize()?;
        let canonical_path = path.canonicalize()?;
        
        // Ensure the file is within the working directory
        if !canonical_path.starts_with(&working_dir) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File is outside repository",
            ));
        }

        // Calculate relative path safely
        let relative_path = match canonical_path.strip_prefix(&working_dir) {
            Ok(rel_path) => rel_path.to_string_lossy().into_owned(),
            Err(_) => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Could not determine relative path",
            )),
        };

        self.staging.insert(relative_path, hash);
        Ok(())
    }

    pub fn commit(&mut self, message: &str) -> std::io::Result<()> {
        if self.staging.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Nothing to commit",
            ));
        }

        let commit = Commit {
            id: utils::generate_commit_id(),
            message: message.to_string(),
            timestamp: Utc::now(),
            files: self.staging.clone(),
        };

        self.commits.push(commit);
        self.staging.clear();
        self.save()?;
        Ok(())
    }

    pub fn get_commit(&self, commit_id: &str) -> Option<&Commit> {
        self.commits.iter().find(|c| c.id.starts_with(commit_id))
    }
}
