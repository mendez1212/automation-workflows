use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context, anyhow};
use regex::Regex;
use log::{info, warn, debug};

/// Generate a UI gallery markdown file with configurable column layout
pub fn generate_gallery(image_folder: &Path, gallery_path: &Path, numbered_images: &[(u32, PathBuf)], columns: u32) -> Result<usize> {
    debug!("Processing UI gallery at {} with {} column(s)", gallery_path.display(), columns);
    
    // Check if parent directory exists
    if let Some(parent) = gallery_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory {}", parent.display()))?;
        }
    }

    if numbered_images.is_empty() {
        warn!("No numbered PNG images found in {}", image_folder.display());
        
        // If gallery exists and has content, clean it up
        if gallery_path.exists() {
            // Keep only the title
            fs::write(gallery_path, "# UI Gallery\n")
                .context(format!("Failed to clean up gallery at {}", gallery_path.display()))?;
            info!("Cleaned up gallery");
        }
        return Ok(0);
    }
    
    info!("Found {} numbered PNG images for gallery", numbered_images.len());
    
    // Generate new markdown content
    let new_markdown = generate_markdown_table(image_folder, numbered_images, columns)?;
    
    // If gallery exists, read its content and compare
    if gallery_path.exists() {
        let existing_content = fs::read_to_string(gallery_path)
            .context(format!("Failed to read existing gallery at {}", gallery_path.display()))?;
        
        // Only update if content is different
        if existing_content != new_markdown {
            info!("Updating ui-gallery.md content");
            fs::write(gallery_path, new_markdown)
                .context(format!("Failed to update gallery at {}", gallery_path.display()))?;
        } else {
            info!("ui-gallery.md content is up to date");
        }
    } else {
        // Create new gallery file
        info!("Creating new ui-gallery.md file");
        fs::write(gallery_path, new_markdown)
            .context(format!("Failed to write gallery to {}", gallery_path.display()))?;
    }
    
    Ok(numbered_images.len())
}

/// Find all PNG images with numeric suffixes and sort them by number
pub fn find_numbered_images(folder_path: &Path) -> Result<Vec<(u32, PathBuf)>> {
    debug!("Looking for numbered PNG images in {}", folder_path.display());
    
    // Updated regex to capture the full number at the end
    let re = Regex::new(r"^(.+?)[-](\d+)\.png$").unwrap();
    let mut numbered_files = Vec::new();
    
    // Check if folder exists
    if !folder_path.exists() {
        return Ok(Vec::new());
    }
    
    // Iterate through folder entries
    for entry in fs::read_dir(folder_path)
        .context(format!("Failed to read directory {}", folder_path.display()))? {
        
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        
        // Skip directories and non-PNG files
        if path.is_dir() || !is_png_file(&path) {
            continue;
        }
        
        // Get the filename as string
        let filename = path.file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Invalid filename"))?;
        
        // Check if filename ends with a number
        if let Some(captures) = re.captures(filename) {
            if let Some(number_str) = captures.get(2) {
                if let Ok(number) = number_str.as_str().parse::<u32>() {
                    numbered_files.push((number, path.clone()));
                }
            }
        }
    }
    
    // Sort by number
    numbered_files.sort_by_key(|(num, _)| *num);
    
    Ok(numbered_files)
}

/// Check if a file is a PNG image based on extension
fn is_png_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase() == "png")
        .unwrap_or(false)
}

/// Get image name from path without number and extension
pub fn get_image_name(path: &Path) -> Result<String> {
    let filename = path.file_stem()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid image name"))?;
    
    // Remove trailing numbers using regex
    let re = Regex::new(r"^(.+?)\d+$").unwrap();
    if let Some(captures) = re.captures(filename) {
        if let Some(name) = captures.get(1) {
            return Ok(name.as_str().replace("-", " ").to_string());
        }
    }
    Ok(filename.to_string())
}

/// Get image path relative to repository root (for README.md)
pub fn get_relative_path_for_readme(image_path: &Path) -> Result<String> {
    let file_name = image_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid image path"))?;
    Ok(format!("docs/ui/{}", file_name))
}

/// Get image path relative to gallery location (for ui-gallery.md)
fn get_relative_path_for_gallery(image_path: &Path) -> Result<String> {
    let file_name = image_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid image path"))?;
    Ok(format!("ui/{}", file_name))
}

// Constants for button HTML
const DETAILS_BUTTON: &str = "<p align=\"center\">\n  <a href=\"../docs/ui/\">\n    <img src=\"https://img.shields.io/badge/See%20Images%20in%20More%20Details-2b90d9\" alt=\"See Images in More Details\" width=\"240\" height=\"50\">\n  </a>\n</p>\n";

/// Generate markdown table based on the specified number of columns
fn generate_markdown_table(_image_folder: &Path, numbered_images: &[(u32, PathBuf)], mut columns: u32) -> Result<String> {
    // Validate columns parameter
    if columns != 1 && columns != 2 {
        warn!("Invalid number of columns ({}). Using default of 2 columns.", columns);
        columns = 2;
    }
    
    let mut markdown = String::from("# UI Gallery\n\n");
    let mut i = 0;
    
    while i < numbered_images.len() {
        // For single column, each item gets its own row
        if columns == 1 {
            // Add image name
            let (num, path) = &numbered_images[i];
            let name = get_image_name(path)?;
            markdown.push_str(&format!("|{}{} 🔽|\n", name, num));
            
            // Add alignment separator
            markdown.push_str("|:---------------:|\n");
            
            // Add image
            let rel_path = get_relative_path_for_gallery(path)?;
            markdown.push_str(&format!("|![{}]({})|\n\n", name, rel_path));
            
            i += 1;
        } else {
            // Two-column layout
            let row_items = std::cmp::min(2, numbered_images.len() - i);
            
            // Add image names for current row
            markdown.push('|');
            for j in 0..row_items {
                let (num, path) = &numbered_images[i + j];
                let name = get_image_name(path)?;
                markdown.push_str(&format!("{}{} 🔽|", name, num));
            }
            markdown.push('\n');
            
            // Add alignment separators
            markdown.push('|');
            for _ in 0..row_items {
                markdown.push_str(":---------------:|");
            }
            markdown.push('\n');
            
            // Add image row
            markdown.push('|');
            for j in 0..row_items {
                let (_, path) = &numbered_images[i + j];
                let name = get_image_name(path)?;
                let rel_path = get_relative_path_for_gallery(path)?;
                markdown.push_str(&format!("![{}]({})|", name, rel_path));
            }
            markdown.push_str("\n\n");
            
            i += 2;
        }
    }
    
    // Add the details button at the end
    markdown.push_str(DETAILS_BUTTON);
    
    Ok(markdown)
}
