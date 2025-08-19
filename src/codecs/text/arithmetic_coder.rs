use std::collections::HashMap;

/// High-precision arithmetic coder implementation
/// Uses 64-bit precision for better compression than traditional 32-bit implementations
pub struct ArithmeticCoder {
    low: u64,
    high: u64,
    pending_bits: u64,
    output: Vec<u8>,
    bit_buffer: u8,
    bit_count: u8,
}

impl ArithmeticCoder {
    const PRECISION: u64 = 62; // Use 62 bits to avoid overflow
    const MAX_VALUE: u64 = (1u64 << Self::PRECISION) - 1;
    const QUARTER: u64 = 1u64 << (Self::PRECISION - 2);
    const HALF: u64 = 2 * Self::QUARTER;
    const THREE_QUARTERS: u64 = 3 * Self::QUARTER;

    pub fn new() -> Self {
        Self {
            low: 0,
            high: Self::MAX_VALUE,
            pending_bits: 0,
            output: Vec::new(),
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    /// Encode a symbol with given probability range
    pub fn encode_symbol(&mut self, symbol_low: u64, symbol_high: u64, total: u64) {
        let range = self.high - self.low + 1;
        
        // Update bounds
        self.high = self.low + (range * symbol_high) / total - 1;
        self.low = self.low + (range * symbol_low) / total;

        // Emit bits to maintain precision
        while self.high < Self::HALF || self.low >= Self::HALF {
            if self.high < Self::HALF {
                self.output_bit(0);
                self.output_pending_bits(1);
            } else {
                self.output_bit(1);
                self.output_pending_bits(0);
                self.low -= Self::HALF;
                self.high -= Self::HALF;
            }
            
            self.low <<= 1;
            self.high = (self.high << 1) | 1;
        }

        // Handle quarter case
        while self.low >= Self::QUARTER && self.high < Self::THREE_QUARTERS {
            self.pending_bits += 1;
            self.low = (self.low - Self::QUARTER) << 1;
            self.high = ((self.high - Self::QUARTER) << 1) | 1;
        }
    }

    /// Finish encoding and return compressed data
    pub fn finish(mut self) -> Vec<u8> {
        // Output final bits
        self.pending_bits += 1;
        if self.low < Self::QUARTER {
            self.output_bit(0);
            self.output_pending_bits(1);
        } else {
            self.output_bit(1);
            self.output_pending_bits(0);
        }

        // Flush remaining bits
        if self.bit_count > 0 {
            self.output.push(self.bit_buffer);
        }

        self.output
    }

    fn output_bit(&mut self, bit: u8) {
        self.bit_buffer = (self.bit_buffer << 1) | bit;
        self.bit_count += 1;
        
        if self.bit_count == 8 {
            self.output.push(self.bit_buffer);
            self.bit_buffer = 0;
            self.bit_count = 0;
        }
    }

    fn output_pending_bits(&mut self, bit: u8) {
        for _ in 0..self.pending_bits {
            self.output_bit(bit);
        }
        self.pending_bits = 0;
    }
}

/// High-precision arithmetic decoder
pub struct ArithmeticDecoder {
    low: u64,
    high: u64,
    value: u64,
    input: Vec<u8>,
    byte_pos: usize,
    bit_pos: u8,
}

impl ArithmeticDecoder {
    const PRECISION: u64 = 62;
    const MAX_VALUE: u64 = (1u64 << Self::PRECISION) - 1;
    const QUARTER: u64 = 1u64 << (Self::PRECISION - 2);
    const HALF: u64 = 2 * Self::QUARTER;
    const THREE_QUARTERS: u64 = 3 * Self::QUARTER;

    pub fn new(input: Vec<u8>) -> Self {
        let mut decoder = Self {
            low: 0,
            high: Self::MAX_VALUE,
            value: 0,
            input,
            byte_pos: 0,
            bit_pos: 0,
        };

        // Initialize value with first bits
        for _ in 0..Self::PRECISION {
            decoder.value = (decoder.value << 1) | decoder.input_bit() as u64;
        }

        decoder
    }

    /// Get the current symbol value for decoding
    pub fn get_symbol_value(&self, total: u64) -> u64 {
        let range = self.high - self.low + 1;
        ((self.value - self.low + 1) * total - 1) / range
    }

    /// Decode a symbol with given probability range
    pub fn decode_symbol(&mut self, symbol_low: u64, symbol_high: u64, total: u64) {
        let range = self.high - self.low + 1;
        
        // Update bounds
        self.high = self.low + (range * symbol_high) / total - 1;
        self.low = self.low + (range * symbol_low) / total;

        // Maintain precision
        while self.high < Self::HALF || self.low >= Self::HALF {
            if self.high < Self::HALF {
                // Do nothing for low
            } else {
                self.low -= Self::HALF;
                self.high -= Self::HALF;
                self.value -= Self::HALF;
            }
            
            self.low <<= 1;
            self.high = (self.high << 1) | 1;
            self.value = (self.value << 1) | self.input_bit() as u64;
        }

        // Handle quarter case
        while self.low >= Self::QUARTER && self.high < Self::THREE_QUARTERS {
            self.low = (self.low - Self::QUARTER) << 1;
            self.high = ((self.high - Self::QUARTER) << 1) | 1;
            self.value = ((self.value - Self::QUARTER) << 1) | self.input_bit() as u64;
        }
    }

    fn input_bit(&mut self) -> u8 {
        if self.byte_pos >= self.input.len() {
            return 0;
        }

        let bit = (self.input[self.byte_pos] >> (7 - self.bit_pos)) & 1;
        self.bit_pos += 1;
        
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }

        bit
    }
}

/// Adaptive frequency model for arithmetic coding
pub struct FrequencyModel {
    frequencies: HashMap<u8, u64>,
    total_frequency: u64,
    symbols: Vec<u8>,
}

impl FrequencyModel {
    pub fn new() -> Self {
        Self {
            frequencies: HashMap::new(),
            total_frequency: 0,
            symbols: Vec::new(),
        }
    }

    /// Build model from input data
    pub fn build_from_data(&mut self, data: &[u8]) {
        self.frequencies.clear();
        self.symbols.clear();
        self.total_frequency = 0;

        // Count frequencies
        for &byte in data {
            *self.frequencies.entry(byte).or_insert(0) += 1;
            self.total_frequency += 1;
        }

        // Create sorted symbol list for consistent encoding/decoding
        self.symbols = self.frequencies.keys().copied().collect();
        self.symbols.sort();

        // Ensure minimum frequency for each symbol (Laplace smoothing)
        for symbol in &self.symbols {
            let freq = self.frequencies.get_mut(symbol).unwrap();
            if *freq == 0 {
                *freq = 1;
                self.total_frequency += 1;
            }
        }
    }

    /// Get probability range for a symbol
    pub fn get_symbol_range(&self, symbol: u8) -> Option<(u64, u64)> {
        let mut cumulative = 0;
        
        for &s in &self.symbols {
            let freq = *self.frequencies.get(&s)?;
            if s == symbol {
                return Some((cumulative, cumulative + freq));
            }
            cumulative += freq;
        }
        
        None
    }

    /// Get symbol from cumulative value
    pub fn get_symbol_from_value(&self, value: u64) -> Option<u8> {
        let mut cumulative = 0;
        
        for &symbol in &self.symbols {
            let freq = self.frequencies.get(&symbol)?;
            if value >= cumulative && value < cumulative + freq {
                return Some(symbol);
            }
            cumulative += freq;
        }
        
        None
    }

    /// Get symbol range from cumulative value
    pub fn get_range_from_value(&self, value: u64) -> Option<(u8, u64, u64)> {
        let mut cumulative = 0;
        
        for &symbol in &self.symbols {
            let freq = self.frequencies.get(&symbol)?;
            if value >= cumulative && value < cumulative + freq {
                return Some((symbol, cumulative, cumulative + freq));
            }
            cumulative += freq;
        }
        
        None
    }

    pub fn total_frequency(&self) -> u64 {
        self.total_frequency
    }

    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(&(self.frequencies.clone(), self.symbols.clone())).unwrap_or_default()
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let (frequencies, symbols): (HashMap<u8, u64>, Vec<u8>) = serde_json::from_slice(data)?;
        let total_frequency = frequencies.values().sum();
        
        Ok(Self {
            frequencies,
            total_frequency,
            symbols,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_coding_roundtrip() {
        let test_data = b"Hello, World! This is a test of arithmetic coding.";
        
        // Build frequency model
        let mut model = FrequencyModel::new();
        model.build_from_data(test_data);
        
        // Encode
        let mut encoder = ArithmeticCoder::new();
        for &byte in test_data {
            if let Some((low, high)) = model.get_symbol_range(byte) {
                encoder.encode_symbol(low, high, model.total_frequency());
            }
        }
        let compressed = encoder.finish();
        
        // Decode
        let mut decoder = ArithmeticDecoder::new(compressed);
        let mut decoded = Vec::new();
        
        for _ in 0..test_data.len() {
            let value = decoder.get_symbol_value(model.total_frequency());
            if let Some((symbol, low, high)) = model.get_range_from_value(value) {
                decoded.push(symbol);
                decoder.decode_symbol(low, high, model.total_frequency());
            }
        }
        
        assert_eq!(test_data, decoded.as_slice());
    }
}