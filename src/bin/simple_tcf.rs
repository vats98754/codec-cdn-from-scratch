use clap::{Arg, Command};
use codec_cdn_rust::codecs::text::SimpleTcfCodec;
use std::fs;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("simple-tcf")
        .version("1.0")
        .about("Simple TCF CLI (working version)")
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
            
            let compressed = SimpleTcfCodec::encode(&text)?;
            fs::write(output, &compressed)?;
            
            let stats = SimpleTcfCodec::get_stats(&text, &compressed);
            println!("✓ Encoding complete!");
            println!("  {}", stats);
        }
        
        Some(("decode", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").unwrap();
            
            let compressed = fs::read(input)?;
            println!("Decoding {} bytes...", compressed.len());
            
            let text = SimpleTcfCodec::decode(&compressed)?;
            
            if output == "-" {
                print!("{}", text);
            } else {
                fs::write(output, &text)?;
            }
            
            println!("✓ Decoding complete!");
            println!("  Decoded {} characters", text.len());
        }
        
        _ => {
            eprintln!("No subcommand provided. Use --help for usage information.");
            std::process::exit(1);
        }
    }
    
    Ok(())
}