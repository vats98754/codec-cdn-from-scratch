use criterion::{black_box, criterion_group, criterion_main, Criterion};
use codec_tcf::{TcfEncoder, ModelParams};
use std::io::Cursor;

fn benchmark_tcf_encoding(c: &mut Criterion) {
    let test_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    
    c.bench_function("tcf_encode", |b| {
        b.iter(|| {
            let mut encoded_data = Vec::new();
            let model_params = ModelParams::default();
            let mut encoder = TcfEncoder::new(Cursor::new(&mut encoded_data), model_params);
            encoder.encode(black_box(&test_text)).unwrap();
        });
    });
}

fn benchmark_range_coder(c: &mut Criterion) {
    c.bench_function("range_encoder", |b| {
        b.iter(|| {
            let mut data = Vec::new();
            let mut encoder = codec_entropy::RangeEncoder::new(&mut data);
            
            for i in 0..1000 {
                encoder.encode_symbol(1, i % 10, 10).unwrap();
            }
            encoder.finish().unwrap();
        });
    });
}

criterion_group!(benches, benchmark_tcf_encoding, benchmark_range_coder);
criterion_main!(benches);