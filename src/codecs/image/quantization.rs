/// Advanced quantization strategies for image compression
pub struct Quantization;

impl Quantization {
    /// JPEG-style quantization table for luminance (Y channel)
    pub const LUMINANCE_TABLE: [[f64; 8]; 8] = [
        [16.0, 11.0, 10.0, 16.0, 24.0, 40.0, 51.0, 61.0],
        [12.0, 12.0, 14.0, 19.0, 26.0, 58.0, 60.0, 55.0],
        [14.0, 13.0, 16.0, 24.0, 40.0, 57.0, 69.0, 56.0],
        [14.0, 17.0, 22.0, 29.0, 51.0, 87.0, 80.0, 62.0],
        [18.0, 22.0, 37.0, 56.0, 68.0, 109.0, 103.0, 77.0],
        [24.0, 35.0, 55.0, 64.0, 81.0, 104.0, 113.0, 92.0],
        [49.0, 64.0, 78.0, 87.0, 103.0, 121.0, 120.0, 101.0],
        [72.0, 92.0, 95.0, 98.0, 112.0, 100.0, 103.0, 99.0],
    ];

    /// JPEG-style quantization table for chrominance (Co/Cg channels)
    pub const CHROMINANCE_TABLE: [[f64; 8]; 8] = [
        [17.0, 18.0, 24.0, 47.0, 99.0, 99.0, 99.0, 99.0],
        [18.0, 21.0, 26.0, 66.0, 99.0, 99.0, 99.0, 99.0],
        [24.0, 26.0, 56.0, 99.0, 99.0, 99.0, 99.0, 99.0],
        [47.0, 66.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0],
        [99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0],
        [99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0],
        [99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0],
        [99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0, 99.0],
    ];

    /// Create scaled quantization table based on quality (1-100)
    pub fn create_quantization_table(quality: u8, is_luminance: bool) -> [[f64; 8]; 8] {
        let base_table = if is_luminance {
            Self::LUMINANCE_TABLE
        } else {
            Self::CHROMINANCE_TABLE
        };

        let scale_factor = if quality < 50 {
            5000.0 / quality as f64
        } else {
            200.0 - 2.0 * quality as f64
        };

        let mut scaled_table = [[0.0; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let scaled_value = (base_table[i][j] * scale_factor / 100.0).floor();
                scaled_table[i][j] = scaled_value.max(1.0); // Minimum value of 1
            }
        }

        scaled_table
    }

    /// Adaptive quantization based on local image characteristics
    pub fn adaptive_quantization_table(
        dct_block: &[[f64; 8]; 8],
        quality: u8,
        is_luminance: bool,
    ) -> [[f64; 8]; 8] {
        let base_table = Self::create_quantization_table(quality, is_luminance);
        let mut adaptive_table = base_table;

        // Calculate block variance to determine adaptation strength
        let mean = Self::calculate_block_mean(dct_block);
        let variance = Self::calculate_block_variance(dct_block, mean);
        
        // Adaptation factor based on variance (high variance = less quantization)
        let adaptation_factor = (1.0 + variance / 1000.0).min(2.0).max(0.5);

        // Apply adaptation with stronger effect on higher frequencies
        for i in 0..8 {
            for j in 0..8 {
                let frequency_weight = ((i + j) as f64 / 14.0).max(0.5);
                adaptive_table[i][j] = (base_table[i][j] / (adaptation_factor * frequency_weight)).max(1.0);
            }
        }

        adaptive_table
    }

    /// Quantize DCT coefficients
    pub fn quantize_block(dct_block: &[[f64; 8]; 8], quantization_table: &[[f64; 8]; 8]) -> [[i16; 8]; 8] {
        let mut quantized = [[0i16; 8]; 8];
        
        for i in 0..8 {
            for j in 0..8 {
                let quantized_value = (dct_block[i][j] / quantization_table[i][j]).round();
                quantized[i][j] = quantized_value.max(i16::MIN as f64).min(i16::MAX as f64) as i16;
            }
        }
        
        quantized
    }

    /// Dequantize DCT coefficients
    pub fn dequantize_block(quantized_block: &[[i16; 8]; 8], quantization_table: &[[f64; 8]; 8]) -> [[f64; 8]; 8] {
        let mut dequantized = [[0.0; 8]; 8];
        
        for i in 0..8 {
            for j in 0..8 {
                dequantized[i][j] = quantized_block[i][j] as f64 * quantization_table[i][j];
            }
        }
        
        dequantized
    }

    /// Perceptual quantization using human visual system model
    pub fn perceptual_quantization_table(quality: u8, viewing_distance: f64) -> [[f64; 8]; 8] {
        let mut table = [[0.0; 8]; 8];
        
        // CSF (Contrast Sensitivity Function) weights for different frequencies
        let csf_weights = [
            [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3],
            [0.9, 0.85, 0.75, 0.65, 0.55, 0.45, 0.35, 0.25],
            [0.8, 0.75, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2],
            [0.7, 0.65, 0.6, 0.55, 0.45, 0.35, 0.25, 0.15],
            [0.6, 0.55, 0.5, 0.45, 0.4, 0.3, 0.2, 0.1],
            [0.5, 0.45, 0.4, 0.35, 0.3, 0.25, 0.15, 0.1],
            [0.4, 0.35, 0.3, 0.25, 0.2, 0.15, 0.1, 0.05],
            [0.3, 0.25, 0.2, 0.15, 0.1, 0.1, 0.05, 0.05],
        ];

        let base_quantizer = if quality < 50 {
            5000.0 / quality as f64
        } else {
            200.0 - 2.0 * quality as f64
        };

        for i in 0..8 {
            for j in 0..8 {
                // Apply CSF weighting and viewing distance adjustment
                let csf_factor = csf_weights[i][j] * viewing_distance.sqrt();
                table[i][j] = (base_quantizer / (100.0 * csf_factor)).max(1.0);
            }
        }

        table
    }

    /// Calculate mean of DCT block
    fn calculate_block_mean(block: &[[f64; 8]; 8]) -> f64 {
        let mut sum = 0.0;
        for i in 0..8 {
            for j in 0..8 {
                sum += block[i][j];
            }
        }
        sum / 64.0
    }

    /// Calculate variance of DCT block
    fn calculate_block_variance(block: &[[f64; 8]; 8], mean: f64) -> f64 {
        let mut sum_squared_diff = 0.0;
        for i in 0..8 {
            for j in 0..8 {
                let diff = block[i][j] - mean;
                sum_squared_diff += diff * diff;
            }
        }
        sum_squared_diff / 64.0
    }

    /// Zigzag scan order for entropy coding
    pub const ZIGZAG_ORDER: [(usize, usize); 64] = [
        (0, 0), (0, 1), (1, 0), (2, 0), (1, 1), (0, 2), (0, 3), (1, 2),
        (2, 1), (3, 0), (4, 0), (3, 1), (2, 2), (1, 3), (0, 4), (0, 5),
        (1, 4), (2, 3), (3, 2), (4, 1), (5, 0), (6, 0), (5, 1), (4, 2),
        (3, 3), (2, 4), (1, 5), (0, 6), (0, 7), (1, 6), (2, 5), (3, 4),
        (4, 3), (5, 2), (6, 1), (7, 0), (7, 1), (6, 2), (5, 3), (4, 4),
        (3, 5), (2, 6), (1, 7), (2, 7), (3, 6), (4, 5), (5, 4), (6, 3),
        (7, 2), (7, 3), (6, 4), (5, 5), (4, 6), (3, 7), (4, 7), (5, 6),
        (6, 5), (7, 4), (7, 5), (6, 6), (5, 7), (6, 7), (7, 6), (7, 7),
    ];

    /// Convert 8x8 block to zigzag-ordered vector
    pub fn block_to_zigzag(block: &[[i16; 8]; 8]) -> Vec<i16> {
        let mut zigzag = Vec::with_capacity(64);
        for &(i, j) in &Self::ZIGZAG_ORDER {
            zigzag.push(block[i][j]);
        }
        zigzag
    }

    /// Convert zigzag-ordered vector back to 8x8 block
    pub fn zigzag_to_block(zigzag: &[i16]) -> [[i16; 8]; 8] {
        let mut block = [[0i16; 8]; 8];
        for (idx, &(i, j)) in Self::ZIGZAG_ORDER.iter().enumerate() {
            if idx < zigzag.len() {
                block[i][j] = zigzag[idx];
            }
        }
        block
    }

    /// Run-length encoding for quantized coefficients
    pub fn run_length_encode(zigzag: &[i16]) -> Vec<(u8, i16)> {
        let mut rle = Vec::new();
        let mut zero_count = 0u8;

        for &value in zigzag {
            if value == 0 {
                zero_count += 1;
                if zero_count == 255 {
                    rle.push((255, 0));
                    zero_count = 0;
                }
            } else {
                rle.push((zero_count, value));
                zero_count = 0;
            }
        }

        // Add end of block marker if needed
        if zero_count > 0 {
            rle.push((0, 0)); // EOB marker
        }

        rle
    }

    /// Run-length decoding
    pub fn run_length_decode(rle: &[(u8, i16)]) -> Vec<i16> {
        let mut decoded = Vec::new();

        for &(zeros, value) in rle {
            // Add the zeros
            for _ in 0..zeros {
                decoded.push(0);
            }
            
            // Add the value (unless it's EOB)
            if !(zeros == 0 && value == 0) {
                decoded.push(value);
            }
        }

        // Pad to 64 elements if needed
        while decoded.len() < 64 {
            decoded.push(0);
        }

        decoded.truncate(64);
        decoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization_tables() {
        // Test basic quantization table creation
        let luma_table = Quantization::create_quantization_table(85, true);
        let chroma_table = Quantization::create_quantization_table(85, false);
        
        // Tables should have reasonable values
        assert!(luma_table[0][0] > 0.0);
        assert!(chroma_table[0][0] > 0.0);
        
        // High quality should have smaller quantization values
        let high_qual = Quantization::create_quantization_table(95, true);
        let low_qual = Quantization::create_quantization_table(10, true);
        
        assert!(high_qual[0][0] < low_qual[0][0]);
    }

    #[test]
    fn test_quantization_roundtrip() {
        let dct_block = [
            [100.0, 50.0, 25.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [50.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [25.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ];

        let q_table = Quantization::create_quantization_table(85, true);
        let quantized = Quantization::quantize_block(&dct_block, &q_table);
        let dequantized = Quantization::dequantize_block(&quantized, &q_table);

        // The dequantized values should be reasonably close to original
        // (exact match not expected due to quantization loss)
        assert!((dct_block[0][0] - dequantized[0][0]).abs() < 20.0);
    }

    #[test]
    fn test_zigzag_scan() {
        let mut block = [[0i16; 8]; 8];
        
        // Fill block with sequential values
        for i in 0..8 {
            for j in 0..8 {
                block[i][j] = (i * 8 + j) as i16;
            }
        }

        let zigzag = Quantization::block_to_zigzag(&block);
        let reconstructed = Quantization::zigzag_to_block(&zigzag);

        assert_eq!(block, reconstructed);
        assert_eq!(zigzag.len(), 64);
    }

    #[test]
    fn test_run_length_encoding() {
        let input = vec![42, 0, 0, 0, 15, 0, 0, 7, 0, 0, 0, 0, 0];
        let encoded = Quantization::run_length_encode(&input);
        let decoded = Quantization::run_length_decode(&encoded);
        
        // Extend input to 64 elements for comparison
        let mut extended_input = input;
        while extended_input.len() < 64 {
            extended_input.push(0);
        }
        
        assert_eq!(decoded, extended_input);
    }

    #[test]
    fn test_perceptual_quantization() {
        let perceptual_table = Quantization::perceptual_quantization_table(85, 1.0);
        
        // Should have different values based on frequency
        assert!(perceptual_table[0][0] != perceptual_table[7][7]);
        
        // Higher frequency components should have larger quantization values
        assert!(perceptual_table[0][0] < perceptual_table[7][7]);
    }
}