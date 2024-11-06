use sha2::{Sha256, Digest};
use uuid::Uuid;
use std::fs;
use std::path::Path;


pub fn calculate_hash_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

pub fn generate_commit_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn copy_dir_contents(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        
        // Skip the .mini-git directory
        if src_path.ends_with(".mini-git") {
            continue;
        }
        
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
