use clap::{Arg, Command};
use codec_cdn_rust::codecs::image::{IcfCodec, ImageCompressionStats};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("icf-cli")
        .version("1.0")
        .about("Image Codec Format (ICF) CLI tool with advanced DCT compression")
        .subcommand(
            Command::new("encode")
                .about("Encode image to ICF format")
                .arg(
                    Arg::new("input")
                        .help("Input image file (JPEG, PNG, WebP, etc.)")
                        .required(true)
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("output")
                        .help("Output ICF file")
                        .required(true)
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("quality")
                        .help("Quality level (1-100, default: 85)")
                        .short('q')
                        .long("quality")
                        .value_name("NUM")
                        .default_value("85")
                )
        )
        .subcommand(
            Command::new("decode")
                .about("Decode ICF file to image")
                .arg(
                    Arg::new("input")
                        .help("Input ICF file")
                        .required(true)
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("output")
                        .help("Output image file")
                        .required(true)
                        .value_name("FILE")
                )
        )
        .subcommand(
            Command::new("info")
                .about("Show information about ICF file")
                .arg(
                    Arg::new("input")
                        .help("Input ICF file")
                        .required(true)
                        .value_name("FILE")
                )
        )
        .subcommand(
            Command::new("compare")
                .about("Compare original and compressed images")
                .arg(
                    Arg::new("original")
                        .help("Original image file")
                        .required(true)
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("icf")
                        .help("ICF compressed file")
                        .required(true)
                        .value_name("FILE")
                )
        )
        .get_matches();

    let codec = IcfCodec::new();

    match matches.subcommand() {
        Some(("encode", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            let quality = sub_matches.get_one::<String>("quality").unwrap()
                .parse::<u8>()
                .map_err(|_| "Quality must be a number between 1 and 100")?;
            
            if quality < 1 || quality > 100 {
                return Err("Quality must be between 1 and 100".into());
            }
            
            println!("Encoding image: {} (quality: {})", input, quality);
            
            let compressed = codec.encode(input, quality)?;
            fs::write(output, &compressed)?;
            
            let stats = codec.get_stats(input, &compressed)?;
            println!("✓ Encoding complete!");
            println!("  Input: {} bytes", stats.original_size);
            println!("  Output: {} bytes", stats.compressed_size);
            println!("  Compression ratio: {:.2}:1", stats.compression_ratio);
            println!("  Space savings: {:.2}%", stats.savings_percent);
        }
        
        Some(("decode", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            
            let compressed = fs::read(input)?;
            println!("Decoding ICF file: {} ({} bytes)", input, compressed.len());
            
            let image = codec.decode(&compressed)?;
            image.save(output)?;
            
            println!("✓ Decoding complete!");
            println!("  Output: {} ({}x{})", output, image.width(), image.height());
        }
        
        Some(("info", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            
            let compressed = fs::read(input)?;
            
            // Parse header to show information
            if let Ok((header, _)) = codec.parse_container(&compressed) {
                println!("ICF File Information:");
                println!("  Magic: {}", header.magic);
                println!("  Version: {}", header.version);
                println!("  Dimensions: {}x{}", header.width, header.height);
                println!("  Channels: {}", header.channels);
                println!("  Color space: {}", header.color_space);
                println!("  Quality: {}", header.quality);
                println!("  Compression method: {}", header.compression_method);
                println!("  Block size: {}x{}", header.block_size, header.block_size);
                println!("  Original size: {} bytes", header.original_size);
                println!("  Compressed size: {} bytes", header.compressed_size);
                println!("  File size: {} bytes", compressed.len());
                println!("  Checksum: {}", header.checksum);
                
                let compression_ratio = header.original_size as f64 / compressed.len() as f64;
                let savings = ((header.original_size - compressed.len() as u64) as f64 / header.original_size as f64) * 100.0;
                println!("  Compression ratio: {:.2}:1", compression_ratio);
                println!("  Space savings: {:.2}%", savings);
            } else {
                return Err("Failed to parse ICF header".into());
            }
        }
        
        Some(("compare", sub_matches)) => {
            let original = sub_matches.get_one::<String>("original").unwrap();
            let icf_file = sub_matches.get_one::<String>("icf").unwrap();
            
            let compressed = fs::read(icf_file)?;
            let stats = codec.get_stats(original, &compressed)?;
            
            println!("Image Compression Comparison:");
            println!("  Original: {}", original);
            println!("  Compressed: {}", icf_file);
            println!("  {}", stats);
            
            // Decode and compare dimensions
            let decoded = codec.decode(&compressed)?;
            let original_img = image::open(original)?;
            
            println!("  Original dimensions: {}x{}", original_img.width(), original_img.height());
            println!("  Decoded dimensions: {}x{}", decoded.width(), decoded.height());
            
            if original_img.width() == decoded.width() && original_img.height() == decoded.height() {
                println!("  ✓ Dimensions match");
            } else {
                println!("  ⚠ Dimension mismatch");
            }
        }
        
        _ => {
            eprintln!("No subcommand provided. Use --help for usage information.");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

// Usage examples:
// icf-cli encode input.jpg output.icf --quality 85
// icf-cli decode output.icf decoded.png
// icf-cli info output.icf
// icf-cli compare input.jpg output.icf