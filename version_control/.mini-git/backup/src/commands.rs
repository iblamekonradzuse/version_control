use std::env;
use std::fs;
use std::path::{Path };
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
// This function handles both single files and directories
pub fn add(path: &str) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let mut repo = Repository::load(working_dir.clone())?;

    // Handle the case when "." is provided (add all files)
    if path == "." {
        for entry in WalkDir::new(&working_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            // Skip .mini-git directory
            if !path.starts_with(working_dir.join(".mini-git")) {
                repo.stage_file(path)?;
            }
        }
    } else {
        let path = Path::new(path);
        if path.is_file() {
            // Handle single file
            repo.stage_file(path)?;
        } else if path.is_dir() {
            // Handle directory by recursively adding all files
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                repo.stage_file(entry.path())?;
            }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Path not found or is not accessible",
            ));
        }
    }

    repo.save()?;
    println!("Added files to staging area");
    Ok(())
}

// Create a new commit with the current staged changes
pub fn commit(message: &str) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let mut repo = Repository::load(working_dir)?;
    repo.commit(message)?;
    println!("Created commit: {}", message);
    Ok(())
}

// Display commit history
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

// Push changes to remote repository
pub fn push() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir.clone())?;
    
    // Create remote directory if it doesn't exist
    let remote_dir = working_dir.join(".mini-git/remote");
    fs::create_dir_all(&remote_dir)?;
    
    // Save repository state to remote
    let remote_repo_file = remote_dir.join("repository.json");
    let serialized = serde_json::to_string_pretty(&repo)?;
    fs::write(remote_repo_file, serialized)?;
    
    println!("Pushed changes to remote");
    Ok(())
}

// Pull changes from remote repository
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
    
    // Load remote repository state
    let content = fs::read_to_string(remote_repo_file)?;
    let remote_repo: Repository = serde_json::from_str(&content)?;
    
    // Update local repository
    let repo_file = working_dir.join(".mini-git/repository.json");
    let serialized = serde_json::to_string_pretty(&remote_repo)?;
    fs::write(repo_file, serialized)?;
    
    println!("Pulled changes from remote");
    Ok(())
}

// Checkout a specific commit
pub fn checkout(commit_id: &str) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir.clone())?;
    
    // Find the specified commit
    let commit = match repo.get_commit(commit_id) {
        Some(commit) => commit,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Commit not found",
            ));
        }
    };
    
    // Create backup of current state
    let backup_dir = working_dir.join(".mini-git/backup");
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)?;
    }
    utils::copy_dir_contents(&working_dir, &backup_dir)?;
    
    // Restore files from the commit
    for (path, content_hash) in &commit.files {
        let file_path = working_dir.join(path);
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Retrieve and write file content from object store
        let content = repo.get_object(content_hash)?;
        fs::write(&file_path, content)?;
    }
    
    println!("Checked out commit: {}", &commit.id[..8]);
    Ok(())
}
