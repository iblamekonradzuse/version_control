use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils;

// Create a separate struct for backwards compatibility
#[derive(Debug, Serialize, Deserialize)]
struct OldRepository {
    pub commits: Vec<Commit>,
    pub staging: HashMap<String, String>,
    pub working_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub commits: Vec<Commit>,
    pub staging: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub objects: HashMap<String, Vec<u8>>,
}

impl Repository {
    pub fn new(working_dir: PathBuf) -> Self {
        Repository {
            commits: Vec::new(),
            staging: HashMap::new(),
            working_dir,
            objects: HashMap::new(),
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

        // Try to read the file content
        let content = fs::read_to_string(&repo_file)?;

        // First try to deserialize as the new format
        match serde_json::from_str::<Repository>(&content) {
            Ok(repo) => Ok(repo),
            Err(_) => {
                // If that fails, try to deserialize as old format and migrate
                let old_repo: OldRepository = serde_json::from_str(&content)?;
                
                // Create new repository with migrated data
                let mut new_repo = Repository {
                    commits: old_repo.commits,
                    staging: old_repo.staging,
                    working_dir,
                    objects: HashMap::new(),
                };

                // Optionally rebuild the objects store from working directory
                new_repo.rebuild_objects_store()?;

                // Save the migrated repository
                new_repo.save()?;

                Ok(new_repo)
            }
        }
    }

    // New helper function to rebuild objects store
    fn rebuild_objects_store(&mut self) -> std::io::Result<()> {
        self.objects.clear();
        
        // Rebuild from staged files
        for (path, hash) in &self.staging {
            let file_path = self.working_dir.join(path);
            if file_path.exists() {
                let content = fs::read(&file_path)?;
                self.objects.insert(hash.clone(), content);
            }
        }

        // Rebuild from committed files
        for commit in &self.commits {
            for (path, hash) in &commit.files {
                if !self.objects.contains_key(hash) {
                    let file_path = self.working_dir.join(path);
                    if file_path.exists() {
                        let content = fs::read(&file_path)?;
                        self.objects.insert(hash.clone(), content);
                    }
                }
            }
        }

        Ok(())
    }

    // Rest of the implementation remains the same
    pub fn stage_file(&mut self, path: &Path) -> std::io::Result<()> {
        let content = fs::read(path)?;
        let hash = utils::calculate_hash_bytes(&content);
        
        self.objects.insert(hash.clone(), content);
        
        let working_dir = self.working_dir.canonicalize()?;
        let canonical_path = path.canonicalize()?;
        
        if !canonical_path.starts_with(&working_dir) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "File is outside repository",
            ));
        }

        let relative_path = canonical_path
            .strip_prefix(&working_dir)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Could not determine relative path",
            ))?
            .to_string_lossy()
            .into_owned();

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

    pub fn get_object(&self, hash: &str) -> std::io::Result<Vec<u8>> {
        self.objects.get(hash).cloned().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Object not found in repository",
            )
        })
    }
}
