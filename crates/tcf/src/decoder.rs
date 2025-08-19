use std::io::Read;
use codec_common::{BitstreamReader, CodecError, Result};
use codec_entropy::RangeDecoder;
use crate::format::{TcfFile, TcfHeader, ModelParams, ChunkEntry, ChunkType};
use crate::model::{TextModel, build_alphabet};

/// TCF decoder for decompressing text
pub struct TcfDecoder<R: Read> {
    reader: BitstreamReader<R>,
}

impl<R: Read> TcfDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: BitstreamReader::new(reader),
        }
    }

    /// Decode TCF format to text
    pub fn decode(&mut self) -> Result<String> {
        // Read header
        let header = self.read_header()?;
        header.validate()?;

        // Read model parameters
        let model_params = self.read_model_params()?;

        // Read chunk table
        let chunks = self.read_chunk_table()?;

        // Find the compressed data chunk
        let data_chunk = chunks
            .iter()
            .find(|chunk| chunk.chunk_type == ChunkType::CompressedData)
            .ok_or_else(|| CodecError::InvalidFormat("No data chunk found".to_string()))?;

        // Read and decode the compressed data
        let compressed_data = self.read_chunk_data(data_chunk)?;
        let text = self.decode_text_data(&compressed_data, &model_params)?;

        Ok(text)
    }

    fn read_header(&mut self) -> Result<TcfHeader> {
        let mut magic = [0u8; 4];
        self.reader.read_bytes(&mut magic)?;
        
        let header = TcfHeader {
            magic,
            version: self.reader.read_u32()?,
            flags: self.reader.read_u32()?,
            model_params_offset: self.reader.read_u32()?,
            chunk_table_offset: self.reader.read_u32()?,
            data_offset: self.reader.read_u32()?,
        };

        Ok(header)
    }

    fn read_model_params(&mut self) -> Result<ModelParams> {
        let params = ModelParams {
            seed: self.reader.read_u32()?,
            max_order: self.reader.read_u8()?,
            adaptation_rate: self.reader.read_u8()?,
            use_escape: self.reader.read_u8()? != 0,
            use_dictionary: self.reader.read_u8()? != 0,
        };

        Ok(params)
    }

    fn read_chunk_table(&mut self) -> Result<Vec<ChunkEntry>> {
        let chunk_count = self.reader.read_u32()? as usize;
        let mut chunks = Vec::with_capacity(chunk_count);

        for _ in 0..chunk_count {
            let chunk = ChunkEntry {
                offset: self.reader.read_u32()?,
                size: self.reader.read_u32()?,
                checksum: self.reader.read_u32()?,
                chunk_type: ChunkType::try_from(self.reader.read_u32()?)?,
            };
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    fn read_chunk_data(&mut self, chunk: &ChunkEntry) -> Result<Vec<u8>> {
        let mut data = vec![0u8; chunk.size as usize];
        self.reader.read_bytes(&mut data)?;

        // Verify checksum
        let calculated_checksum = crc32fast::hash(&data);
        if calculated_checksum != chunk.checksum {
            return Err(CodecError::CorruptedData("Checksum mismatch".to_string()));
        }

        Ok(data)
    }

    fn decode_text_data(&mut self, compressed_data: &[u8], _model_params: &ModelParams) -> Result<String> {
        let mut range_decoder = RangeDecoder::new(std::io::Cursor::new(compressed_data))?;
        let mut result = String::new();

        // Use the same alphabet as encoder
        let alphabet: Vec<char> = (32u8..127u8).map(|b| b as char).collect();

        // Decode length (4 bytes)
        let mut length = 0usize;
        for i in 0..4 {
            let byte_freqs = vec![1u32; 256];
            let byte_val = range_decoder.decode_symbol(&byte_freqs)?;
            length |= (byte_val as usize) << (i * 8);
        }

        // Use the same frequency model as encoder
        let char_freq = 10u32;
        let escape_freq = 1u32;
        let total_freq = (alphabet.len() as u32) * char_freq + escape_freq;
        let mut freqs = vec![char_freq; alphabet.len()];
        freqs.push(escape_freq);

        // Decode exactly 'length' characters
        for _ in 0..length {
            let symbol_idx = range_decoder.decode_symbol(&freqs)?;
            
            if symbol_idx < alphabet.len() {
                // Regular character from alphabet
                let ch = alphabet[symbol_idx];
                result.push(ch);
            } else {
                // Escape symbol - decode raw character
                let byte_freqs = vec![1u32; 256];
                let char_code = range_decoder.decode_symbol(&byte_freqs)? as u8;
                let ch = char_code as char;
                result.push(ch);
            }
        }

        Ok(result)
    }
}