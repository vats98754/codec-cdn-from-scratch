use std::io::{Read, Write};
use codec_common::{CodecError, Result};

const RANGE_CODER_TOP: u32 = 1 << 24;
const RANGE_CODER_BOTTOM: u32 = 1 << 16;

/// Simple range encoder for entropy coding
pub struct RangeEncoder<W: Write> {
    writer: W,
    low: u32,
    range: u32,
}

impl<W: Write> RangeEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            low: 0,
            range: u32::MAX,
        }
    }

    /// Encode a symbol with given frequency and cumulative frequency
    pub fn encode_symbol(&mut self, sym_freq: u32, cum_freq: u32, total_freq: u32) -> Result<()> {
        if total_freq == 0 || sym_freq == 0 {
            return Err(CodecError::EntropyCoding("Invalid frequency".to_string()));
        }
        
        let r = self.range / total_freq;
        let new_low = self.low.saturating_add(cum_freq.saturating_mul(r));
        let new_range = sym_freq.saturating_mul(r);
        
        if new_range == 0 {
            return Err(CodecError::EntropyCoding("Range underflow".to_string()));
        }
        
        self.low = new_low;
        self.range = new_range;

        while self.range < RANGE_CODER_BOTTOM {
            self.normalize()?;
        }
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        self.writer.write_all(&[(self.low >> 24) as u8])?;
        self.low <<= 8;
        self.range <<= 8;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<()> {
        for _ in 0..4 {
            self.normalize()?;
        }
        self.writer.flush()?;
        Ok(())
    }
}

/// Simple range decoder for entropy decoding
pub struct RangeDecoder<R: Read> {
    reader: R,
    low: u32,
    range: u32,
    code: u32,
}

impl<R: Read> RangeDecoder<R> {
    pub fn new(mut reader: R) -> Result<Self> {
        let mut code = 0;
        for _ in 0..4 {
            code = (code << 8) | Self::read_byte(&mut reader)? as u32;
        }

        Ok(Self {
            reader,
            low: 0,
            range: u32::MAX,
            code,
        })
    }

    /// Decode a symbol with given frequency table
    pub fn decode_symbol(&mut self, freqs: &[u32]) -> Result<usize> {
        let total_freq: u32 = freqs.iter().sum();
        
        if total_freq == 0 {
            return Err(CodecError::CorruptedData("Zero total frequency".to_string()));
        }
        
        let r = self.range / total_freq;
        if r == 0 {
            return Err(CodecError::CorruptedData("Range underflow".to_string()));
        }
        
        let scaled_code = (self.code.saturating_sub(self.low)) / r;

        let mut cum_freq = 0u32;
        for (i, &freq) in freqs.iter().enumerate() {
            if freq > 0 && scaled_code < cum_freq + freq {
                self.low += cum_freq * r;
                self.range = freq * r;

                while self.range < RANGE_CODER_BOTTOM {
                    if let Err(_) = self.normalize() {
                        return Err(CodecError::CorruptedData("Normalization failed".to_string()));
                    }
                }
                return Ok(i);
            }
            cum_freq += freq;
        }

        Err(CodecError::CorruptedData(format!("Invalid symbol: scaled_code={}, total_freq={}", scaled_code, total_freq)))
    }

    fn normalize(&mut self) -> Result<()> {
        self.low <<= 8;
        self.range <<= 8;
        match Self::read_byte(&mut self.reader) {
            Ok(byte) => {
                self.code = (self.code << 8) | byte as u32;
                Ok(())
            }
            Err(_) => {
                // End of stream - pad with zeros
                self.code <<= 8;
                Ok(())
            }
        }
    }

    fn read_byte(reader: &mut R) -> Result<u8> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

/// Simple frequency model for adaptive entropy coding
#[derive(Clone)]
pub struct FrequencyModel {
    freqs: Vec<u32>,
    total: u32,
}

impl FrequencyModel {
    pub fn new(alphabet_size: usize) -> Self {
        Self {
            freqs: vec![1; alphabet_size],
            total: alphabet_size as u32,
        }
    }

    pub fn get_frequency(&self, symbol: usize) -> u32 {
        self.freqs[symbol]
    }

    pub fn get_cumulative_frequency(&self, symbol: usize) -> u32 {
        self.freqs[..symbol].iter().sum()
    }

    pub fn get_total_frequency(&self) -> u32 {
        self.total
    }

    pub fn get_frequencies(&self) -> &[u32] {
        &self.freqs
    }

    pub fn update(&mut self, symbol: usize) {
        self.freqs[symbol] += 1;
        self.total += 1;

        // Prevent overflow by scaling down
        if self.total > (1 << 15) {
            for freq in &mut self.freqs {
                *freq = (*freq + 1) / 2;
            }
            self.total = self.freqs.iter().sum();
        }
    }
}