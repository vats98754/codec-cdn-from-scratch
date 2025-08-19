use codec_common::{CodecError, Result};

/// TCF file format constants
pub const TCF_MAGIC: &[u8; 4] = b"TCF1";
pub const TCF_VERSION: u32 = 1;

/// TCF file header structure
#[derive(Debug, Clone)]
pub struct TcfHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub flags: u32,
    pub model_params_offset: u32,
    pub chunk_table_offset: u32,
    pub data_offset: u32,
}

impl TcfHeader {
    pub fn new() -> Self {
        Self {
            magic: *TCF_MAGIC,
            version: TCF_VERSION,
            flags: 0,
            model_params_offset: 0,
            chunk_table_offset: 0,
            data_offset: 0,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if &self.magic != TCF_MAGIC {
            return Err(CodecError::InvalidFormat("Invalid TCF magic".to_string()));
        }
        if self.version != TCF_VERSION {
            return Err(CodecError::UnsupportedVersion(self.version));
        }
        Ok(())
    }
}

/// Model parameters for PPM/CM compression
#[derive(Debug, Clone)]
pub struct ModelParams {
    pub seed: u32,
    pub max_order: u8,
    pub adaptation_rate: u8,
    pub use_escape: bool,
    pub use_dictionary: bool,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            seed: 12345,
            max_order: 4,
            adaptation_rate: 128,
            use_escape: true,
            use_dictionary: false,
        }
    }
}

/// Chunk table entry
#[derive(Debug, Clone)]
pub struct ChunkEntry {
    pub offset: u32,
    pub size: u32,
    pub checksum: u32,
    pub chunk_type: ChunkType,
}

/// Type of chunk in TCF file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkType {
    CompressedData = 0,
    Dictionary = 1,
    Metadata = 2,
}

impl TryFrom<u32> for ChunkType {
    type Error = CodecError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(ChunkType::CompressedData),
            1 => Ok(ChunkType::Dictionary),
            2 => Ok(ChunkType::Metadata),
            _ => Err(CodecError::InvalidFormat(format!("Unknown chunk type: {}", value))),
        }
    }
}

/// TCF file structure
#[derive(Debug)]
pub struct TcfFile {
    pub header: TcfHeader,
    pub model_params: ModelParams,
    pub chunks: Vec<ChunkEntry>,
    pub data: Vec<u8>,
}

impl TcfFile {
    pub fn new() -> Self {
        Self {
            header: TcfHeader::new(),
            model_params: ModelParams::default(),
            chunks: Vec::new(),
            data: Vec::new(),
        }
    }
}