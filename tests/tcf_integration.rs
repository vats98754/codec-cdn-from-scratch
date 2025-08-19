use codec_tcf::{TcfEncoder, TcfDecoder, ModelParams};
use std::io::Cursor;

#[test]
fn test_tcf_roundtrip() {
    let input_text = "Hello, World! This is a test of the TCF codec.";
    
    // Encode
    let mut encoded_data = Vec::new();
    {
        let model_params = ModelParams::default();
        let mut encoder = TcfEncoder::new(Cursor::new(&mut encoded_data), model_params);
        encoder.encode(input_text).expect("Encoding should succeed");
    }
    
    // Decode
    let mut decoder = TcfDecoder::new(Cursor::new(&encoded_data));
    let decoded_text = decoder.decode().expect("Decoding should succeed");
    
    assert_eq!(input_text, decoded_text);
}

#[test]
fn test_unicode_handling() {
    let input_text = "Unicode test: café, naïve, 北京";
    
    // Encode
    let mut encoded_data = Vec::new();
    {
        let model_params = ModelParams::default();
        let mut encoder = TcfEncoder::new(Cursor::new(&mut encoded_data), model_params);
        encoder.encode(input_text).expect("Encoding should succeed");
    }
    
    // Decode
    let mut decoder = TcfDecoder::new(Cursor::new(&encoded_data));
    let decoded_text = decoder.decode().expect("Decoding should succeed");
    
    assert_eq!(input_text, decoded_text);
}

#[test]
fn test_empty_text() {
    let input_text = "";
    
    // Encode
    let mut encoded_data = Vec::new();
    {
        let model_params = ModelParams::default();
        let mut encoder = TcfEncoder::new(Cursor::new(&mut encoded_data), model_params);
        encoder.encode(input_text).expect("Encoding should succeed");
    }
    
    // Decode
    let mut decoder = TcfDecoder::new(Cursor::new(&encoded_data));
    let decoded_text = decoder.decode().expect("Decoding should succeed");
    
    assert_eq!(input_text, decoded_text);
}