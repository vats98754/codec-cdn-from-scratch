use ndarray::Array2;
use std::f64::consts::PI;

/// High-performance 2D DCT implementation using separable transforms
pub struct DctTransform {
    size: usize,
    cosine_table: Vec<Vec<f64>>,
}

impl DctTransform {
    /// Create a new DCT transform for blocks of given size
    pub fn new(size: usize) -> Self {
        let mut cosine_table = vec![vec![0.0; size]; size];
        
        for i in 0..size {
            for j in 0..size {
                cosine_table[i][j] = (PI * (2.0 * j as f64 + 1.0) * i as f64) / (2.0 * size as f64);
            }
        }

        Self {
            size,
            cosine_table: cosine_table.into_iter().map(|row| {
                row.into_iter().map(|x| x.cos()).collect()
            }).collect(),
        }
    }

    /// Apply forward 2D DCT transform
    pub fn forward(&self, input: &Array2<f64>) -> Array2<f64> {
        let mut output = Array2::zeros((self.size, self.size));
        
        for u in 0..self.size {
            for v in 0..self.size {
                let mut sum = 0.0;
                
                for x in 0..self.size {
                    for y in 0..self.size {
                        sum += input[[x, y]] 
                            * self.cosine_table[u][x] 
                            * self.cosine_table[v][y];
                    }
                }
                
                let cu = if u == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
                let cv = if v == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
                
                output[[u, v]] = (2.0 / self.size as f64) * cu * cv * sum;
            }
        }
        
        output
    }

    /// Apply inverse 2D DCT transform
    pub fn inverse(&self, input: &Array2<f64>) -> Array2<f64> {
        let mut output = Array2::zeros((self.size, self.size));
        
        for x in 0..self.size {
            for y in 0..self.size {
                let mut sum = 0.0;
                
                for u in 0..self.size {
                    for v in 0..self.size {
                        let cu = if u == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
                        let cv = if v == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
                        
                        sum += cu * cv * input[[u, v]] 
                            * self.cosine_table[u][x] 
                            * self.cosine_table[v][y];
                    }
                }
                
                output[[x, y]] = (2.0 / self.size as f64) * sum;
            }
        }
        
        output
    }

    /// Apply forward DCT using separable approach (more efficient)
    pub fn forward_separable(&self, input: &Array2<f64>) -> Array2<f64> {
        // First apply 1D DCT to rows
        let mut temp = Array2::zeros((self.size, self.size));
        for row in 0..self.size {
            let row_data = input.row(row);
            let transformed_row = self.forward_1d(&row_data.to_vec());
            for col in 0..self.size {
                temp[[row, col]] = transformed_row[col];
            }
        }
        
        // Then apply 1D DCT to columns
        let mut output = Array2::zeros((self.size, self.size));
        for col in 0..self.size {
            let col_data = temp.column(col);
            let transformed_col = self.forward_1d(&col_data.to_vec());
            for row in 0..self.size {
                output[[row, col]] = transformed_col[row];
            }
        }
        
        output
    }

    /// Apply inverse DCT using separable approach
    pub fn inverse_separable(&self, input: &Array2<f64>) -> Array2<f64> {
        // First apply 1D IDCT to rows
        let mut temp = Array2::zeros((self.size, self.size));
        for row in 0..self.size {
            let row_data = input.row(row);
            let transformed_row = self.inverse_1d(&row_data.to_vec());
            for col in 0..self.size {
                temp[[row, col]] = transformed_row[col];
            }
        }
        
        // Then apply 1D IDCT to columns
        let mut output = Array2::zeros((self.size, self.size));
        for col in 0..self.size {
            let col_data = temp.column(col);
            let transformed_col = self.inverse_1d(&col_data.to_vec());
            for row in 0..self.size {
                output[[row, col]] = transformed_col[row];
            }
        }
        
        output
    }

    /// 1D forward DCT
    fn forward_1d(&self, input: &[f64]) -> Vec<f64> {
        let mut output = vec![0.0; self.size];
        
        for k in 0..self.size {
            let mut sum = 0.0;
            for n in 0..self.size {
                sum += input[n] * self.cosine_table[k][n];
            }
            
            let ck = if k == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
            output[k] = ck * (2.0 / self.size as f64).sqrt() * sum;
        }
        
        output
    }

    /// 1D inverse DCT
    fn inverse_1d(&self, input: &[f64]) -> Vec<f64> {
        let mut output = vec![0.0; self.size];
        
        for n in 0..self.size {
            let mut sum = 0.0;
            for k in 0..self.size {
                let ck = if k == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
                sum += ck * input[k] * self.cosine_table[k][n];
            }
            
            output[n] = (2.0 / self.size as f64).sqrt() * sum;
        }
        
        output
    }
}

/// Optimized 8x8 DCT implementation with precomputed coefficients
pub struct Dct8x8 {
    forward_table: [[f64; 8]; 8],
    inverse_table: [[f64; 8]; 8],
}

impl Dct8x8 {
    pub fn new() -> Self {
        let mut forward_table = [[0.0; 8]; 8];
        let mut inverse_table = [[0.0; 8]; 8];

        // Precompute DCT coefficients for 8x8 blocks
        for i in 0..8 {
            for j in 0..8 {
                let ci = if i == 0 { 1.0 / (2.0_f64).sqrt() } else { 1.0 };
                let angle = PI * (2.0 * j as f64 + 1.0) * i as f64 / 16.0;
                
                forward_table[i][j] = ci * 0.5 * angle.cos();
                inverse_table[j][i] = forward_table[i][j]; // Transpose for inverse
            }
        }

        Self {
            forward_table,
            inverse_table,
        }
    }

    /// Fast 8x8 forward DCT
    pub fn forward_8x8(&self, input: &[[f64; 8]; 8]) -> [[f64; 8]; 8] {
        let mut output = [[0.0; 8]; 8];
        
        // Apply 1D DCT to rows
        let mut temp = [[0.0; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let mut sum = 0.0;
                for k in 0..8 {
                    sum += input[i][k] * self.forward_table[j][k];
                }
                temp[i][j] = sum;
            }
        }
        
        // Apply 1D DCT to columns
        for i in 0..8 {
            for j in 0..8 {
                let mut sum = 0.0;
                for k in 0..8 {
                    sum += temp[k][j] * self.forward_table[i][k];
                }
                output[i][j] = sum;
            }
        }
        
        output
    }

    /// Fast 8x8 inverse DCT
    pub fn inverse_8x8(&self, input: &[[f64; 8]; 8]) -> [[f64; 8]; 8] {
        let mut output = [[0.0; 8]; 8];
        
        // Apply 1D IDCT to rows
        let mut temp = [[0.0; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let mut sum = 0.0;
                for k in 0..8 {
                    sum += input[i][k] * self.inverse_table[j][k];
                }
                temp[i][j] = sum;
            }
        }
        
        // Apply 1D IDCT to columns
        for i in 0..8 {
            for j in 0..8 {
                let mut sum = 0.0;
                for k in 0..8 {
                    sum += temp[k][j] * self.inverse_table[i][k];
                }
                output[i][j] = sum;
            }
        }
        
        output
    }
}

/// Color space conversion utilities
pub struct ColorSpace;

impl ColorSpace {
    /// Convert RGB to YCoCg color space for better compression
    pub fn rgb_to_ycocg(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
        let co = r - b;
        let t = b + co / 2.0;
        let cg = g - t;
        let y = t + cg / 2.0;
        (y, co, cg)
    }

    /// Convert YCoCg back to RGB
    pub fn ycocg_to_rgb(y: f64, co: f64, cg: f64) -> (f64, f64, f64) {
        let t = y - cg / 2.0;
        let g = cg + t;
        let b = t - co / 2.0;
        let r = b + co;
        (r, g, b)
    }

    /// Convert RGB to YUV color space
    pub fn rgb_to_yuv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let u = -0.14713 * r - 0.28886 * g + 0.436 * b;
        let v = 0.615 * r - 0.51499 * g - 0.10001 * b;
        (y, u, v)
    }

    /// Convert YUV back to RGB
    pub fn yuv_to_rgb(y: f64, u: f64, v: f64) -> (f64, f64, f64) {
        let r = y + 1.13983 * v;
        let g = y - 0.39465 * u - 0.58060 * v;
        let b = y + 2.03211 * u;
        (r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    #[test]
    fn test_dct_transform_roundtrip() {
        let dct = DctTransform::new(8);
        let mut input = Array2::zeros((8, 8));
        
        // Create a test pattern
        for i in 0..8 {
            for j in 0..8 {
                input[[i, j]] = (i + j) as f64;
            }
        }
        
        let transformed = dct.forward_separable(&input);
        let reconstructed = dct.inverse_separable(&transformed);
        
        // Check if the reconstruction is close to the original
        for i in 0..8 {
            for j in 0..8 {
                let diff = (input[[i, j]] - reconstructed[[i, j]]).abs();
                assert!(diff < 1e-10, "DCT roundtrip error too large: {}", diff);
            }
        }
    }

    #[test]
    fn test_8x8_dct_roundtrip() {
        let dct = Dct8x8::new();
        let mut input = [[0.0; 8]; 8];
        
        // Create a test pattern
        for i in 0..8 {
            for j in 0..8 {
                input[i][j] = (i * 8 + j) as f64;
            }
        }
        
        let transformed = dct.forward_8x8(&input);
        let reconstructed = dct.inverse_8x8(&transformed);
        
        // Check reconstruction
        for i in 0..8 {
            for j in 0..8 {
                let diff = (input[i][j] - reconstructed[i][j]).abs();
                assert!(diff < 1e-10, "8x8 DCT roundtrip error: {}", diff);
            }
        }
    }

    #[test]
    fn test_color_space_conversions() {
        let rgb = (0.5, 0.7, 0.3);
        
        // Test YCoCg roundtrip
        let (y, co, cg) = ColorSpace::rgb_to_ycocg(rgb.0, rgb.1, rgb.2);
        let rgb_back = ColorSpace::ycocg_to_rgb(y, co, cg);
        
        assert!((rgb.0 - rgb_back.0).abs() < 1e-10);
        assert!((rgb.1 - rgb_back.1).abs() < 1e-10);
        assert!((rgb.2 - rgb_back.2).abs() < 1e-10);
        
        // Test YUV roundtrip
        let (y, u, v) = ColorSpace::rgb_to_yuv(rgb.0, rgb.1, rgb.2);
        let rgb_back = ColorSpace::yuv_to_rgb(y, u, v);
        
        assert!((rgb.0 - rgb_back.0).abs() < 1e-6);
        assert!((rgb.1 - rgb_back.1).abs() < 1e-6);
        assert!((rgb.2 - rgb_back.2).abs() < 1e-6);
    }
}