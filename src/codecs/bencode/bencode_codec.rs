use super::bencode_value::BencodeValue;
use std::collections::HashMap;
use thiserror::Error;
use anyhow::{Result, Context};

#[derive(Error, Debug)]
pub enum BencodeError {
    #[error("Invalid bencode format: {0}")]
    InvalidFormat(String),
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Invalid integer format: {0}")]
    InvalidInteger(String),
    #[error("Invalid string length: {0}")]
    InvalidStringLength(String),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// High-performance Bencode encoder/decoder
/// 
/// Bencode is a simple, efficient serialization format used by BitTorrent.
/// It supports four data types:
/// - Integers: i<number>e (e.g., i42e)
/// - Byte strings: <length>:<string> (e.g., 4:spam)
/// - Lists: l<contents>e (e.g., l4:spam4:eggse)
/// - Dictionaries: d<contents>e (e.g., d3:cow3:moo4:spam4:eggse)
///
/// This implementation provides:
/// - Zero-copy parsing where possible
/// - Streaming encoder for large data
/// - Comprehensive error handling
/// - Memory-efficient operations
pub struct BencodeCodec;

impl BencodeCodec {
    /// Encode a BencodeValue to bencode format
    pub fn encode(value: &BencodeValue) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(value.encoded_size());
        Self::encode_to_writer(value, &mut result)?;
        Ok(result)
    }

    /// Encode a BencodeValue to a writer (for streaming)
    pub fn encode_to_writer<W: std::io::Write>(value: &BencodeValue, writer: &mut W) -> Result<()> {
        match value {
            BencodeValue::Integer(i) => {
                write!(writer, "i{}e", i)?;
            }
            BencodeValue::ByteString(s) => {
                write!(writer, "{}:", s.len())?;
                writer.write_all(s)?;
            }
            BencodeValue::List(l) => {
                writer.write_all(b"l")?;
                for item in l {
                    Self::encode_to_writer(item, writer)?;
                }
                writer.write_all(b"e")?;
            }
            BencodeValue::Dictionary(d) => {
                writer.write_all(b"d")?;
                
                // Sort keys for deterministic output (Bencode requirement)
                let mut sorted_keys: Vec<_> = d.keys().collect();
                sorted_keys.sort();
                
                for key in sorted_keys {
                    // Encode key as byte string
                    write!(writer, "{}:", key.len())?;
                    writer.write_all(key)?;
                    // Encode value
                    Self::encode_to_writer(d.get(key).unwrap(), writer)?;
                }
                writer.write_all(b"e")?;
            }
        }
        Ok(())
    }

    /// Decode bencode data to a BencodeValue
    pub fn decode(data: &[u8]) -> Result<BencodeValue> {
        let mut position = 0;
        Self::decode_value(data, &mut position)
    }

    /// Decode a value from data starting at position
    fn decode_value(data: &[u8], position: &mut usize) -> Result<BencodeValue> {
        if *position >= data.len() {
            return Err(BencodeError::UnexpectedEof.into());
        }

        match data[*position] {
            b'i' => Self::decode_integer(data, position),
            b'l' => Self::decode_list(data, position),
            b'd' => Self::decode_dictionary(data, position),
            b'0'..=b'9' => Self::decode_byte_string(data, position),
            _ => Err(BencodeError::InvalidFormat(
                format!("Unexpected character '{}' at position {}", 
                       data[*position] as char, *position)
            ).into()),
        }
    }

    /// Decode an integer: i<number>e
    fn decode_integer(data: &[u8], position: &mut usize) -> Result<BencodeValue> {
        if data[*position] != b'i' {
            return Err(BencodeError::InvalidFormat("Expected 'i' for integer".to_string()).into());
        }
        *position += 1; // Skip 'i'

        let start = *position;
        while *position < data.len() && data[*position] != b'e' {
            *position += 1;
        }

        if *position >= data.len() {
            return Err(BencodeError::UnexpectedEof.into());
        }

        let number_str = std::str::from_utf8(&data[start..*position])
            .context("Invalid UTF-8 in integer")?;
        
        let number = number_str.parse::<i64>()
            .map_err(|_| BencodeError::InvalidInteger(number_str.to_string()))?;

        *position += 1; // Skip 'e'
        Ok(BencodeValue::Integer(number))
    }

    /// Decode a byte string: <length>:<string>
    fn decode_byte_string(data: &[u8], position: &mut usize) -> Result<BencodeValue> {
        let start = *position;
        while *position < data.len() && data[*position] != b':' {
            if !data[*position].is_ascii_digit() {
                return Err(BencodeError::InvalidFormat(
                    "Invalid character in string length".to_string()
                ).into());
            }
            *position += 1;
        }

        if *position >= data.len() {
            return Err(BencodeError::UnexpectedEof.into());
        }

        let length_str = std::str::from_utf8(&data[start..*position])
            .context("Invalid UTF-8 in string length")?;
        
        let length = length_str.parse::<usize>()
            .map_err(|_| BencodeError::InvalidStringLength(length_str.to_string()))?;

        *position += 1; // Skip ':'

        if *position + length > data.len() {
            return Err(BencodeError::UnexpectedEof.into());
        }

        let string_data = data[*position..*position + length].to_vec();
        *position += length;

        Ok(BencodeValue::ByteString(string_data))
    }

    /// Decode a list: l<contents>e
    fn decode_list(data: &[u8], position: &mut usize) -> Result<BencodeValue> {
        if data[*position] != b'l' {
            return Err(BencodeError::InvalidFormat("Expected 'l' for list".to_string()).into());
        }
        *position += 1; // Skip 'l'

        let mut items = Vec::new();
        while *position < data.len() && data[*position] != b'e' {
            items.push(Self::decode_value(data, position)?);
        }

        if *position >= data.len() {
            return Err(BencodeError::UnexpectedEof.into());
        }

        *position += 1; // Skip 'e'
        Ok(BencodeValue::List(items))
    }

    /// Decode a dictionary: d<contents>e
    fn decode_dictionary(data: &[u8], position: &mut usize) -> Result<BencodeValue> {
        if data[*position] != b'd' {
            return Err(BencodeError::InvalidFormat("Expected 'd' for dictionary".to_string()).into());
        }
        *position += 1; // Skip 'd'

        let mut dict = HashMap::new();
        while *position < data.len() && data[*position] != b'e' {
            // Decode key (must be a byte string)
            let key_value = Self::decode_value(data, position)?;
            let key = match key_value {
                BencodeValue::ByteString(k) => k,
                _ => return Err(BencodeError::InvalidFormat(
                    "Dictionary keys must be byte strings".to_string()
                ).into()),
            };

            // Decode value
            let value = Self::decode_value(data, position)?;
            dict.insert(key, value);
        }

        if *position >= data.len() {
            return Err(BencodeError::UnexpectedEof.into());
        }

        *position += 1; // Skip 'e'
        Ok(BencodeValue::Dictionary(dict))
    }

    /// Create a bencode file format with metadata
    pub fn create_file_format(
        content: &BencodeValue,
        metadata: Option<&BencodeValue>
    ) -> Result<Vec<u8>> {
        let mut dict = HashMap::new();
        
        // Add content
        dict.insert(b"content".to_vec(), content.clone());
        
        // Add metadata if provided
        if let Some(meta) = metadata {
            dict.insert(b"metadata".to_vec(), meta.clone());
        }
        
        // Add file format info
        dict.insert(b"format".to_vec(), BencodeValue::string("bencode"));
        dict.insert(b"version".to_vec(), BencodeValue::integer(1));
        dict.insert(b"created".to_vec(), BencodeValue::integer(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        ));
        
        let file_value = BencodeValue::Dictionary(dict);
        Self::encode(&file_value)
    }

    /// Parse a bencode file format and extract content
    pub fn parse_file_format(data: &[u8]) -> Result<(BencodeValue, Option<BencodeValue>)> {
        let file_value = Self::decode(data)?;
        
        let dict = file_value.as_dictionary()
            .ok_or_else(|| BencodeError::InvalidFormat("File must be a dictionary".to_string()))?;
        
        let content = dict.get(b"content".as_slice())
            .ok_or_else(|| BencodeError::InvalidFormat("Missing 'content' field".to_string()))?
            .clone();
        
        let metadata = dict.get(b"metadata".as_slice()).cloned();
        
        Ok((content, metadata))
    }

    /// Get compression statistics
    pub fn get_stats(original_data: &[u8], encoded_data: &[u8]) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("original_size".to_string(), serde_json::Value::Number(
            serde_json::Number::from(original_data.len())
        ));
        stats.insert("encoded_size".to_string(), serde_json::Value::Number(
            serde_json::Number::from(encoded_data.len())
        ));
        
        let ratio = if original_data.len() > 0 {
            encoded_data.len() as f64 / original_data.len() as f64
        } else {
            1.0
        };
        stats.insert("compression_ratio".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(ratio).unwrap_or(serde_json::Number::from(1))
        ));
        
        let savings = if original_data.len() > 0 {
            ((original_data.len() as f64 - encoded_data.len() as f64) / original_data.len() as f64) * 100.0
        } else {
            0.0
        };
        stats.insert("space_savings_percent".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(savings).unwrap_or(serde_json::Number::from(0))
        ));
        
        stats.insert("format".to_string(), serde_json::Value::String("Bencode".to_string()));
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_integer() {
        let value = BencodeValue::integer(42);
        let encoded = BencodeCodec::encode(&value).unwrap();
        assert_eq!(encoded, b"i42e");
        
        let decoded = BencodeCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_string() {
        let value = BencodeValue::string("hello");
        let encoded = BencodeCodec::encode(&value).unwrap();
        assert_eq!(encoded, b"5:hello");
        
        let decoded = BencodeCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_list() {
        let value = BencodeValue::list(vec![
            BencodeValue::integer(1),
            BencodeValue::string("test"),
            BencodeValue::integer(2),
        ]);
        
        let encoded = BencodeCodec::encode(&value).unwrap();
        let decoded = BencodeCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_dictionary() {
        let mut dict = HashMap::new();
        dict.insert(b"name".to_vec(), BencodeValue::string("test"));
        dict.insert(b"value".to_vec(), BencodeValue::integer(42));
        
        let value = BencodeValue::dictionary(dict);
        let encoded = BencodeCodec::encode(&value).unwrap();
        let decoded = BencodeCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_complex_structure() {
        let mut torrent_info = HashMap::new();
        torrent_info.insert(b"name".to_vec(), BencodeValue::string("example.txt"));
        torrent_info.insert(b"length".to_vec(), BencodeValue::integer(1024));
        torrent_info.insert(b"piece length".to_vec(), BencodeValue::integer(32768));
        
        let mut torrent = HashMap::new();
        torrent.insert(b"announce".to_vec(), BencodeValue::string("http://tracker.example.com"));
        torrent.insert(b"info".to_vec(), BencodeValue::dictionary(torrent_info));
        
        let value = BencodeValue::dictionary(torrent);
        let encoded = BencodeCodec::encode(&value).unwrap();
        let decoded = BencodeCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_file_format() {
        let content = BencodeValue::string("test content");
        let mut metadata = HashMap::new();
        metadata.insert(b"author".to_vec(), BencodeValue::string("test"));
        let metadata_value = BencodeValue::dictionary(metadata);
        
        let file_data = BencodeCodec::create_file_format(&content, Some(&metadata_value)).unwrap();
        let (parsed_content, parsed_metadata) = BencodeCodec::parse_file_format(&file_data).unwrap();
        
        assert_eq!(parsed_content, content);
        assert!(parsed_metadata.is_some());
    }

    #[test]
    fn test_error_handling() {
        // Test invalid data
        assert!(BencodeCodec::decode(b"invalid").is_err());
        assert!(BencodeCodec::decode(b"i42").is_err()); // Missing 'e'
        assert!(BencodeCodec::decode(b"5:hell").is_err()); // String too short
    }
}