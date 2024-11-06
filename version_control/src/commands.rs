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
        None => { //commit not found
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

pub fn loadlast() -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir)?;
    
    if repo.commits.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No commits found in repository",
        ));
    }
    
    // Get the last commit's ID
    let last_commit = repo.commits.last().unwrap();
    
    // Use the existing checkout function to load the last commit
    checkout(&last_commit.id)?;
    
    Ok(())
}

pub fn diff(commit_id1: Option<&str>, commit_id2: Option<&str>) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    let repo = Repository::load(working_dir.clone())?;

    if repo.commits.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No commits found in repository",
        ));
    }

    match (commit_id1, commit_id2) {
        // No arguments - compare working directory with last commit
        (None, None) => {
            let last_commit = repo.commits.last().unwrap();
            compare_with_working_dir(&repo, last_commit)?;
        }
        // One argument - compare working directory with specific commit
        (Some(commit_id), None) => {
            let commit = repo.get_commit(commit_id).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Commit not found")
            })?;
            compare_with_working_dir(&repo, commit)?;
        }
        // Two arguments - compare two commits
        (Some(commit_id1), Some(commit_id2)) => {
            let commit1 = repo.get_commit(commit_id1).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "First commit not found")
            })?;
            let commit2 = repo.get_commit(commit_id2).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Second commit not found")
            })?;
            compare_commits(&repo, commit1, commit2)?;
        }
        // Invalid case
        (None, Some(_)) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid diff command usage",
            ));
        }
    }

    Ok(())
}

fn compare_with_working_dir(_repo: &Repository, commit: &crate::repository::Commit) -> std::io::Result<()> {
    let working_dir = env::current_dir()?;
    println!("Comparing working directory with commit {} ({})", &commit.id[..8], commit.message);
    println!("----------------------------------------");

    // Check files in commit
    for (path, commit_hash) in &commit.files {
        let file_path = working_dir.join(path);
        if file_path.exists() {
            let current_content = fs::read(&file_path)?;
            let current_hash = utils::calculate_hash_bytes(&current_content);
            
            if &current_hash != commit_hash {
                println!("Modified: {}", path);
                // You could add more detailed diff information here
            }
        } else {
            println!("Deleted: {}", path);
        }
    }

    // Check for new files
    for entry in WalkDir::new(&working_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if !path.starts_with(working_dir.join(".mini-git")) {
            let relative_path = path.strip_prefix(&working_dir).unwrap().to_string_lossy();
            if !commit.files.contains_key(&*relative_path) {
                println!("New file: {}", relative_path);
            }
        }
    }

    Ok(())
}

fn compare_commits(_repo: &Repository, commit1: &crate::repository::Commit, commit2: &crate::repository::Commit) -> std::io::Result<()> {
    println!(
        "Comparing commit {} ({}) with {} ({})",
        &commit1.id[..8],
        commit1.message,
        &commit2.id[..8],
        commit2.message
    );
    println!("----------------------------------------");

    // Check for modified and deleted files
    for (path, hash1) in &commit1.files {
        match commit2.files.get(path) {
            Some(hash2) if hash1 != hash2 => {
                println!("Modified: {}", path);
                // You could add more detailed diff information here
            }
            None => println!("Deleted in second commit: {}", path),
            _ => {} // File unchanged
        }
    }

    // Check for new files in commit2
    for path in commit2.files.keys() {
        if !commit1.files.contains_key(path) {
            println!("Added in second commit: {}", path);
        }
    }

    Ok(())
}
