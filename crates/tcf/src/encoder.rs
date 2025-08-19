use std::io::Write;
use codec_common::{BitstreamWriter, CodecError, Result};
use codec_entropy::{RangeEncoder, FrequencyModel};
use crate::format::{TcfFile, TcfHeader, ModelParams, ChunkEntry, ChunkType};
use crate::model::{TextModel, build_alphabet};

/// TCF encoder for compressing text
pub struct TcfEncoder<W: Write> {
    writer: BitstreamWriter<W>,
    model_params: ModelParams,
}

impl<W: Write> TcfEncoder<W> {
    pub fn new(writer: W, model_params: ModelParams) -> Self {
        Self {
            writer: BitstreamWriter::new(writer),
            model_params,
        }
    }

    /// Encode text to TCF format
    pub fn encode(&mut self, text: &str) -> Result<()> {
        // Normalize the text
        let normalized_text = TextModel::normalize_text(text);
        
        // Build alphabet
        let alphabet = build_alphabet(&normalized_text);
        
        // Create text model
        let mut text_model = TextModel::new(self.model_params.max_order as usize);
        
        // Prepare file structure
        let mut tcf_file = TcfFile::new();
        tcf_file.model_params = self.model_params.clone();
        
        // Write header (placeholder for now)
        self.write_header(&tcf_file.header)?;
        
        // Write model parameters
        let model_params_offset = self.get_current_position()?;
        self.write_model_params(&tcf_file.model_params)?;
        
        // Encode the text data
        let compressed_data = self.encode_text_data(&normalized_text, &alphabet, &mut text_model)?;
        
        // Create chunk entry
        let data_offset = self.get_current_position()?;
        let chunk = ChunkEntry {
            offset: data_offset,
            size: compressed_data.len() as u32,
            checksum: crc32fast::hash(&compressed_data),
            chunk_type: ChunkType::CompressedData,
        };
        
        // Write chunk table
        let chunk_table_offset = self.get_current_position()?;
        self.write_chunk_table(&[chunk])?;
        
        // Write data
        self.writer.write_bytes(&compressed_data)?;
        
        // Update header with correct offsets
        let mut updated_header = tcf_file.header;
        updated_header.model_params_offset = model_params_offset;
        updated_header.chunk_table_offset = chunk_table_offset;
        updated_header.data_offset = data_offset;
        
        // Rewrite header at the beginning
        self.rewrite_header(&updated_header)?;
        
        self.writer.flush()?;
        Ok(())
    }

    fn encode_text_data(&mut self, text: &str, alphabet: &[char], _text_model: &mut TextModel) -> Result<Vec<u8>> {
        let mut compressed_data = Vec::new();
        let mut range_encoder = RangeEncoder::new(&mut compressed_data);
        
        let chars: Vec<char> = text.chars().collect();
        
        // Encode length as a simple fixed-size field
        let length = chars.len();
        for i in 0..4 {
            let byte = ((length >> (i * 8)) & 0xFF) as u8;
            range_encoder.encode_symbol(1, byte as u32, 256)?;
        }
        
        // Use a simple static frequency model
        let char_freq = 10u32;
        let escape_freq = 1u32;
        let total_freq = (alphabet.len() as u32) * char_freq + escape_freq;
        
        // Encode each character
        for &ch in &chars {
            if let Some(symbol_idx) = alphabet.iter().position(|&c| c == ch) {
                // Character is in alphabet
                let cum_freq = (symbol_idx as u32) * char_freq;
                range_encoder.encode_symbol(char_freq, cum_freq, total_freq)?;
            } else {
                // Use escape
                let cum_freq = (alphabet.len() as u32) * char_freq;
                range_encoder.encode_symbol(escape_freq, cum_freq, total_freq)?;
                
                // Encode raw character
                let char_code = (ch as u32) % 256;
                range_encoder.encode_symbol(1, char_code, 256)?;
            }
        }
        
        range_encoder.finish()?;
        Ok(compressed_data)
    }

    fn write_header(&mut self, header: &TcfHeader) -> Result<()> {
        self.writer.write_bytes(&header.magic)?;
        self.writer.write_u32(header.version)?;
        self.writer.write_u32(header.flags)?;
        self.writer.write_u32(header.model_params_offset)?;
        self.writer.write_u32(header.chunk_table_offset)?;
        self.writer.write_u32(header.data_offset)?;
        Ok(())
    }

    fn write_model_params(&mut self, params: &ModelParams) -> Result<()> {
        self.writer.write_u32(params.seed)?;
        self.writer.write_u8(params.max_order)?;
        self.writer.write_u8(params.adaptation_rate)?;
        self.writer.write_u8(if params.use_escape { 1 } else { 0 })?;
        self.writer.write_u8(if params.use_dictionary { 1 } else { 0 })?;
        Ok(())
    }

    fn write_chunk_table(&mut self, chunks: &[ChunkEntry]) -> Result<()> {
        self.writer.write_u32(chunks.len() as u32)?;
        for chunk in chunks {
            self.writer.write_u32(chunk.offset)?;
            self.writer.write_u32(chunk.size)?;
            self.writer.write_u32(chunk.checksum)?;
            self.writer.write_u32(chunk.chunk_type as u32)?;
        }
        Ok(())
    }

    fn get_current_position(&self) -> Result<u32> {
        // Simplified - in a real implementation, track position properly
        Ok(0)
    }

    fn rewrite_header(&mut self, _header: &TcfHeader) -> Result<()> {
        // Simplified - in a real implementation, seek to beginning and rewrite
        Ok(())
    }
}