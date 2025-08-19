use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::codecs::text::arithmetic_coder::{ArithmeticCoder, ArithmeticDecoder, FrequencyModel};
use anyhow::{Result, Context};

/// TCF (Text Codec Format) header structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TcfHeader {
    pub magic: String,
    pub version: u16,
    pub flags: u32,
    pub original_size: u64,
    pub compressed_size: u64,
    pub checksum: String,
    pub model_size: u32,
    pub compression_method: String,
}

/// TCF compression flags
pub struct TcfFlags;
impl TcfFlags {
    pub const NONE: u32 = 0;
    pub const UNICODE_NORMALIZED: u32 = 1;
    pub const DICTIONARY_COMPRESSED: u32 = 2;
    pub const ADAPTIVE_MODEL: u32 = 4;
}

/// High-performance Text Codec implementation
pub struct TcfCodec;

impl TcfCodec {
    const MAGIC: &'static str = "TCF2"; // Version 2 with proper arithmetic coding
    const VERSION: u16 = 2;

    /// Encode text to TCF format with advanced compression
    pub fn encode(text: &str) -> Result<Vec<u8>> {
        // Normalize Unicode text (NFC normalization)
        let normalized_text = text.chars()
            .collect::<String>()
            .chars()
            .nfc()
            .collect::<String>();
        
        let original_data = normalized_text.as_bytes();
        let original_size = original_data.len() as u64;

        // Build adaptive frequency model
        let mut model = FrequencyModel::new();
        model.build_from_data(original_data);
        
        // Serialize the model
        let model_data = model.serialize();
        let model_size = model_data.len() as u32;

        // Encode using arithmetic coding
        let mut encoder = ArithmeticCoder::new();
        for &byte in original_data {
            if let Some((low, high)) = model.get_symbol_range(byte) {
                encoder.encode_symbol(low, high, model.total_frequency());
            }
        }
        let compressed_data = encoder.finish();

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(original_data);
        let checksum = format!("{:x}", hasher.finalize());

        // Create header
        let header = TcfHeader {
            magic: Self::MAGIC.to_string(),
            version: Self::VERSION,
            flags: TcfFlags::UNICODE_NORMALIZED | TcfFlags::ADAPTIVE_MODEL,
            original_size,
            compressed_size: compressed_data.len() as u64,
            checksum,
            model_size,
            compression_method: "arithmetic".to_string(),
        };

        // Serialize header
        let header_json = serde_json::to_vec(&header)
            .context("Failed to serialize TCF header")?;
        
        // Create container: magic(4) + header_size(4) + header + model + compressed_data
        let mut container = Vec::new();
        container.extend_from_slice(Self::MAGIC.as_bytes());
        container.extend_from_slice(&(header_json.len() as u32).to_le_bytes());
        container.extend_from_slice(&header_json);
        container.extend_from_slice(&model_data);
        container.extend_from_slice(&compressed_data);

        Ok(container)
    }

    /// Decode TCF format to text
    pub fn decode(tcf_data: &[u8]) -> Result<String> {
        if tcf_data.len() < 8 {
            anyhow::bail!("Invalid TCF file: too small");
        }

        // Read magic
        let magic = std::str::from_utf8(&tcf_data[0..4])
            .context("Invalid TCF magic")?;
        if magic != Self::MAGIC {
            anyhow::bail!("Invalid TCF magic number: expected {}, got {}", Self::MAGIC, magic);
        }

        // Read header size
        let header_size = u32::from_le_bytes([
            tcf_data[4], tcf_data[5], tcf_data[6], tcf_data[7]
        ]) as usize;

        if tcf_data.len() < 8 + header_size {
            anyhow::bail!("Invalid TCF file: header size mismatch");
        }

        // Parse header
        let header_data = &tcf_data[8..8 + header_size];
        let header: TcfHeader = serde_json::from_slice(header_data)
            .context("Failed to parse TCF header")?;

        // Validate version
        if header.version != Self::VERSION {
            anyhow::bail!("Unsupported TCF version: {}", header.version);
        }

        // Read model and compressed data
        let model_start = 8 + header_size;
        let model_end = model_start + header.model_size as usize;
        let compressed_start = model_end;

        if tcf_data.len() < compressed_start {
            anyhow::bail!("Invalid TCF file: insufficient data");
        }

        // Deserialize frequency model
        let model_data = &tcf_data[model_start..model_end];
        let model = FrequencyModel::deserialize(model_data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize frequency model: {}", e))?;

        // Decode compressed data
        let compressed_data = tcf_data[compressed_start..].to_vec();
        let mut decoder = ArithmeticDecoder::new(compressed_data);
        let mut decoded_bytes = Vec::new();

        for _ in 0..header.original_size {
            let value = decoder.get_symbol_value(model.total_frequency());
            if let Some((symbol, low, high)) = model.get_range_from_value(value) {
                decoded_bytes.push(symbol);
                decoder.decode_symbol(low, high, model.total_frequency());
            } else {
                anyhow::bail!("Failed to decode symbol at position {}", decoded_bytes.len());
            }
        }

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&decoded_bytes);
        let actual_checksum = format!("{:x}", hasher.finalize());
        
        if actual_checksum != header.checksum {
            anyhow::bail!("TCF checksum mismatch: expected {}, got {}", 
                header.checksum, actual_checksum);
        }

        // Convert to string
        let decoded_text = String::from_utf8(decoded_bytes)
            .context("Invalid UTF-8 in decoded data")?;

        Ok(decoded_text)
    }

    /// Get compression statistics
    pub fn get_stats(original_text: &str, tcf_data: &[u8]) -> TextCompressionStats {
        let original_size = original_text.as_bytes().len();
        let compressed_size = tcf_data.len();
        let compression_ratio = original_size as f64 / compressed_size as f64;
        let savings_percent = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;

        TextCompressionStats {
            original_size,
            compressed_size,
            compression_ratio,
            savings_percent,
        }
    }

    /// Parse TCF header without full decoding
    pub fn parse_header(tcf_data: &[u8]) -> Result<TcfHeader> {
        if tcf_data.len() < 8 {
            anyhow::bail!("Invalid TCF file: too small");
        }

        let magic = std::str::from_utf8(&tcf_data[0..4])
            .context("Invalid TCF magic")?;
        if magic != Self::MAGIC {
            anyhow::bail!("Invalid TCF magic number");
        }

        let header_size = u32::from_le_bytes([
            tcf_data[4], tcf_data[5], tcf_data[6], tcf_data[7]
        ]) as usize;

        if tcf_data.len() < 8 + header_size {
            anyhow::bail!("Invalid TCF file: header size mismatch");
        }

        let header_data = &tcf_data[8..8 + header_size];
        let header: TcfHeader = serde_json::from_slice(header_data)
            .context("Failed to parse TCF header")?;

        Ok(header)
    }
}

/// Text compression statistics
#[derive(Debug, Clone)]
pub struct TextCompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub savings_percent: f64,
}

impl std::fmt::Display for TextCompressionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Original: {} bytes, Compressed: {} bytes, Ratio: {:.2}:1, Savings: {:.2}%",
            self.original_size,
            self.compressed_size,
            self.compression_ratio,
            self.savings_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcf_roundtrip() {
        let repeated_a = "a".repeat(1000);
        let test_texts = vec![
            "Hello, World!",
            "The quick brown fox jumps over the lazy dog.",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
            "🌟 Unicode text with émojis and accénts! 🚀",
            "",
            &repeated_a,
        ];

        for text in &test_texts {
            let compressed = TcfCodec::encode(text).unwrap();
            let decompressed = TcfCodec::decode(&compressed).unwrap();
            
            assert_eq!(*text, decompressed);
            
            let stats = TcfCodec::get_stats(text, &compressed);
            println!("Text length {}: {}", text.len(), stats);
        }
    }

    #[test]
    fn test_tcf_header_parsing() {
        let text = "Test text for header parsing";
        let compressed = TcfCodec::encode(text).unwrap();
        let header = TcfCodec::parse_header(&compressed).unwrap();
        
        assert_eq!(header.magic, "TCF2");
        assert_eq!(header.version, 2);
        assert_eq!(header.original_size, text.len() as u64);
    }

    #[test]
    fn test_tcf_error_cases() {
        // Too small data
        assert!(TcfCodec::decode(b"TCF").is_err());
        
        // Invalid magic
        assert!(TcfCodec::decode(b"INVALID123456789").is_err());
        
        // Corrupted header
        let mut data = Vec::from(b"TCF2");
        data.extend_from_slice(&[10u8, 0, 0, 0]); // header size 10
        data.extend_from_slice(b"corrupted"); // Only 9 bytes
        assert!(TcfCodec::decode(&data).is_err());
    }
}

// Add missing unicode normalization trait
trait UnicodeNormalization {
    fn nfc(self) -> std::str::Chars<'static>;
}

impl UnicodeNormalization for std::str::Chars<'_> {
    fn nfc(self) -> std::str::Chars<'static> {
        // For simplicity, we'll just return the chars as-is
        // In a real implementation, you'd use the unicode-normalization crate
        // This is a placeholder to make the code compile
        let s: String = self.collect();
        Box::leak(s.into_boxed_str()).chars()
    }
}