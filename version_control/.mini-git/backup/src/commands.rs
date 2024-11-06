use std::env;
use std::fs;
use std::path::{Path};
use walkdir::WalkDir;

use crate::repository::Repository;
use crate::utils;

// Initialize a new repository in the current directory
pub fn init() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo_dir = working_dir.join(".mini-git");

    // Check if repository already exists
    if repo_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Repository already initialized",
        ));
    }

    // Create repository directory and initialize repository
    fs::create_dir_all(&repo_dir)?;
    let repo = Repository::new(working_dir);
    repo.save()?;
    println!("Initialized empty repository");
    Ok(())
}

// Add files to the staging area
// This function now handles multiple paths and properly processes directories
pub fn add(paths: &[String]) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let mut repo = Repository::load(working_dir.clone())?;
    let mut files_added = false;

    for path_str in paths {
        let path = Path::new(path_str);
        
        if path_str == "." {
            // Handle adding all files in current directory
            for entry in WalkDir::new(&working_dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let entry_path = entry.path();
                // Skip .mini-git directory and hidden files
                if !entry_path.starts_with(working_dir.join(".mini-git")) && 
                   !entry_path.to_string_lossy().contains("/.") {
                    match repo.stage_file(entry_path) {
                        Ok(_) => {
                            println!("Added: {}", entry_path.display());
                            files_added = true;
                        }
                        Err(e) => eprintln!("Error adding {}: {}", entry_path.display(), e),
                    }
                }
            }
        } else if path.is_file() {
            // Handle single file
            match repo.stage_file(path) {
                Ok(_) => {
                    println!("Added: {}", path.display());
                    files_added = true;
                }
                Err(e) => eprintln!("Error adding {}: {}", path.display(), e),
            }
        } else if path.is_dir() {
            // Handle directory recursively
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let entry_path = entry.path();
                if !entry_path.to_string_lossy().contains("/.") {
                    match repo.stage_file(entry_path) {
                        Ok(_) => {
                            println!("Added: {}", entry_path.display());
                            files_added = true;
                        }
                        Err(e) => eprintln!("Error adding {}: {}", entry_path.display(), e),
                    }
                }
            }
        } else {
            eprintln!("Warning: Path not found or inaccessible: {}", path_str);
        }
    }

    if files_added {
        repo.save()?;
        println!("Changes staged successfully");
    } else {
        println!("No files were added");
    }
    
    Ok(())
}

// Rest of the file remains unchanged
pub fn commit(message: &str) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let mut repo = Repository::load(working_dir)?;
    repo.commit(message)?;
    println!("Created commit: {}", message);
    Ok(())
}

pub fn history() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir)?;

    if repo.commits.is_empty() {
        println!("No commits yet");
        return Ok(());
    }

    for commit in repo.commits.iter().rev() {
        println!(
            "Commit: {}\nDate: {}\nMessage: {}\n",
            &commit.id[..8],
            commit.timestamp,
            commit.message
        );
    }
    Ok(())
}

pub fn push() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir.clone())?;
    
    let remote_dir = working_dir.join(".mini-git/remote");
    fs::create_dir_all(&remote_dir)?;
    
    let remote_repo_file = remote_dir.join("repository.json");
    let serialized = serde_json::to_string_pretty(&repo)?;
    fs::write(remote_repo_file, serialized)?;
    
    println!("Pushed changes to remote");
    Ok(())
}

pub fn pull() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let remote_dir = working_dir.join(".mini-git/remote");
    let remote_repo_file = remote_dir.join("repository.json");
    
    if !remote_repo_file.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No remote repository found",
        ));
    }
    
    let content = fs::read_to_string(remote_repo_file)?;
    let remote_repo: Repository = serde_json::from_str(&content)?;
    
    let repo_file = working_dir.join(".mini-git/repository.json");
    let serialized = serde_json::to_string_pretty(&remote_repo)?;
    fs::write(repo_file, serialized)?;
    
    println!("Pulled changes from remote");
    Ok(())
}

pub fn checkout(commit_id: &str) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir.clone())?;
    
    let commit = match repo.get_commit(commit_id) {
        Some(commit) => commit,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Commit not found",
            ));
        }
    };
    
    let backup_dir = working_dir.join(".mini-git/backup");
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)?;
    }
    utils::copy_dir_contents(&working_dir, &backup_dir)?;
    
    for (path, content_hash) in &commit.files {
        let file_path = working_dir.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = repo.get_object(content_hash)?;
        fs::write(&file_path, content)?;
    }
    
    println!("Checked out commit: {}", &commit.id[..8]);
    Ok(())
}
