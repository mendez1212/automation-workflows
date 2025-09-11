use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use log::{info, debug};
use std::cmp::min;

const README_PREVIEW_SECTION: &str = "\n## UI Preview\n\n";
const GALLERY_BUTTON: &str = "<p align=\"center\">\n  <a href=\"docs/ui-gallery.md\">\n    <img src=\"https://img.shields.io/badge/See%20All%20UI%20Images-2b90d9\" alt=\"See All UI Images\" width=\"200\" height=\"50\">\n  </a>\n</p>\n\n";
const REPO_CREATION_MARKER: &str = "> **Repository created on:**";

/// Find the best position to insert the preview section based on the following priority:
/// 1. Before the first "---" that appears before repository creation timestamp
/// 2. Before the repository creation timestamp if no separator exists
/// 3. At the end of the content if neither exists
fn find_preview_insertion_position(content: &str) -> (usize, bool) {
    let repo_marker_pos = content.find(REPO_CREATION_MARKER);
    
    // Case 1: Look for separator before repo marker
    if let Some(repo_pos) = repo_marker_pos {
        if let Some(separator_pos) = content[..repo_pos].rfind("\n---\n") {
            // Found a separator - insert before it and don't add another separator
            return (separator_pos + 1, false);
        }
        // Case 2: No separator, but we have repo marker - add one
        let prev_newline = content[..repo_pos].rfind('\n').unwrap_or(0);
        return (prev_newline + 1, true);
    }
    
    // Case 3: Append at the end, with proper spacing
    (content.len(), true)
}

pub fn update_readme_preview(
    readme_path: &Path,
    images: &[(u32, PathBuf)],
    _base_path: &Path,
    show_gallery_button: bool,
    columns: u32
) -> Result<()> {
    // Skip if no images found
    if images.is_empty() {
        debug!("No images found, skipping README preview update");
        return Ok(());
    }

    debug!("Checking README preview section");

    // Read the current README content
    let current_content = if readme_path.exists() {
        fs::read_to_string(readme_path)
            .context(format!("Failed to read README at {}", readme_path.display()))?
    } else {
        String::new()
    };

    // Generate new preview section
    let mut preview = String::from(README_PREVIEW_SECTION);
    
    // Display up to 4 images in a configurable layout
    let display_count = std::cmp::min(4, images.len());
    let mut i = 0;
    
    while i < display_count {
        let row_items = min(columns as usize, display_count - i);
        
        // Add image names for current row
        preview.push('|');
        for j in 0..row_items {
            let (num, path) = &images[i + j];
            let name = super::gallery::get_image_name(path)?;
            preview.push_str(&format!("{}{} 🔽|", name, num));
        }
        preview.push('\n');
        
        // Add alignment separators
        preview.push('|');
        for _ in 0..row_items {
            preview.push_str(":---------------:|");
        }
        preview.push('\n');
        
        // Add images for current row
        preview.push('|');
        for j in 0..row_items {
            let (_, path) = &images[i + j];
            let name = super::gallery::get_image_name(path)?;
            let rel_path = super::gallery::get_relative_path_for_readme(path)?;
            preview.push_str(&format!("![{}]({})|", name, rel_path));
        }
        preview.push('\n');
        
        i += row_items;
        
        // Add a newline between rows except for the last row
        if i < display_count {
            preview.push('\n');
        }
    }

    // Add gallery button if needed
    if show_gallery_button {
        preview.push_str(GALLERY_BUTTON);
    }

    // Remove any existing preview section
    let mut new_content = current_content.clone();
    if let Some(start) = new_content.find(README_PREVIEW_SECTION) {
        if let Some(end) = new_content[start..].find("\n---\n") {
            // Remove the section including the separator
            new_content.replace_range(start..start + end + 5, "");
        } else {
            // If no separator found, try to find the next section header
            if let Some(end) = new_content[start..].find("\n## ") {
                new_content.replace_range(start..start + end, "");
            } else {
                // If no next section, remove to the end
                new_content.truncate(start);
            }
        }
    }

    // Find the best position to insert the preview
    let (insert_pos, needs_separator) = find_preview_insertion_position(&new_content);
    
    // Prepare the content for insertion
    let mut insert_content = preview.trim_end().to_string();
    
    // Add separator after preview content if needed and one doesn't exist
    if needs_separator {
        insert_content.push_str("\n---\n");
    }
    
    // Ensure proper spacing around the insertion
    if insert_pos > 0 && !new_content[insert_pos-1..insert_pos].contains('\n') {
        insert_content = format!("\n{}", insert_content);
    }
    if insert_pos < new_content.len() && !new_content[insert_pos..insert_pos+1].contains('\n') {
        insert_content.push('\n');
    }
    
    // Insert the preview section
    new_content.insert_str(insert_pos, &insert_content);
    
    // Normalize line endings and multiple newlines
    new_content = new_content.replace("\r\n", "\n");
    while new_content.contains("\n\n\n") {
        new_content = new_content.replace("\n\n\n", "\n\n");
    }

    // Write the updated content
    fs::write(readme_path, new_content)
        .context(format!("Failed to update README at {}", readme_path.display()))?;
    info!("Updated README.md UI preview content");

    Ok(())
}
