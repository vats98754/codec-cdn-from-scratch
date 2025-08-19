use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use codec_cdn_rust::codecs::{
    text::{TcfCodec},
    bencode::{BencodeCodec, BencodeValue},
};
use std::time::Duration;
use std::collections::HashMap;

fn create_test_texts() -> Vec<(String, String)> {
    vec![
        ("Small".to_string(), "Hello, World!".to_string()),
        ("Medium".to_string(), "The quick brown fox jumps over the lazy dog. ".repeat(50)),
        ("Large".to_string(), "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(200)),
        ("Repetitive".to_string(), "aaaaaaaaaa".repeat(100)),
        ("Random".to_string(), "abcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()".repeat(25)),
        ("Code".to_string(), include_str!("../src/main.rs").to_string()),
        ("JSON-like".to_string(), r#"{"name":"test","values":[1,2,3,4,5],"nested":{"key":"value"}}"#.repeat(100)),
    ]
}

fn create_test_bencode_data() -> Vec<(String, BencodeValue)> {
    let mut torrent_info = HashMap::new();
    torrent_info.insert(b"name".to_vec(), BencodeValue::string("example.mkv"));
    torrent_info.insert(b"length".to_vec(), BencodeValue::integer(1073741824)); // 1GB
    torrent_info.insert(b"piece length".to_vec(), BencodeValue::integer(262144)); // 256KB
    torrent_info.insert(b"pieces".to_vec(), BencodeValue::byte_string(vec![0u8; 8000])); // Mock hashes
    
    let mut torrent = HashMap::new();
    torrent.insert(b"announce".to_vec(), BencodeValue::string("http://tracker.example.com/announce"));
    torrent.insert(b"info".to_vec(), BencodeValue::dictionary(torrent_info));
    torrent.insert(b"creation date".to_vec(), BencodeValue::integer(1640995200));
    torrent.insert(b"created by".to_vec(), BencodeValue::string("bencode-cli 1.0.0"));
    
    let announce_list = BencodeValue::list(vec![
        BencodeValue::list(vec![BencodeValue::string("http://tracker1.example.com/announce")]),
        BencodeValue::list(vec![BencodeValue::string("http://tracker2.example.com/announce")]),
    ]);
    torrent.insert(b"announce-list".to_vec(), announce_list);

    vec![
        ("Simple Integer".to_string(), BencodeValue::integer(42)),
        ("Simple String".to_string(), BencodeValue::string("Hello, Bencode!")),
        ("List of Integers".to_string(), BencodeValue::list(
            (1..=100).map(|i| BencodeValue::integer(i)).collect()
        )),
        ("Complex Torrent".to_string(), BencodeValue::dictionary(torrent)),
        ("Nested Structure".to_string(), BencodeValue::list(vec![
            BencodeValue::dictionary({
                let mut dict = HashMap::new();
                dict.insert(b"type".to_vec(), BencodeValue::string("movie"));
                dict.insert(b"rating".to_vec(), BencodeValue::integer(8));
                dict.insert(b"tags".to_vec(), BencodeValue::list(vec![
                    BencodeValue::string("action"),
                    BencodeValue::string("adventure"),
                ]));
                dict
            }),
            BencodeValue::dictionary({
                let mut dict = HashMap::new();
                dict.insert(b"type".to_vec(), BencodeValue::string("series"));
                dict.insert(b"episodes".to_vec(), BencodeValue::integer(24));
                dict
            }),
        ])),
    ]
}

fn bench_text_compression(c: &mut Criterion) {
    let test_texts = create_test_texts();
    
    let mut group = c.benchmark_group("text_compression");
    group.measurement_time(Duration::from_secs(15));
    
    for (name, text) in test_texts {
        let text_size = text.len();
        group.throughput(Throughput::Bytes(text_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("encode", &name),
            &text,
            |b, text| {
                b.iter(|| {
                    let _compressed = TcfCodec::encode(black_box(text));
                })
            },
        );
        
        // Pre-encode for decode benchmark
        if let Ok(compressed) = TcfCodec::encode(&text) {
            group.throughput(Throughput::Bytes(compressed.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("decode", &name),
                &compressed,
                |b, compressed| {
                    b.iter(|| {
                        let _decompressed = TcfCodec::decode(black_box(compressed));
                    })
                },
            );
            
            // Report compression statistics
            println!("📊 Text Compression Stats for {}: Original: {} bytes, Compressed: {} bytes, Ratio: {:.2}x", 
                name, text_size, compressed.len(), text_size as f64 / compressed.len() as f64);
        }
    }
    
    group.finish();
}

fn bench_bencode_operations(c: &mut Criterion) {
    let test_data = create_test_bencode_data();
    
    let mut group = c.benchmark_group("bencode_operations");
    group.measurement_time(Duration::from_secs(10));
    
    for (name, data) in test_data {
        let estimated_size = data.encoded_size();
        group.throughput(Throughput::Bytes(estimated_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("encode", &name),
            &data,
            |b, data| {
                b.iter(|| {
                    let _encoded = BencodeCodec::encode(black_box(data));
                })
            },
        );
        
        // Pre-encode for decode benchmark
        if let Ok(encoded) = BencodeCodec::encode(&data) {
            group.throughput(Throughput::Bytes(encoded.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("decode", &name),
                &encoded,
                |b, encoded| {
                    b.iter(|| {
                        let _decoded = BencodeCodec::decode(black_box(encoded));
                    })
                },
            );
            
            println!("📊 Bencode Stats for {}: Size: {} bytes, Efficiency: {:.1}%", 
                name, encoded.len(), 
                (estimated_size as f64 / encoded.len() as f64) * 100.0
            );
        }
    }
    
    group.finish();
}

fn bench_text_sizes(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let base_text = "The quick brown fox jumps over the lazy dog. This is a sample text for compression benchmarking. ";
    
    let mut group = c.benchmark_group("text_size_scaling");
    
    for size in sizes {
        let text = base_text.repeat(size / base_text.len() + 1);
        let text = &text[..size.min(text.len())];
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("encode", size),
            text,
            |b, text| {
                b.iter(|| {
                    let _compressed = TcfCodec::encode(black_box(text));
                })
            },
        );
        
        // Also test decode performance
        if let Ok(compressed) = TcfCodec::encode(text) {
            group.bench_with_input(
                BenchmarkId::new("decode", size),
                &compressed,
                |b, compressed| {
                    b.iter(|| {
                        let _decompressed = TcfCodec::decode(black_box(compressed));
                    })
                },
            );
        }
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let large_text = "A".repeat(1_000_000); // 1MB of 'A's
    let random_text = (0..1_000_000)
        .map(|i| ((i % 26) as u8 + b'a') as char)
        .collect::<String>();
    
    let mut group = c.benchmark_group("memory_efficiency");
    group.sample_size(10); // Fewer samples for large data
    
    for (name, text) in vec![
        ("Large_Repetitive", large_text),
        ("Large_Random", random_text),
    ] {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("encode", &name),
            &text,
            |b, text| {
                b.iter(|| {
                    let _compressed = TcfCodec::encode(black_box(text));
                })
            },
        );
        
        // Measure peak memory usage and compression ratio
        if let Ok(compressed) = TcfCodec::encode(&text) {
            let ratio = text.len() as f64 / compressed.len() as f64;
            println!("📊 Memory Efficiency for {}: {}KB -> {}KB ({}x compression)", 
                name, text.len() / 1024, compressed.len() / 1024, ratio);
        }
    }
    
    group.finish();
}

fn bench_codec_comparison(c: &mut Criterion) {
    let test_text = "The quick brown fox jumps over the lazy dog. ".repeat(1000);
    
    // Convert to BencodeValue for comparison
    let bencode_data = BencodeValue::string(&test_text);
    
    let mut group = c.benchmark_group("codec_comparison");
    group.throughput(Throughput::Bytes(test_text.len() as u64));
    
    // TCF encoding
    group.bench_function("tcf_encode", |b| {
        b.iter(|| {
            let _compressed = TcfCodec::encode(black_box(&test_text));
        })
    });
    
    // Bencode encoding
    group.bench_function("bencode_encode", |b| {
        b.iter(|| {
            let _encoded = BencodeCodec::encode(black_box(&bencode_data));
        })
    });
    
    // Standard compression for comparison
    group.bench_function("gzip_encode", |b| {
        b.iter(|| {
            use flate2::write::GzEncoder;
            use flate2::Compression;
            use std::io::Write;
            
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            let _ = encoder.write_all(black_box(test_text.as_bytes()));
            let _compressed = encoder.finish();
        })
    });
    
    group.finish();
    
    // Print comparison results
    if let (Ok(tcf_compressed), Ok(bencode_encoded)) = (
        TcfCodec::encode(&test_text),
        BencodeCodec::encode(&bencode_data)
    ) {
        println!("\n🏆 Codec Comparison Results:");
        println!("📄 Original size: {} bytes", test_text.len());
        println!("🔤 TCF compressed: {} bytes ({:.2}x)", 
            tcf_compressed.len(), test_text.len() as f64 / tcf_compressed.len() as f64);
        println!("📦 Bencode encoded: {} bytes ({:.2}x)", 
            bencode_encoded.len(), test_text.len() as f64 / bencode_encoded.len() as f64);
    }
}

fn bench_real_world_data(c: &mut Criterion) {
    // Try to load real files if available
    let real_data = vec![
        ("Cargo.toml", include_str!("../Cargo.toml")),
        ("Source Code", include_str!("../src/main.rs")),
        ("README", include_str!("../README.md")),
    ];
    
    let mut group = c.benchmark_group("real_world_data");
    
    for (name, content) in real_data {
        if content.len() > 100 { // Only benchmark non-empty files
            group.throughput(Throughput::Bytes(content.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("tcf_encode", name),
                content,
                |b, content| {
                    b.iter(|| {
                        let _compressed = TcfCodec::encode(black_box(content));
                    })
                },
            );
            
            let bencode_data = BencodeValue::string(content);
            group.bench_with_input(
                BenchmarkId::new("bencode_encode", name),
                &bencode_data,
                |b, data| {
                    b.iter(|| {
                        let _encoded = BencodeCodec::encode(black_box(data));
                    })
                },
            );
        }
    }
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_text_compression, 
    bench_bencode_operations,
    bench_text_sizes,
    bench_memory_usage,
    bench_codec_comparison,
    bench_real_world_data
);
criterion_main!(benches);