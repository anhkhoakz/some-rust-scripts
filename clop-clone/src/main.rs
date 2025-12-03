use arboard::{Clipboard, ImageData};
use clap::{Parser, Subcommand};
use image::{ImageBuffer, ImageFormat, RgbaImage};
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "clop")]
#[command(about = "CLI clipboard optimizer - optimize images in your clipboard", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Optimize the current clipboard image (default action)
    Optimize {
        /// Quality level (1-100, default: 85)
        #[arg(short, long, default_value_t = 85)]
        quality: u8,

        /// Aggressive optimization (lower quality)
        #[arg(short, long)]
        aggressive: bool,
    },
    /// Downscale clipboard image
    Downscale {
        /// Scale percentage (10-100)
        #[arg(short, long, default_value_t = 50)]
        scale: u8,

        /// Quality after downscaling (1-100, default: 85)
        #[arg(short, long, default_value_t = 85)]
        quality: u8,
    },
    /// Watch clipboard and auto-optimize
    Watch {
        /// Quality level (1-100, default: 85)
        #[arg(short, long, default_value_t = 85)]
        quality: u8,
    },
    /// Save clipboard image to file (with optimization)
    Save {
        /// Output file path
        output: PathBuf,

        /// Quality level (1-100, default: 85)
        #[arg(short, long, default_value_t = 85)]
        quality: u8,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Optimize {
            quality,
            aggressive,
        }) => {
            let q = if aggressive {
                quality.saturating_sub(20)
            } else {
                quality
            };
            optimize_clipboard(q);
        }
        Some(Commands::Downscale { scale, quality }) => {
            downscale_clipboard(scale.clamp(10, 100), quality);
        }
        Some(Commands::Watch { quality }) => {
            watch_clipboard(quality);
        }
        Some(Commands::Save { output, quality }) => {
            save_clipboard(output, quality);
        }
        None => {
            // Default: optimize with standard quality
            optimize_clipboard(85);
        }
    }
}

fn optimize_clipboard(quality: u8) {
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to access clipboard: {}", e);
            return;
        }
    };

    let image = match clipboard.get_image() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("No image in clipboard: {}", e);
            return;
        }
    };

    eprintln!("Optimizing {}×{} image...", image.width, image.height);

    let rgba_image: RgbaImage = match ImageBuffer::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.into_owned(),
    ) {
        Some(img) => img,
        None => {
            eprintln!("Failed to parse image data");
            return;
        }
    };

    let optimized = optimize_image(&rgba_image, quality);

    if let Err(e) = clipboard.set_image(ImageData {
        width: rgba_image.width() as usize,
        height: rgba_image.height() as usize,
        bytes: Cow::Owned(optimized),
    }) {
        eprintln!("Failed to set clipboard: {}", e);
        return;
    }

    eprintln!("✓ Image optimized and copied to clipboard");
}

fn downscale_clipboard(scale_percent: u8, quality: u8) {
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to access clipboard: {}", e);
            return;
        }
    };

    let image = match clipboard.get_image() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("No image in clipboard: {}", e);
            return;
        }
    };

    let original_width = image.width;
    let original_height = image.height;

    let rgba_image: RgbaImage = match ImageBuffer::from_raw(
        original_width as u32,
        original_height as u32,
        image.bytes.into_owned(),
    ) {
        Some(img) => img,
        None => {
            eprintln!("Failed to parse image data");
            return;
        }
    };

    let scale = scale_percent as f32 / 100.0;
    let new_width = ((original_width as f32) * scale) as u32;
    let new_height = ((original_height as f32) * scale) as u32;

    eprintln!(
        "Downscaling {}×{} → {}×{} ({}%)",
        original_width, original_height, new_width, new_height, scale_percent
    );

    let resized = image::imageops::resize(
        &rgba_image,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    );

    let optimized = optimize_image(&resized, quality);

    if let Err(e) = clipboard.set_image(ImageData {
        width: new_width as usize,
        height: new_height as usize,
        bytes: Cow::Owned(optimized),
    }) {
        eprintln!("Failed to set clipboard: {}", e);
        return;
    }

    eprintln!("✓ Image downscaled and copied to clipboard");
}

fn watch_clipboard(quality: u8) {
    eprintln!("👀 Watching clipboard for images... (Press Ctrl+C to stop)");

    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to access clipboard: {}", e);
            return;
        }
    };

    let mut last_hash: Option<u64> = None;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Ok(image) = clipboard.get_image() {
            // Simple hash to detect changes
            let hash = simple_hash(&image.bytes);

            if Some(hash) != last_hash {
                last_hash = Some(hash);
                eprintln!("\n📋 New image detected: {}×{}", image.width, image.height);

                if let Some(rgba_image) = ImageBuffer::from_raw(
                    image.width as u32,
                    image.height as u32,
                    image.bytes.into_owned(),
                ) {
                    let optimized = optimize_image(&rgba_image, quality);

                    if clipboard
                        .set_image(ImageData {
                            width: rgba_image.width() as usize,
                            height: rgba_image.height() as usize,
                            bytes: Cow::Owned(optimized),
                        })
                        .is_ok()
                    {
                        eprintln!("✓ Auto-optimized");
                    }
                }
            }
        }
    }
}

fn save_clipboard(output: PathBuf, quality: u8) {
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(e) => {
            eprintln!("Failed to access clipboard: {}", e);
            return;
        }
    };

    let image = match clipboard.get_image() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("No image in clipboard: {}", e);
            return;
        }
    };

    let rgba_image: RgbaImage = match ImageBuffer::from_raw(
        image.width as u32,
        image.height as u32,
        image.bytes.into_owned(),
    ) {
        Some(img) => img,
        None => {
            eprintln!("Failed to parse image data");
            return;
        }
    };

    // Determine format from extension
    let format = match output.extension().and_then(|s| s.to_str()) {
        Some("png") => ImageFormat::Png,
        Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
        Some("gif") => ImageFormat::Gif,
        Some("webp") => ImageFormat::WebP,
        _ => {
            eprintln!("Unsupported format. Using PNG.");
            ImageFormat::Png
        }
    };

    if let Err(e) = rgba_image.save_with_format(&output, format) {
        eprintln!("Failed to save image: {}", e);
        return;
    }

    eprintln!("✓ Saved to {}", output.display());
}

fn optimize_image(image: &RgbaImage, quality: u8) -> Vec<u8> {
    use imagequant::Attributes;

    let width = image.width() as usize;
    let height = image.height() as usize;

    let mut attr = Attributes::new();
    let min_quality = quality.saturating_sub(10);
    let max_quality = quality;

    if attr.set_quality(min_quality, max_quality).is_err() {
        eprintln!("Warning: Invalid quality settings, using defaults");
        let _ = attr.set_quality(75, 85);
    }
    let _ = attr.set_speed(5);

    let pixels: Box<[imagequant::RGBA]> = image
        .pixels()
        .map(|p| imagequant::RGBA::new(p[0], p[1], p[2], p[3]))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    match attr.new_image(pixels, width, height, 0.0) {
        Ok(mut img) => match attr.quantize(&mut img) {
            Ok(mut result) => match result.remapped(&mut img) {
                Ok((palette, indices)) => indices
                    .iter()
                    .flat_map(|&idx| {
                        let color = palette[idx as usize];
                        [color.r, color.g, color.b, color.a]
                    })
                    .collect(),
                Err(_) => image.as_raw().clone(),
            },
            Err(_) => image.as_raw().clone(),
        },
        Err(_) => image.as_raw().clone(),
    }
}

fn simple_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
