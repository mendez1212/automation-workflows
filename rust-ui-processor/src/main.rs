mod processor;
mod gallery;
mod utils;
mod generate_readme_preview;

use std::path::PathBuf;
use clap::Parser;
use log::{info, warn, LevelFilter};
use anyhow::{Result, Context};
use std::fs;

/// Image processor for GitHub Actions workflow
#[derive(Parser, Debug)]
#[clap(name = "image-processor", about = "Process PNG images for falconsoft25 repositories")]
struct Args {
    /// Path to the images folder
    #[clap(long, default_value = "docs/ui/")]
    image_folder: String,

    /// Enable or disable gallery generation
    #[clap(long, default_value = "true")]
    enable_gallery: String,

    /// Path to the README.md file
    #[clap(long, default_value = "README.md")]
    readme_path: String,

    /// Maximum image width in pixels
    #[clap(long, default_value = "300")]
    max_width: u32,

    /// Check image size before processing
    #[clap(long, default_value = "true")]
    check_size: String,

    /// Check border radius before processing
    #[clap(long, default_value = "true")]
    check_radius: String,

    /// Target border radius percentage
    #[clap(long, default_value = "6.5")]
    target_radius: f32,

    /// Use fast check for radius detection
    #[clap(long, default_value = "true")]
    fast_check: String,

    /// Number of columns for preview and gallery (1 or 2)
    #[clap(long, default_value = "2")]
    columns: u32,
}

fn main() -> Result<()> {
    // Initialize logging with specific timestamp format
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            use std::io::Write;
            let utc = chrono::Utc::now();
            writeln!(
                buf,
                "[{}Z {:5} {}] {}",
                utc.format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                env!("CARGO_PKG_NAME"),
                record.args()
            )
        })
        .init();
    
    // Parse command line arguments
    let mut args = Args::parse();
    
    // Convert string flags to booleans
    let enable_gallery = args.enable_gallery.to_lowercase() == "true";
    let check_size = args.check_size.to_lowercase() == "true";
    let check_radius = args.check_radius.to_lowercase() == "true";
    let fast_check = args.fast_check.to_lowercase() == "true";
    
    // Log startup information
    info!("Starting image processor");
    info!("Image folder: {}", args.image_folder);
    if check_size {
        info!("Size check enabled (max width: {}px)", args.max_width);
    }
    if check_radius {
        info!("Radius check enabled (target: {}%)", args.target_radius);
    }
    info!("ui-gallery is {}", if enable_gallery { "on" } else { "off" });
    info!("ui-preview is {}", if PathBuf::from(&args.readme_path).exists() { "on" } else { "off" });
    info!("Layout: {} column(s)", args.columns);

    // Validate columns parameter
    if args.columns != 1 && args.columns != 2 {
        warn!("Invalid number of columns ({}). Using default of 2 columns.", args.columns);
        args.columns = 2;
    }
    
    // Create image folder path
    let image_folder = PathBuf::from(&args.image_folder);
    
    // Check if the image folder exists
    if !image_folder.exists() {
        warn!("Image folder '{}' does not exist. Creating it...", args.image_folder);
        std::fs::create_dir_all(&image_folder)
            .context(format!("Failed to create image folder '{}'", args.image_folder))?;
    }
    
    // Process images with the converted boolean flags
    let processed_count = processor::process_images(
        &image_folder,
        args.max_width,
        check_size,
        check_radius,
        args.target_radius,
        fast_check
    )
        .context("Failed to process images")?;
    
    info!("Successfully processed {} images", processed_count);

    // Find numbered PNG files first - we'll need this for both README and gallery
    let numbered_images = gallery::find_numbered_images(&image_folder)?;

    // Update README.md first
    let readme_path = PathBuf::from(&args.readme_path);
    if readme_path.exists() {
        // Use the bool value we converted earlier
        let should_create_gallery = enable_gallery && numbered_images.len() > 4;
        generate_readme_preview::update_readme_preview(&readme_path, &numbered_images, &image_folder, should_create_gallery, args.columns)?;
    }
    
    // Generate gallery if enabled and there are more than 4 images
    if enable_gallery && numbered_images.len() > 4 {
        let gallery_path = PathBuf::from("docs/ui-gallery.md");
        match gallery::generate_gallery(&image_folder, &gallery_path, &numbered_images, args.columns) {
            Ok(image_count) => info!("Generated gallery with {} images", image_count),
            Err(e) => warn!("Failed to generate gallery: {}", e),
        }
    } else {
        info!("Skipping gallery creation: {} images found (minimum 5 required)", numbered_images.len());
        // Remove existing gallery if it exists and we have 4 or fewer images
        let gallery_path = PathBuf::from("docs/ui-gallery.md");
        if gallery_path.exists() && numbered_images.len() <= 4 {
            if let Err(e) = fs::remove_file(&gallery_path) {
                warn!("Failed to remove existing gallery: {}", e);
            } else {
                info!("Removed existing gallery as image count is 4 or fewer");
            }
        }
    }
    
    info!("Image processing completed successfully");
    Ok(())
}