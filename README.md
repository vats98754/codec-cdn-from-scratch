# codec-cdn-from-scratch

A custom codec and streaming platform implementation with TCF (Text Codec Format), ICF (Image Codec Format), and VCF (Video Codec Format), plus a CDN for streaming.

## Overview

This project implements a complete codec and content delivery network from scratch, following a phased approach:

1. **Phase 1 - Text Codec (TCF)** ✅ *In Progress*
2. **Phase 2 - Image Codec (ICF)** 🔄 *Planned*
3. **Phase 3 - Video Codec (VCF)** 🔄 *Planned*
4. **Phase 4 - Segmented Packaging & Streaming** 🔄 *Planned*
5. **Phase 5 - CDN** 🔄 *Planned*
6. **Phase 6 - Player** 🔄 *Planned*
7. **Phase 7 - Tooling & QA** 🔄 *Planned*

## Phase 1 - Text Codec Format (TCF)

The TCF implementation includes:

- **Unicode Normalization**: NFC normalization for consistent text handling
- **Range Coding**: Entropy coding for compression
- **Custom File Format**: TCF format with magic bytes, headers, and chunk structure
- **CLI Tools**: Command-line encoder and decoder

### TCF File Format

```
+------------------+
| TCF Header       | Magic "TCF1", version, offsets
+------------------+
| Model Parameters | PPM/CM configuration
+------------------+
| Chunk Table      | Index of data chunks
+------------------+
| Data Chunks      | Compressed text data
+------------------+
| Footer           | Checksums and validation
+------------------+
```

### Features Implemented

- ✅ Range coder for entropy compression
- ✅ TCF file format with proper headers
- ✅ Unicode text normalization (NFC)
- ✅ Basic text modeling infrastructure
- ✅ CLI encoder/decoder tools
- ✅ Integration tests and benchmarks
- 🔄 PPM/CM modeling (basic version implemented)
- 🔄 Dictionary support for long-range repetition

## Installation & Usage

### Building

```bash
cargo build --release
```

### Text Encoding/Decoding

```bash
# Encode text file to TCF format
./target/release/tcf-cli encode --input text.txt --output text.tcf

# Decode TCF file back to text
./target/release/tcf-cli decode --input text.tcf --output decoded.txt

# Show file information
./target/release/tcf-cli info --input text.tcf
```

### Advanced Options

```bash
# Encode with custom parameters
./target/release/tcf-cli encode \
    --input text.txt \
    --output text.tcf \
    --max-order 6 \
    --use-escape
```

## Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Test with sample data
echo "Hello, World!" > sample.txt
./target/release/tcf-cli encode --input sample.txt --output sample.tcf
./target/release/tcf-cli decode --input sample.tcf --output decoded.txt
diff sample.txt decoded.txt
```

## Project Structure

```
codec-cdn-from-scratch/
├── crates/
│   ├── common/          # Shared utilities and error types
│   ├── entropy/         # Range coder implementation
│   ├── tcf/            # Text Codec Format implementation
│   └── cli/            # Command-line interface
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
└── src/                # Main binary crates
```

## Architecture

### Range Coder

The entropy coder uses arithmetic coding principles:
- 32-bit precision arithmetic
- Adaptive frequency models
- Overflow protection
- EOF handling

### Text Model

Basic predictive modeling:
- Unicode normalization (NFC)
- Character-level prediction
- Context-based frequency estimation
- Escape symbol handling

## Roadmap

### Milestone 1 (Current) - TCF v0
- ✅ Basic range coder
- ✅ TCF file format
- ✅ CLI tools
- 🔄 Fix decoder synchronization issues
- 🔄 Improve compression ratios

### Milestone 2 - ICF v0 (Next)
- Integer DCT/wavelet transforms
- Intra prediction modes
- ICF file format
- PSNR/SSIM quality tools

### Milestone 3 - VCF v0
- Motion compensation
- I/P frame structure
- VCF format with GOP alignment
- Basic rate control

### Milestone 4 - CDN MVP
- 2-tier caching
- HTTP/2 transport
- Basic observability
- Manifest format

## Contributing

This is a learning project implementing codec fundamentals. Key areas for contribution:

1. **Performance optimization** - SIMD, threading, cache efficiency
2. **Algorithm improvements** - Better prediction, entropy coding
3. **Testing** - More comprehensive test suites
4. **Documentation** - Better API docs and examples

## Patent Considerations

This implementation avoids modern patented techniques by:
- Using arithmetic/range coding (older, well-established)
- Simple transform techniques (DCT, basic wavelets)
- Basic prediction modes
- No advanced features like CABAC, advanced in-loop filters

## License

MIT License - see LICENSE file for details.
