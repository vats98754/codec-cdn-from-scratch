use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::codecs::text::simple_coder::{SimpleArithmeticCoder, SimpleFrequencyModel};
use anyhow::{Result, Context};

/// Working TCF implementation using simplified compression
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleTcfHeader {
    pub magic: String,
    pub version: u16,
    pub original_size: u64,
    pub compressed_size: u64,
    pub checksum: String,
    pub model_size: u32,
}

pub struct SimpleTcfCodec;

impl SimpleTcfCodec {
    const MAGIC: &'static str = "TCF2";
    const VERSION: u16 = 2;

    pub fn encode(text: &str) -> Result<Vec<u8>> {
        let original_data = text.as_bytes();
        let original_size = original_data.len() as u64;

        // Build frequency model
        let mut model = SimpleFrequencyModel::new();
        model.build_from_data(original_data);
        
        let model_data = model.serialize();
        let model_size = model_data.len() as u32;

        // Compress using simplified arithmetic coding
        let compressed_data = SimpleArithmeticCoder::encode(original_data, &model);

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(original_data);
        let checksum = format!("{:x}", hasher.finalize());

        // Create header
        let header = SimpleTcfHeader {
            magic: Self::MAGIC.to_string(),
            version: Self::VERSION,
            original_size,
            compressed_size: compressed_data.len() as u64,
            checksum,
            model_size,
        };

        // Serialize header
        let header_json = serde_json::to_vec(&header)
            .context("Failed to serialize TCF header")?;
        
        // Create container
        let mut container = Vec::new();
        container.extend_from_slice(Self::MAGIC.as_bytes());
        container.extend_from_slice(&(header_json.len() as u32).to_le_bytes());
        container.extend_from_slice(&header_json);
        container.extend_from_slice(&model_data);
        container.extend_from_slice(&compressed_data);

        Ok(container)
    }

    pub fn decode(tcf_data: &[u8]) -> Result<String> {
        if tcf_data.len() < 8 {
            anyhow::bail!("Invalid TCF file: too small");
        }

        // Read magic
        let magic = std::str::from_utf8(&tcf_data[0..4])
            .context("Invalid TCF magic")?;
        if magic != Self::MAGIC {
            anyhow::bail!("Invalid TCF magic number");
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
        let header: SimpleTcfHeader = serde_json::from_slice(header_data)
            .context("Failed to parse TCF header")?;

        // Read model and compressed data
        let model_start = 8 + header_size;
        let model_end = model_start + header.model_size as usize;
        let compressed_start = model_end;

        if tcf_data.len() < compressed_start {
            anyhow::bail!("Invalid TCF file: insufficient data");
        }

        // Deserialize frequency model
        let model_data = &tcf_data[model_start..model_end];
        let model = SimpleFrequencyModel::deserialize(model_data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize frequency model: {}", e))?;

        // Decode compressed data
        let compressed_data = &tcf_data[compressed_start..];
        let decoded_bytes = SimpleArithmeticCoder::decode(compressed_data, &model)
            .map_err(|e| anyhow::anyhow!("Failed to decode compressed data: {}", e))?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&decoded_bytes);
        let actual_checksum = format!("{:x}", hasher.finalize());
        
        if actual_checksum != header.checksum {
            anyhow::bail!("TCF checksum mismatch");
        }

        let decoded_text = String::from_utf8(decoded_bytes)
            .context("Invalid UTF-8 in decoded data")?;

        Ok(decoded_text)
    }

    pub fn get_stats(original_text: &str, tcf_data: &[u8]) -> TextCompressionStats {
        let original_size = original_text.as_bytes().len();
        let compressed_size = tcf_data.len();
        let compression_ratio = if compressed_size > 0 {
            original_size as f64 / compressed_size as f64
        } else {
            0.0
        };
        let savings_percent = if original_size > 0 {
            if compressed_size >= original_size {
                -((compressed_size - original_size) as f64 / original_size as f64) * 100.0
            } else {
                ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
            }
        } else {
            0.0
        };

        TextCompressionStats {
            original_size,
            compressed_size,
            compression_ratio,
            savings_percent,
        }
    }

    pub fn parse_header(tcf_data: &[u8]) -> Result<SimpleTcfHeader> {
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
        let header: SimpleTcfHeader = serde_json::from_slice(header_data)
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