use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use codec_cdn_rust::codecs::text::{TcfCodec, TextCompressionStats};
use std::fs;
use std::time::Duration;

fn create_test_texts() -> Vec<(String, String)> {
    vec![
        ("Small".to_string(), "Hello, World!".to_string()),
        ("Medium".to_string(), "The quick brown fox jumps over the lazy dog. ".repeat(50)),
        ("Large".to_string(), "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(200)),
        ("Repetitive".to_string(), "aaaaaaaaaa".repeat(100)),
        ("Random".to_string(), "abcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()".repeat(25)),
    ]
}

fn bench_text_compression(c: &mut Criterion) {
    let test_texts = create_test_texts();
    
    let mut group = c.benchmark_group("text_compression");
    group.measurement_time(Duration::from_secs(10));
    
    for (name, text) in test_texts {
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
            let stats = TcfCodec::get_stats(&text, &compressed);
            println!("Text Compression Stats for {}: {}", name, stats);
        }
    }
    
    group.finish();
}

fn bench_text_sizes(c: &mut Criterion) {
    let sizes = vec![100, 1000, 10000, 100000];
    let base_text = "The quick brown fox jumps over the lazy dog. ";
    
    let mut group = c.benchmark_group("text_size_scaling");
    
    for size in sizes {
        let text = base_text.repeat(size / base_text.len() + 1);
        let text = &text[..size.min(text.len())];
        
        group.bench_with_input(
            BenchmarkId::new("encode", size),
            text,
            |b, text| {
                b.iter(|| {
                    let _compressed = TcfCodec::encode(black_box(text));
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_text_compression, bench_text_sizes);
criterion_main!(benches);