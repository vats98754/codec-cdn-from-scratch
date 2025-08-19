use clap::{Arg, Command};
use codec_cdn_rust::codecs::text::TcfCodec;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("tcf-cli")
        .version("1.0")
        .about("Text Codec Format (TCF) CLI tool with advanced arithmetic coding")
        .subcommand(
            Command::new("encode")
                .about("Encode text file to TCF format")
                .arg(
                    Arg::new("input")
                        .help("Input text file (or '-' for stdin)")
                        .required(true)
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("output")
                        .help("Output TCF file")
                        .required(true)
                        .value_name("FILE")
                )
        )
        .subcommand(
            Command::new("decode")
                .about("Decode TCF file to text")
                .arg(
                    Arg::new("input")
                        .help("Input TCF file")
                        .required(true)
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("output")
                        .help("Output text file (or '-' for stdout)")
                        .required(true)
                        .value_name("FILE")
                )
        )
        .subcommand(
            Command::new("info")
                .about("Show information about TCF file")
                .arg(
                    Arg::new("input")
                        .help("Input TCF file")
                        .required(true)
                        .value_name("FILE")
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("encode", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            
            let text = if input == "-" {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer
            } else {
                fs::read_to_string(input)?
            };
            
            println!("Encoding {} characters...", text.len());
            
            let compressed = TcfCodec::encode(&text)?;
            fs::write(output, &compressed)?;
            
            let stats = TcfCodec::get_stats(&text, &compressed);
            println!("✓ Encoding complete!");
            println!("  Input: {} bytes", text.as_bytes().len());
            println!("  Output: {} bytes", compressed.len());
            println!("  Compression ratio: {:.2}:1", stats.compression_ratio);
            println!("  Space savings: {:.2}%", stats.savings_percent);
        }
        
        Some(("decode", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            
            let compressed = fs::read(input)?;
            println!("Decoding {} bytes...", compressed.len());
            
            let text = TcfCodec::decode(&compressed)?;
            
            if output == "-" {
                print!("{}", text);
            } else {
                fs::write(output, &text)?;
            }
            
            println!("✓ Decoding complete!");
            println!("  Decoded {} characters", text.len());
        }
        
        Some(("info", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            
            let compressed = fs::read(input)?;
            let header = TcfCodec::parse_header(&compressed)?;
            
            println!("TCF File Information:");
            println!("  Magic: {}", header.magic);
            println!("  Version: {}", header.version);
            println!("  Original size: {} bytes", header.original_size);
            println!("  Compressed size: {} bytes", header.compressed_size);
            println!("  Compression method: {}", header.compression_method);
            println!("  Model size: {} bytes", header.model_size);
            println!("  Checksum: {}", header.checksum);
            
            let compression_ratio = header.original_size as f64 / compressed.len() as f64;
            let savings = ((header.original_size - compressed.len() as u64) as f64 / header.original_size as f64) * 100.0;
            println!("  Compression ratio: {:.2}:1", compression_ratio);
            println!("  Space savings: {:.2}%", savings);
        }
        
        _ => {
            eprintln!("No subcommand provided. Use --help for usage information.");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

// Usage examples:
// echo "Hello, World!" | tcf-cli encode - hello.tcf
// tcf-cli decode hello.tcf -
// tcf-cli info hello.tcf