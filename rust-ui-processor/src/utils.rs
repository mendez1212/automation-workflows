use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use glob::glob;
use log::debug;

/// Find all PNG files in a directory and its subdirectories
pub fn find_png_files(dir_path: &Path) -> Result<Vec<PathBuf>> {
    debug!("Searching for PNG files in {}", dir_path.display());
    
    let mut result = Vec::new();
    
    // Check if directory exists
    if !dir_path.exists() {
        return Ok(result);
    }
    
    // Use glob pattern to find all PNG files
    let pattern = dir_path.join("**/*.png");
    let pattern_str = pattern.to_string_lossy();
    
    for entry in glob(&pattern_str)
        .context(format!("Failed to read glob pattern {}", pattern_str))? {
        
        if let Ok(path) = entry {
            if path.is_file() {
                result.push(path);
            }
        }
    }
    
    debug!("Found {} PNG files", result.len());
    Ok(result)
}

/// Calculate checksum of a file to detect changes
#[allow(dead_code)]
pub fn calculate_file_checksum(file_path: &Path) -> Result<String> {
    use std::io::Read;
    use std::fs::File;
    
    let mut file = File::open(file_path)
        .context(format!("Failed to open file {}", file_path.display()))?;
    
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .context(format!("Failed to read file {}", file_path.display()))?;
    
    // Calculate simple hash for detection of changes
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash_slice(&contents, &mut hasher);
    
    Ok(format!("{:x}", std::hash::Hasher::finish(&hasher)))
}

/// Get the relative path of a file from a base directory
#[allow(dead_code)]
pub fn get_relative_path(base_path: &Path, full_path: &Path) -> Option<PathBuf> {
    full_path.strip_prefix(base_path).ok().map(|p| p.to_path_buf())
}

/// Returns the numeric suffix from a filename (e.g., "image5.png" returns 5)
#[allow(dead_code)]
pub fn extract_numeric_suffix(filename: &str) -> Option<u32> {
    use regex::Regex;
    
    // Create regex to extract numeric suffix
    let re = Regex::new(r"(\d+)\.png$").unwrap();
    
    re.captures(filename)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
}
