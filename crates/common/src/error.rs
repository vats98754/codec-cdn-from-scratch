use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("Corrupted data: {0}")]
    CorruptedData(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Entropy coding error: {0}")]
    EntropyCoding(String),
    
    #[error("Unicode error: {0}")]
    Unicode(String),
}

pub type Result<T> = std::result::Result<T, CodecError>;