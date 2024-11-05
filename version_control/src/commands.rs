// commands.rs
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::repository::Repository;
use crate::utils;

pub fn init() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo_dir = working_dir.join(".mini-git");

    if repo_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Repository already initialized",
        ));
    }

    fs::create_dir_all(&repo_dir)?;
    let repo = Repository::new(working_dir);
    repo.save()?;
    println!("Initialized empty repository");
    Ok(())
}

pub fn add(path: &str) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let mut repo = Repository::load(working_dir.clone())?;

    if path == "." {
        for entry in WalkDir::new(&working_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if !path.starts_with(working_dir.join(".mini-git")) {
                repo.stage_file(path)?;
            }
        }
    } else {
        let path = Path::new(path);
        if path.is_file() {
            repo.stage_file(path)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ));
        }
    }

    repo.save()?;
    println!("Added files to staging area");
    Ok(())
}

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
    
    // In a real implementation, this would push to a remote repository
    // For this example, we'll just save to a "remote" directory
    let remote_dir = working_dir.join(".mini-git/remote");
    fs::create_dir_all(&remote_dir)?;
    
    // Save the repository state to the remote
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
    
    // Load the remote repository state
    let content = fs::read_to_string(remote_repo_file)?;
    let remote_repo: Repository = serde_json::from_str(&content)?;
    
    // Update local repository with remote state
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
    
    // Create a backup of the current state
    let backup_dir = working_dir.join(".mini-git/backup");
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)?;
    }
    utils::copy_dir_contents(&working_dir, &backup_dir)?;
    
    // Restore files from the commit
    for (path, _) in &commit.files {
        let file_path = working_dir.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        // In a real implementation, we would restore the file content from a blob store
        // For this example, we'll just create empty files
        fs::write(&file_path, "")?;
    }
    
    println!("Checked out commit: {}", &commit.id[..8]);
    Ok(())
}
