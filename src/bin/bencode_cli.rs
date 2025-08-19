use clap::{Arg, ArgMatches, Command};
use std::fs;
use std::collections::HashMap;
use std::time::Instant;
use base64::{Engine as _, engine::general_purpose};

use codec_cdn_rust::codecs::bencode::{BencodeCodec, BencodeValue};

fn main() -> anyhow::Result<()> {
    let matches = Command::new("bencode-cli")
        .version("1.0.0")
        .author("vats98754")
        .about("Bencode encoder/decoder - BitTorrent serialization format")
        .subcommand(
            Command::new("encode")
                .about("Encode data to bencode format")
                .arg(
                    Arg::new("input")
                        .help("Input file (JSON format)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .help("Output file (.bencode)")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("pretty")
                        .long("pretty")
                        .help("Pretty print the bencode structure")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("decode")
                .about("Decode bencode data")
                .arg(
                    Arg::new("input")
                        .help("Input file (.bencode)")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .help("Output file (JSON format)")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .help("Output format")
                        .value_parser(["json", "text"])
                        .default_value("json"),
                ),
        )
        .subcommand(
            Command::new("info")
                .about("Show information about bencode file")
                .arg(
                    Arg::new("input")
                        .help("Input file (.bencode)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("create-torrent")
                .about("Create a torrent file structure")
                .arg(
                    Arg::new("name")
                        .help("Torrent name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("output")
                        .help("Output torrent file")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::new("announce")
                        .long("announce")
                        .help("Tracker announce URL")
                        .default_value("http://tracker.example.com/announce"),
                )
                .arg(
                    Arg::new("piece-length")
                        .long("piece-length")
                        .help("Piece length in bytes")
                        .value_parser(clap::value_parser!(u64))
                        .default_value("32768"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("encode", sub_matches)) => encode_command(sub_matches),
        Some(("decode", sub_matches)) => decode_command(sub_matches),
        Some(("info", sub_matches)) => info_command(sub_matches),
        Some(("create-torrent", sub_matches)) => create_torrent_command(sub_matches),
        _ => {
            eprintln!("No subcommand specified. Use --help for usage information.");
            Ok(())
        }
    }
}

fn encode_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let pretty = matches.get_flag("pretty");

    let start_time = Instant::now();

    // Read input JSON file
    let input_data = fs::read_to_string(input_path)?;
    let json_value: serde_json::Value = serde_json::from_str(&input_data)?;
    
    // Convert JSON to BencodeValue
    let bencode_value = json_to_bencode(&json_value)?;
    
    // Create file format with metadata
    let mut metadata = HashMap::new();
    metadata.insert(b"source".to_vec(), BencodeValue::string("bencode-cli"));
    metadata.insert(b"input_file".to_vec(), BencodeValue::string(input_path));
    let metadata_value = BencodeValue::dictionary(metadata);
    
    // Encode to bencode format
    let encoded_data = BencodeCodec::create_file_format(&bencode_value, Some(&metadata_value))?;
    
    // Write output
    fs::write(output_path, &encoded_data)?;
    
    let encode_time = start_time.elapsed();
    
    if pretty {
        println!("Bencode structure:");
        println!("{}", bencode_value);
        println!();
    }
    
    // Show statistics
    let stats = BencodeCodec::get_stats(input_data.as_bytes(), &encoded_data);
    println!("✅ Encoding complete!");
    println!("📄 Input: {}", input_path);
    println!("📦 Output: {}", output_path);
    println!("📊 Original size: {} bytes", stats["original_size"]);
    println!("📊 Encoded size: {} bytes", stats["encoded_size"]);
    println!("📊 Compression ratio: {:.2}", stats["compression_ratio"]);
    println!("⏱️  Encoding time: {:.3}ms", encode_time.as_secs_f64() * 1000.0);
    
    Ok(())
}

fn decode_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let format = matches.get_one::<String>("format").unwrap();

    let start_time = Instant::now();

    // Read bencode file
    let encoded_data = fs::read(input_path)?;
    
    // Parse file format
    let (content, metadata) = BencodeCodec::parse_file_format(&encoded_data)?;
    
    // Convert to output format
    match format.as_str() {
        "json" => {
            let json_value = bencode_to_json(&content)?;
            let output_data = serde_json::to_string_pretty(&json_value)?;
            fs::write(output_path, output_data)?;
        }
        "text" => {
            let output_data = format!("{}", content);
            fs::write(output_path, output_data)?;
        }
        _ => unreachable!(),
    }
    
    let decode_time = start_time.elapsed();
    
    println!("✅ Decoding complete!");
    println!("📦 Input: {}", input_path);
    println!("📄 Output: {}", output_path);
    println!("📊 Decoded size: {} bytes", fs::metadata(output_path)?.len());
    println!("⏱️  Decoding time: {:.3}ms", decode_time.as_secs_f64() * 1000.0);
    
    if let Some(meta) = metadata {
        println!("\n📋 Metadata:");
        println!("{}", meta);
    }
    
    Ok(())
}

fn info_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let input_path = matches.get_one::<String>("input").unwrap();

    let encoded_data = fs::read(input_path)?;
    let (content, metadata) = BencodeCodec::parse_file_format(&encoded_data)?;
    
    println!("📁 File: {}", input_path);
    println!("📦 Size: {} bytes", encoded_data.len());
    println!("🏷️  Format: Bencode");
    
    // Analyze content structure
    match &content {
        BencodeValue::Integer(_) => println!("📊 Content type: Integer"),
        BencodeValue::ByteString(s) => {
            println!("📊 Content type: Byte string ({} bytes)", s.len());
            if let Ok(text) = String::from_utf8(s.clone()) {
                println!("📄 Text preview: {}", 
                    if text.len() > 100 { 
                        format!("{}...", &text[..100]) 
                    } else { 
                        text 
                    }
                );
            }
        }
        BencodeValue::List(l) => {
            println!("📊 Content type: List ({} items)", l.len());
            for (i, item) in l.iter().enumerate().take(5) {
                println!("  [{}]: {}", i, type_name(item));
            }
            if l.len() > 5 {
                println!("  ... and {} more items", l.len() - 5);
            }
        }
        BencodeValue::Dictionary(d) => {
            println!("📊 Content type: Dictionary ({} keys)", d.keys().len());
            let mut sorted_keys: Vec<_> = d.keys().collect();
            sorted_keys.sort();
            for key in sorted_keys.iter().take(10) {
                if let Ok(key_str) = String::from_utf8((*key).clone()) {
                    println!("  \"{}\": {}", key_str, type_name(d.get(*key).unwrap()));
                }
            }
            if d.len() > 10 {
                println!("  ... and {} more keys", d.len() - 10);
            }
        }
    }
    
    if let Some(meta) = metadata {
        println!("\n📋 Metadata:");
        if let Some(dict) = meta.as_dictionary() {
            for (key, value) in dict.iter() {
                if let Ok(key_str) = String::from_utf8(key.clone()) {
                    println!("  {}: {}", key_str, value);
                }
            }
        }
    }
    
    Ok(())
}

fn create_torrent_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let name = matches.get_one::<String>("name").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let announce = matches.get_one::<String>("announce").unwrap();
    let piece_length = *matches.get_one::<u64>("piece-length").unwrap();

    // Create torrent info dictionary
    let mut info = HashMap::new();
    info.insert(b"name".to_vec(), BencodeValue::string(name));
    info.insert(b"piece length".to_vec(), BencodeValue::integer(piece_length as i64));
    info.insert(b"length".to_vec(), BencodeValue::integer(0)); // Placeholder
    info.insert(b"pieces".to_vec(), BencodeValue::byte_string(Vec::new())); // Placeholder
    
    // Create main torrent dictionary
    let mut torrent = HashMap::new();
    torrent.insert(b"announce".to_vec(), BencodeValue::string(announce));
    torrent.insert(b"info".to_vec(), BencodeValue::dictionary(info));
    torrent.insert(b"creation date".to_vec(), BencodeValue::integer(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    ));
    torrent.insert(b"created by".to_vec(), BencodeValue::string("bencode-cli 1.0.0"));
    
    let torrent_value = BencodeValue::dictionary(torrent);
    let encoded_data = BencodeCodec::encode(&torrent_value)?;
    
    fs::write(output_path, encoded_data)?;
    
    println!("✅ Torrent file created!");
    println!("📁 Name: {}", name);
    println!("📦 Output: {}", output_path);
    println!("🌐 Announce: {}", announce);
    println!("📊 Piece length: {} bytes", piece_length);
    
    Ok(())
}

fn json_to_bencode(json: &serde_json::Value) -> anyhow::Result<BencodeValue> {
    match json {
        serde_json::Value::Null => Ok(BencodeValue::string("")),
        serde_json::Value::Bool(b) => Ok(BencodeValue::integer(if *b { 1 } else { 0 })),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(BencodeValue::integer(i))
            } else {
                Ok(BencodeValue::string(&n.to_string()))
            }
        }
        serde_json::Value::String(s) => Ok(BencodeValue::string(s)),
        serde_json::Value::Array(arr) => {
            let mut bencode_list = Vec::new();
            for item in arr {
                bencode_list.push(json_to_bencode(item)?);
            }
            Ok(BencodeValue::list(bencode_list))
        }
        serde_json::Value::Object(obj) => {
            let mut bencode_dict = HashMap::new();
            for (key, value) in obj {
                bencode_dict.insert(key.as_bytes().to_vec(), json_to_bencode(value)?);
            }
            Ok(BencodeValue::dictionary(bencode_dict))
        }
    }
}

fn bencode_to_json(bencode: &BencodeValue) -> anyhow::Result<serde_json::Value> {
    match bencode {
        BencodeValue::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(*i))),
        BencodeValue::ByteString(s) => {
            if let Ok(text) = String::from_utf8(s.clone()) {
                Ok(serde_json::Value::String(text))
            } else {
                // For binary data, encode as base64
                Ok(serde_json::Value::String(general_purpose::STANDARD.encode(s)))
            }
        }
        BencodeValue::List(l) => {
            let mut json_array = Vec::new();
            for item in l {
                json_array.push(bencode_to_json(item)?);
            }
            Ok(serde_json::Value::Array(json_array))
        }
        BencodeValue::Dictionary(d) => {
            let mut json_object = serde_json::Map::new();
            for (key, value) in d {
                let key_str = String::from_utf8(key.clone())
                    .unwrap_or_else(|_| general_purpose::STANDARD.encode(key));
                json_object.insert(key_str, bencode_to_json(value)?);
            }
            Ok(serde_json::Value::Object(json_object))
        }
    }
}

fn type_name(value: &BencodeValue) -> &'static str {
    match value {
        BencodeValue::Integer(_) => "integer",
        BencodeValue::ByteString(_) => "byte string",
        BencodeValue::List(_) => "list",
        BencodeValue::Dictionary(_) => "dictionary",
    }
}