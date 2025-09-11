use std::path::Path;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{Result, Context};
use image::{ImageFormat, GenericImageView, ImageEncoder};
use rayon::prelude::*;
use log::{info, warn, debug, error};

use crate::utils;

// Constants
const CORNER_RADIUS_PERCENT: f32 = 6.5;
const ALPHA_THRESHOLD: u8 = 250;  // Consider pixels with alpha > 250 as opaque

/// Process all PNG images in the specified folder
pub fn process_images(
    folder_path: &Path,
    max_width: u32,
    check_size: bool,
    check_radius: bool,
    target_radius: f32,
    fast_check: bool
) -> Result<usize> {
    debug!("Looking for PNG images in {}", folder_path.display());
    
    // Find all PNG files in the folder
    let png_files = utils::find_png_files(folder_path)?;
    
    if png_files.is_empty() {
        info!("No PNG files found in {}", folder_path.display());
        return Ok(0);
    }
    
    info!("Found {} PNG files to process", png_files.len());
    
    // Process images in parallel
    let processed_count = Arc::new(AtomicUsize::new(0));
    let processed_count_clone = Arc::clone(&processed_count);
    
    png_files.par_iter()
        .for_each(|file_path| {
            match process_single_image(file_path, max_width, check_size, check_radius, target_radius, fast_check) {
                Ok((processed, resize_done, radius_done, resize_time, radius_time)) => {
                    if processed {
                        processed_count_clone.fetch_add(1, Ordering::SeqCst);
                        if resize_done && radius_done {
                            info!("Applied resize ({:?}) and radius ({:?}) to {}", 
                                resize_time.unwrap_or_default(), 
                                radius_time.unwrap_or_default(), 
                                file_path.display());
                        } else if resize_done {
                            info!("Applied resize ({:?}) to {}", 
                                resize_time.unwrap_or_default(), 
                                file_path.display());
                        } else if radius_done {
                            info!("Applied radius ({:?}) to {}", 
                                radius_time.unwrap_or_default(), 
                                file_path.display());
                        }
                    } else {
                        debug!("Skipped: {} (already optimized)", file_path.display());
                    }
                },
                Err(e) => {
                    error!("Failed to process {}: {}", file_path.display(), e);
                }
            }
        });
    
    Ok(processed_count.load(Ordering::SeqCst))
}

/// Process a single image file
/// Returns (was_processed, resize_applied, radius_applied, resize_time, radius_time)
fn process_single_image(
    file_path: &Path,
    max_width: u32,
    check_size: bool,
    check_radius: bool,
    target_radius: f32,
    _fast_check: bool
) -> Result<(bool, bool, bool, Option<std::time::Duration>, Option<std::time::Duration>)> {
    // Open the image
    let mut img = image::open(file_path)
        .with_context(|| format!("Failed to open image {}", file_path.display()))?;
    
    // Check if image format is PNG
    if !is_png(file_path)? {
        warn!("{} is not a PNG file, skipping", file_path.display());
        return Ok((false, false, false, None, None));
    }
    
    // Get current dimensions before any processing
    let (width, height) = img.dimensions();
    let mut modified = false;
    
    // Check if we need any processing at all
    let mut needs_resize = false;
    let mut needs_radius = false;
    let mut resize_time = None;
    let mut radius_time = None;

    // Check resize requirements
    if check_size && width > max_width {
        needs_resize = true;
        debug!("Image needs resize: {}x{} -> {}x{}", 
               width, height, max_width, (height * max_width) / width);
    }

    // Check radius requirements - only check top-right corner
    if check_radius {
        if let Some(rgba) = img.as_rgba8() {
            let _corner_size = (width as f32 * (target_radius / 100.0)) as u32;
            
            // Check exactly 6 pixels in top-right corner
            let check_points = [
                (width - 1, 0),      // Top edge
                (width - 1, 1),      // One pixel down
                (width - 2, 1),      // Diagonal in
                (width - 2, 2),      // More diagonal
                (width - 3, 1),      // Further in
                (width - 3, 2),      // Last check point
            ];
            
            // Check if ANY of these points are opaque (meaning no radius)
            needs_radius = check_points.iter().any(|(x, y)| {
                rgba.get_pixel(*x, *y)[3] > ALPHA_THRESHOLD
            });
            
            if needs_radius {
                debug!("Image needs corner rounding: {}", file_path.display());
            }
        } else {
            // If no alpha channel, needs radius
            needs_radius = true;
        }
    }

    // If no processing needed at all, return early
    if !needs_resize && !needs_radius {
        if check_size && check_radius {
            info!("{} already meets size and radius requirements ({}x{})", file_path.display(), width, height);
        } else if check_size {
            info!("{} already meets size requirements ({}x{})", file_path.display(), width, height);
        } else if check_radius {
            info!("{} already meets radius requirements", file_path.display());
        }
        return Ok((false, false, false, None, None));
    }
    
    // Do all needed transformations
    if needs_resize || needs_radius {

        // Resize if needed
        if needs_resize {
            debug!("Resizing {} from {}x{} to {}x{} (aspect ratio preserved)", 
                   file_path.display(), width, height, max_width, (height * max_width) / width);
            
            // Calculate new height, preserving aspect ratio
            let new_height = (height as f32 * (max_width as f32 / width as f32)).round() as u32;
            
            // Resize the image and measure time
            let start = std::time::Instant::now();
            img = img.resize(max_width, new_height, image::imageops::FilterType::Lanczos3);
            resize_time = Some(start.elapsed());
            modified = true;
        }
        
        // Apply corner rounding if needed
        if needs_radius {
            debug!("Applying rounded corners to {}", file_path.display());
            let start = std::time::Instant::now();
            img = apply_rounded_corners(img);
            radius_time = Some(start.elapsed());
            modified = true;
        }
    }
    
    // Save the image if modified and return what was done
    if modified {
        // Use custom encoder to set compression level
        let file = fs::File::create(file_path)
            .with_context(|| format!("Failed to create file {}", file_path.display()))?;
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            file,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::Sub,
        );
        
        // Get raw image data
        let (width, height) = img.dimensions();
        let data = img.as_bytes();
        let color_type = img.color();
        
        // Encode and save
        encoder.write_image(data, width, height, color_type)
            .with_context(|| format!("Failed to save processed image {}", file_path.display()))?;
        return Ok((true, needs_resize, needs_radius, resize_time, radius_time));
    } else {
        debug!("{} already meets all requirements", file_path.display());
        return Ok((false, false, false, None, None));
    }
}

/// Check if the file is a PNG image
fn is_png(file_path: &Path) -> Result<bool> {
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());
    
    if extension != Some("png".to_string()) {
        return Ok(false);
    }
    
    // Additional check by reading image header
    let file = fs::File::open(file_path)
        .with_context(|| format!("Failed to open file {}", file_path.display()))?;
    
    let format = image::io::Reader::new(std::io::BufReader::new(file))
        .with_guessed_format()
        .with_context(|| format!("Failed to read image format for {}", file_path.display()))?
        .format();
    
    Ok(format == Some(ImageFormat::Png))
}

/// Apply rounded corners to an image with anti-aliasing for smooth edges
fn apply_rounded_corners(img: image::DynamicImage) -> image::DynamicImage {
    let (width, height) = img.dimensions();
    let radius = (width as f32 * CORNER_RADIUS_PERCENT / 100.0).round() as u32;
    let radius_f32 = radius as f32;
    
    debug!("Applying rounded corners with {}px radius and anti-aliasing", radius);
    
    // Convert to RGBA
    let mut rgba = img.to_rgba8();
    
    // This function calculates the alpha value for a pixel based on its distance from the corner
    // using anti-aliasing for smooth transitions
    let calculate_alpha = |x: u32, y: u32, corner_x: f32, corner_y: f32| -> u8 {
        let dx = x as f32 - corner_x;
        let dy = y as f32 - corner_y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Full transparency outside the radius
        if distance >= radius_f32 + 1.0 {
            return 0;
        }
        
        // Full opacity inside the radius
        if distance <= radius_f32 - 1.0 {
            return 255;
        }
        
        // Anti-aliased transition at the edge
        let alpha_f = ((radius_f32 + 1.0 - distance) * 255.0).clamp(0.0, 255.0);
        alpha_f as u8
    };
    
    // Process each corner
    for y in 0..height {
        for x in 0..width {
            let mut alpha = 255; // Default full opacity
            
            // Top-left corner
            if x < radius && y < radius {
                alpha = calculate_alpha(x, y, radius as f32, radius as f32);
            }
            // Top-right corner
            else if x >= width - radius && y < radius {
                alpha = calculate_alpha(x, y, (width - radius - 1) as f32, radius as f32);
            }
            // Bottom-left corner
            else if x < radius && y >= height - radius {
                alpha = calculate_alpha(x, y, radius as f32, (height - radius - 1) as f32);
            }
            // Bottom-right corner
            else if x >= width - radius && y >= height - radius {
                alpha = calculate_alpha(x, y, (width - radius - 1) as f32, (height - radius - 1) as f32);
            }
            
            // Apply the calculated alpha
            if alpha < 255 {
                let pixel = rgba.get_pixel_mut(x, y);
                // Preserve the RGB values but adjust the alpha channel
                pixel[3] = (pixel[3] as u16 * alpha as u16 / 255) as u8;
            }
        }
    }
    
    image::DynamicImage::ImageRgba8(rgba)
}