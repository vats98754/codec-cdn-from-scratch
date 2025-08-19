use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use anyhow::{Result, Context};
use rayon::prelude::*;

use crate::codecs::image::{
    dct_transform::{Dct8x8, ColorSpace},
    quantization::Quantization,
};

/// ICF (Image Codec Format) header structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IcfHeader {
    pub magic: String,
    pub version: u16,
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub color_space: String,
    pub quality: u8,
    pub compression_method: String,
    pub block_size: u8,
    pub quantization_tables: Vec<Vec<Vec<f64>>>, // [channel][row][col]
    pub original_size: u64,
    pub compressed_size: u64,
    pub checksum: String,
}

/// Compressed block data
#[derive(Serialize, Deserialize, Clone)]
pub struct CompressedBlock {
    pub x: u16,
    pub y: u16,
    pub channel: u8,
    pub dc_coefficient: i16,
    pub ac_coefficients: Vec<(u8, i16)>, // Run-length encoded AC coefficients
}

/// High-performance Image Codec implementation
pub struct IcfCodec {
    dct: Dct8x8,
}

impl IcfCodec {
    const MAGIC: &'static str = "ICF2"; // Version 2 with proper DCT
    const VERSION: u16 = 2;
    const BLOCK_SIZE: usize = 8;

    pub fn new() -> Self {
        Self {
            dct: Dct8x8::new(),
        }
    }

    /// Encode image to ICF format with advanced compression
    pub fn encode(&self, image_path: &str, quality: u8) -> Result<Vec<u8>> {
        // Load image
        let img = image::open(image_path)
            .context("Failed to load image")?;
        
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        
        // Convert to YCoCg color space for better compression
        let ycocg_data = self.rgb_to_ycocg_blocks(&rgb_img);
        
        // Create quantization tables for each channel
        let quantization_tables = vec![
            Quantization::create_quantization_table(quality, true),  // Y channel
            Quantization::create_quantization_table(quality, false), // Co channel
            Quantization::create_quantization_table(quality, false), // Cg channel
        ];

        // Compress each channel in parallel
        let compressed_blocks: Vec<CompressedBlock> = (0..3)
            .into_par_iter()
            .flat_map(|channel| {
                self.compress_channel_blocks(
                    &ycocg_data[channel],
                    width,
                    height,
                    channel as u8,
                    &quantization_tables[channel],
                )
            })
            .collect();

        // Calculate checksum of original image data
        let mut hasher = Sha256::new();
        hasher.update(rgb_img.as_raw());
        let checksum = format!("{:x}", hasher.finalize());

        // Create header
        let header = IcfHeader {
            magic: Self::MAGIC.to_string(),
            version: Self::VERSION,
            width,
            height,
            channels: 3,
            color_space: "YCoCg".to_string(),
            quality,
            compression_method: "DCT+RLE".to_string(),
            block_size: Self::BLOCK_SIZE as u8,
            quantization_tables: quantization_tables.into_iter()
                .map(|table| table.iter().map(|row| row.to_vec()).collect())
                .collect(),
            original_size: rgb_img.as_raw().len() as u64,
            compressed_size: 0, // Will be updated
            checksum,
        };

        // Serialize compressed blocks
        let compressed_data = self.serialize_blocks(&compressed_blocks)?;
        
        // Create final container
        self.create_container(header, compressed_data)
    }

    /// Decode ICF format to image
    pub fn decode(&self, icf_data: &[u8]) -> Result<DynamicImage> {
        let (header, compressed_data) = self.parse_container(icf_data)?;
        
        // Validate header
        if header.magic != Self::MAGIC {
            anyhow::bail!("Invalid ICF magic number: expected {}, got {}", 
                Self::MAGIC, header.magic);
        }

        if header.version != Self::VERSION {
            anyhow::bail!("Unsupported ICF version: {}", header.version);
        }

        // Deserialize compressed blocks
        let compressed_blocks = self.deserialize_blocks(&compressed_data)?;

        // Reconstruct quantization tables
        let quantization_tables: Vec<[[f64; 8]; 8]> = header.quantization_tables
            .into_iter()
            .map(|table| {
                let mut array = [[0.0; 8]; 8];
                for (i, row) in table.into_iter().enumerate() {
                    for (j, val) in row.into_iter().enumerate() {
                        if i < 8 && j < 8 {
                            array[i][j] = val;
                        }
                    }
                }
                array
            })
            .collect();

        // Decompress blocks back to YCoCg data
        let ycocg_blocks = self.decompress_blocks(
            &compressed_blocks,
            header.width,
            header.height,
            &quantization_tables,
        )?;

        // Convert YCoCg back to RGB
        let rgb_img = self.ycocg_blocks_to_rgb(&ycocg_blocks, header.width, header.height);

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(rgb_img.as_raw());
        let actual_checksum = format!("{:x}", hasher.finalize());
        
        if actual_checksum != header.checksum {
            println!("Warning: ICF checksum mismatch (lossy compression expected)");
        }

        Ok(DynamicImage::ImageRgb8(rgb_img))
    }

    /// Convert RGB image to YCoCg blocks
    fn rgb_to_ycocg_blocks(&self, rgb_img: &RgbImage) -> Vec<Vec<Vec<[[f64; 8]; 8]>>> {
        let (width, height) = rgb_img.dimensions();
        let blocks_x = ((width + 7) / 8) as usize;
        let blocks_y = ((height + 7) / 8) as usize;

        let mut channels = vec![vec![vec![[[0.0; 8]; 8]; blocks_x]; blocks_y]; 3];

        for block_y in 0..blocks_y {
            for block_x in 0..blocks_x {
                // Extract 8x8 block from each channel
                let mut rgb_block = [[[0.0; 3]; 8]; 8];
                
                for y in 0..8 {
                    for x in 0..8 {
                        let img_x = (block_x * 8 + x).min(width as usize - 1);
                        let img_y = (block_y * 8 + y).min(height as usize - 1);
                        
                        let pixel = rgb_img.get_pixel(img_x as u32, img_y as u32);
                        rgb_block[y][x][0] = pixel[0] as f64;
                        rgb_block[y][x][1] = pixel[1] as f64;
                        rgb_block[y][x][2] = pixel[2] as f64;
                    }
                }

                // Convert RGB to YCoCg for this block
                for y in 0..8 {
                    for x in 0..8 {
                        let (r, g, b) = (
                            rgb_block[y][x][0] / 255.0,
                            rgb_block[y][x][1] / 255.0,
                            rgb_block[y][x][2] / 255.0,
                        );
                        
                        let (yval, co, cg) = ColorSpace::rgb_to_ycocg(r, g, b);
                        
                        // Center around 0 for DCT
                        channels[0][block_y][block_x][y][x] = (yval * 255.0) - 128.0;
                        channels[1][block_y][block_x][y][x] = co * 255.0;
                        channels[2][block_y][block_x][y][x] = cg * 255.0;
                    }
                }
            }
        }

        channels
    }

    /// Compress blocks for a single channel
    fn compress_channel_blocks(
        &self,
        channel_blocks: &[Vec<[[f64; 8]; 8]>],
        width: u32,
        height: u32,
        channel: u8,
        quantization_table: &[[f64; 8]; 8],
    ) -> Vec<CompressedBlock> {
        let blocks_x = ((width + 7) / 8) as usize;
        let blocks_y = ((height + 7) / 8) as usize;
        let mut compressed_blocks = Vec::new();
        let mut prev_dc = 0i16; // For DC coefficient differential encoding

        for block_y in 0..blocks_y {
            for block_x in 0..blocks_x {
                let block = &channel_blocks[block_y][block_x];
                
                // Apply DCT transform
                let dct_block = self.dct.forward_8x8(block);
                
                // Quantize coefficients
                let quantized_block = Quantization::quantize_block(&dct_block, quantization_table);
                
                // Extract DC coefficient (differential encoding)
                let dc_coefficient = quantized_block[0][0] - prev_dc;
                prev_dc = quantized_block[0][0];
                
                // Convert to zigzag order and skip DC coefficient
                let mut zigzag = Quantization::block_to_zigzag(&quantized_block);
                zigzag.remove(0); // Remove DC coefficient (already stored separately)
                
                // Run-length encode AC coefficients
                let ac_coefficients = Quantization::run_length_encode(&zigzag);
                
                compressed_blocks.push(CompressedBlock {
                    x: block_x as u16,
                    y: block_y as u16,
                    channel,
                    dc_coefficient,
                    ac_coefficients,
                });
            }
        }

        compressed_blocks
    }

    /// Decompress blocks back to spatial domain
    fn decompress_blocks(
        &self,
        compressed_blocks: &[CompressedBlock],
        width: u32,
        height: u32,
        quantization_tables: &[[[f64; 8]; 8]],
    ) -> Result<Vec<Vec<Vec<[[f64; 8]; 8]>>>> {
        let blocks_x = ((width + 7) / 8) as usize;
        let blocks_y = ((height + 7) / 8) as usize;
        
        let mut channels = vec![vec![vec![[[0.0; 8]; 8]; blocks_x]; blocks_y]; 3];
        let mut prev_dc = [0i16; 3]; // DC prediction for each channel

        // Group blocks by channel for sequential DC decoding
        let mut blocks_by_channel: Vec<Vec<&CompressedBlock>> = vec![Vec::new(); 3];
        for block in compressed_blocks {
            blocks_by_channel[block.channel as usize].push(block);
        }

        // Sort blocks by position for correct DC prediction
        for channel_blocks in &mut blocks_by_channel {
            channel_blocks.sort_by_key(|b| (b.y, b.x));
        }

        // Decompress each channel
        for (channel_idx, channel_blocks) in blocks_by_channel.iter().enumerate() {
            for block in channel_blocks {
                // Reconstruct DC coefficient
                let dc_coefficient = block.dc_coefficient + prev_dc[channel_idx];
                prev_dc[channel_idx] = dc_coefficient;

                // Reconstruct AC coefficients
                let ac_coeffs = Quantization::run_length_decode(&block.ac_coefficients);
                
                // Combine DC and AC coefficients in zigzag order
                let mut zigzag = vec![dc_coefficient];
                zigzag.extend(ac_coeffs);
                zigzag.truncate(64);

                // Convert back to 8x8 block
                let quantized_block = Quantization::zigzag_to_block(&zigzag);

                // Dequantize
                let dequantized_block = Quantization::dequantize_block(
                    &quantized_block,
                    &quantization_tables[channel_idx],
                );

                // Apply inverse DCT
                let spatial_block = self.dct.inverse_8x8(&dequantized_block);

                // Store in channel array
                if (block.y as usize) < blocks_y && (block.x as usize) < blocks_x {
                    channels[channel_idx][block.y as usize][block.x as usize] = spatial_block;
                }
            }
        }

        Ok(channels)
    }

    /// Convert YCoCg blocks back to RGB image
    fn ycocg_blocks_to_rgb(
        &self,
        ycocg_blocks: &[Vec<Vec<[[f64; 8]; 8]>>],
        width: u32,
        height: u32,
    ) -> RgbImage {
        let mut rgb_img = ImageBuffer::new(width, height);
        let blocks_x = ((width + 7) / 8) as usize;
        let blocks_y = ((height + 7) / 8) as usize;

        for block_y in 0..blocks_y {
            for block_x in 0..blocks_x {
                for y in 0..8 {
                    for x in 0..8 {
                        let img_x = block_x * 8 + x;
                        let img_y = block_y * 8 + y;
                        
                        if img_x < width as usize && img_y < height as usize {
                            // Get YCoCg values and denormalize
                            let yval = (ycocg_blocks[0][block_y][block_x][y][x] + 128.0) / 255.0;
                            let co = ycocg_blocks[1][block_y][block_x][y][x] / 255.0;
                            let cg = ycocg_blocks[2][block_y][block_x][y][x] / 255.0;
                            
                            // Convert back to RGB
                            let (r, g, b) = ColorSpace::ycocg_to_rgb(yval, co, cg);
                            
                            // Clamp to valid range
                            let r = (r * 255.0).round().max(0.0).min(255.0) as u8;
                            let g = (g * 255.0).round().max(0.0).min(255.0) as u8;
                            let b = (b * 255.0).round().max(0.0).min(255.0) as u8;
                            
                            rgb_img.put_pixel(img_x as u32, img_y as u32, Rgb([r, g, b]));
                        }
                    }
                }
            }
        }

        rgb_img
    }

    /// Serialize compressed blocks to binary data
    fn serialize_blocks(&self, blocks: &[CompressedBlock]) -> Result<Vec<u8>> {
        serde_json::to_vec(blocks)
            .context("Failed to serialize compressed blocks")
    }

    /// Deserialize compressed blocks from binary data
    fn deserialize_blocks(&self, data: &[u8]) -> Result<Vec<CompressedBlock>> {
        serde_json::from_slice(data)
            .context("Failed to deserialize compressed blocks")
    }

    /// Create ICF container
    fn create_container(&self, mut header: IcfHeader, compressed_data: Vec<u8>) -> Result<Vec<u8>> {
        header.compressed_size = compressed_data.len() as u64;
        
        let header_json = serde_json::to_vec(&header)
            .context("Failed to serialize ICF header")?;
        
        let mut container = Vec::new();
        container.extend_from_slice(Self::MAGIC.as_bytes());
        container.extend_from_slice(&(header_json.len() as u32).to_le_bytes());
        container.extend_from_slice(&header_json);
        container.extend_from_slice(&compressed_data);

        Ok(container)
    }

    /// Parse ICF container
    pub fn parse_container(&self, icf_data: &[u8]) -> Result<(IcfHeader, Vec<u8>)> {
        if icf_data.len() < 8 {
            anyhow::bail!("Invalid ICF file: too small");
        }

        let magic = std::str::from_utf8(&icf_data[0..4])
            .context("Invalid ICF magic")?;
        if magic != Self::MAGIC {
            anyhow::bail!("Invalid ICF magic number");
        }

        let header_size = u32::from_le_bytes([
            icf_data[4], icf_data[5], icf_data[6], icf_data[7]
        ]) as usize;

        if icf_data.len() < 8 + header_size {
            anyhow::bail!("Invalid ICF file: header size mismatch");
        }

        let header_data = &icf_data[8..8 + header_size];
        let header: IcfHeader = serde_json::from_slice(header_data)
            .context("Failed to parse ICF header")?;

        let compressed_data = icf_data[8 + header_size..].to_vec();

        Ok((header, compressed_data))
    }

    /// Get compression statistics
    pub fn get_stats(&self, original_path: &str, icf_data: &[u8]) -> Result<ImageCompressionStats> {
        let original_img = image::open(original_path)?;
        let original_size = original_img.as_bytes().len();
        let compressed_size = icf_data.len();
        let compression_ratio = original_size as f64 / compressed_size as f64;
        let savings_percent = ((original_size - compressed_size) as f64 / original_size as f64) * 100.0;

        Ok(ImageCompressionStats {
            original_size,
            compressed_size,
            compression_ratio,
            savings_percent,
        })
    }
}

/// Image compression statistics
#[derive(Debug, Clone)]
pub struct ImageCompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub savings_percent: f64,
}

impl std::fmt::Display for ImageCompressionStats {
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
    use tempfile::TempDir;
    use image::{ImageBuffer, Rgb};

    #[test]
    fn test_icf_codec_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let test_image_path = temp_dir.path().join("test.png");
        
        // Create a simple test image
        let img = ImageBuffer::from_fn(64, 64, |x, y| {
            let intensity = ((x + y) % 256) as u8;
            Rgb([intensity, intensity / 2, intensity / 4])
        });
        
        img.save(&test_image_path).unwrap();
        
        let codec = IcfCodec::new();
        
        // Test different quality levels
        for quality in [10, 50, 85, 95] {
            let compressed = codec.encode(test_image_path.to_str().unwrap(), quality).unwrap();
            let decompressed = codec.decode(&compressed).unwrap();
            
            // Check dimensions
            assert_eq!(decompressed.width(), 64);
            assert_eq!(decompressed.height(), 64);
            
            let stats = codec.get_stats(test_image_path.to_str().unwrap(), &compressed).unwrap();
            println!("Quality {}: {}", quality, stats);
            
            // Higher quality should generally have larger file sizes
            if quality > 10 {
                // Basic sanity check that compression is working
                assert!(compressed.len() < img.as_raw().len());
            }
        }
    }
}