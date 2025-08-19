use std::collections::HashMap;
use std::fmt;

/// Represents a Bencode value
/// Bencode supports four types: integers, byte strings, lists, and dictionaries
#[derive(Debug, Clone, PartialEq)]
pub enum BencodeValue {
    Integer(i64),
    ByteString(Vec<u8>),
    List(Vec<BencodeValue>),
    Dictionary(HashMap<Vec<u8>, BencodeValue>),
}

impl BencodeValue {
    /// Create a new integer value
    pub fn integer(value: i64) -> Self {
        BencodeValue::Integer(value)
    }

    /// Create a new byte string value
    pub fn byte_string(value: Vec<u8>) -> Self {
        BencodeValue::ByteString(value)
    }

    /// Create a new string value (convenience method)
    pub fn string(value: &str) -> Self {
        BencodeValue::ByteString(value.as_bytes().to_vec())
    }

    /// Create a new list value
    pub fn list(value: Vec<BencodeValue>) -> Self {
        BencodeValue::List(value)
    }

    /// Create a new dictionary value
    pub fn dictionary(value: HashMap<Vec<u8>, BencodeValue>) -> Self {
        BencodeValue::Dictionary(value)
    }

    /// Get the value as an integer if possible
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            BencodeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get the value as a byte string if possible
    pub fn as_byte_string(&self) -> Option<&Vec<u8>> {
        match self {
            BencodeValue::ByteString(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as a UTF-8 string if possible
    pub fn as_string(&self) -> Option<String> {
        match self {
            BencodeValue::ByteString(s) => String::from_utf8(s.clone()).ok(),
            _ => None,
        }
    }

    /// Get the value as a list if possible
    pub fn as_list(&self) -> Option<&Vec<BencodeValue>> {
        match self {
            BencodeValue::List(l) => Some(l),
            _ => None,
        }
    }

    /// Get the value as a dictionary if possible
    pub fn as_dictionary(&self) -> Option<&HashMap<Vec<u8>, BencodeValue>> {
        match self {
            BencodeValue::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    /// Get a dictionary value by string key
    pub fn get_dict_value(&self, key: &str) -> Option<&BencodeValue> {
        match self {
            BencodeValue::Dictionary(d) => d.get(key.as_bytes()),
            _ => None,
        }
    }

    /// Get the estimated encoded size in bytes
    pub fn encoded_size(&self) -> usize {
        match self {
            BencodeValue::Integer(i) => {
                format!("i{}e", i).len()
            }
            BencodeValue::ByteString(s) => {
                format!("{}:", s.len()).len() + s.len()
            }
            BencodeValue::List(l) => {
                2 + l.iter().map(|v| v.encoded_size()).sum::<usize>() // "l" + content + "e"
            }
            BencodeValue::Dictionary(d) => {
                2 + d.iter().map(|(k, v)| {
                    format!("{}:", k.len()).len() + k.len() + v.encoded_size()
                }).sum::<usize>() // "d" + content + "e"
            }
        }
    }
}

impl fmt::Display for BencodeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BencodeValue::Integer(i) => write!(f, "{}", i),
            BencodeValue::ByteString(s) => {
                if let Ok(string) = String::from_utf8(s.clone()) {
                    write!(f, "\"{}\"", string)
                } else {
                    write!(f, "<{} bytes>", s.len())
                }
            }
            BencodeValue::List(l) => {
                write!(f, "[")?;
                for (i, item) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            BencodeValue::Dictionary(d) => {
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in d.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    
                    if let Ok(key_str) = String::from_utf8(k.clone()) {
                        write!(f, "\"{}\": {}", key_str, v)?;
                    } else {
                        write!(f, "<{} bytes>: {}", k.len(), v)?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_value() {
        let value = BencodeValue::integer(42);
        assert_eq!(value.as_integer(), Some(42));
        assert_eq!(value.encoded_size(), 4); // "i42e"
    }

    #[test]
    fn test_string_value() {
        let value = BencodeValue::string("hello");
        assert_eq!(value.as_string(), Some("hello".to_string()));
        assert_eq!(value.encoded_size(), 7); // "5:hello"
    }

    #[test]
    fn test_list_value() {
        let value = BencodeValue::list(vec![
            BencodeValue::integer(1),
            BencodeValue::string("test"),
        ]);
        assert_eq!(value.as_list().unwrap().len(), 2);
    }

    #[test]
    fn test_dictionary_value() {
        let mut dict = HashMap::new();
        dict.insert(b"key".to_vec(), BencodeValue::string("value"));
        let value = BencodeValue::dictionary(dict);
        
        assert_eq!(value.get_dict_value("key").unwrap().as_string().unwrap(), "value");
    }

    #[test]
    fn test_display() {
        let value = BencodeValue::integer(42);
        assert_eq!(format!("{}", value), "42");
        
        let value = BencodeValue::string("hello");
        assert_eq!(format!("{}", value), "\"hello\"");
    }
}