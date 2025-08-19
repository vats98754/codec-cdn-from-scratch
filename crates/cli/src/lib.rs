use clap::{Parser, Subcommand};
use std::fs::{File, read_to_string};
use std::io::{BufWriter, BufReader};
use anyhow::Result;
use codec_tcf::{TcfEncoder, TcfDecoder, ModelParams};

#[derive(Parser)]
#[command(name = "tcf-cli")]
#[command(about = "Text Codec Format (TCF) encoder/decoder")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode text to TCF format
    Encode {
        /// Input text file
        #[arg(short, long)]
        input: String,
        
        /// Output TCF file
        #[arg(short, long)]
        output: String,
        
        /// Maximum context order for modeling
        #[arg(long, default_value = "4")]
        max_order: u8,
        
        /// Use escape sequences
        #[arg(long)]
        use_escape: bool,
    },
    
    /// Decode TCF format to text
    Decode {
        /// Input TCF file
        #[arg(short, long)]
        input: String,
        
        /// Output text file
        #[arg(short, long)]
        output: String,
    },
    
    /// Show information about a TCF file
    Info {
        /// TCF file to inspect
        #[arg(short, long)]
        input: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode { input, output, max_order, use_escape } => {
            encode_file(&input, &output, max_order, use_escape)?;
            println!("Successfully encoded {} to {}", input, output);
        }
        
        Commands::Decode { input, output } => {
            decode_file(&input, &output)?;
            println!("Successfully decoded {} to {}", input, output);
        }
        
        Commands::Info { input } => {
            show_info(&input)?;
        }
    }

    Ok(())
}

fn encode_file(input_path: &str, output_path: &str, max_order: u8, use_escape: bool) -> Result<()> {
    // Read input text
    let text = read_to_string(input_path)?;
    
    // Create model parameters
    let model_params = ModelParams {
        max_order,
        use_escape,
        ..Default::default()
    };
    
    // Create encoder
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut encoder = TcfEncoder::new(writer, model_params);
    
    // Encode
    encoder.encode(&text)?;
    
    Ok(())
}

fn decode_file(input_path: &str, output_path: &str) -> Result<()> {
    // Open input file
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    
    // Create decoder
    let mut decoder = TcfDecoder::new(reader);
    
    // Decode
    let decoded_text = decoder.decode()?;
    
    // Write output
    std::fs::write(output_path, decoded_text)?;
    
    Ok(())
}

fn show_info(input_path: &str) -> Result<()> {
    println!("TCF File Information: {}", input_path);
    println!("This feature is not yet implemented.");
    
    // TODO: Implement file info display
    // - File size
    // - Compression ratio
    // - Model parameters
    // - Chunk information
    
    Ok(())
}