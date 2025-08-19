# Quick Start Guide

## Building the Project

```bash
# Clone the repository
git clone https://github.com/vats98754/codec-cdn-from-scratch.git
cd codec-cdn-from-scratch

# Build the project
cargo build --release

# Run tests
cargo test
```

## Basic Usage

### Encoding Text

```bash
# Create a sample text file
echo "Hello, World! This is a test of the TCF codec." > sample.txt

# Encode to TCF format
./target/release/tcf-cli encode --input sample.txt --output sample.tcf

# Check the file sizes
ls -la sample.*
```

### Decoding Text

```bash
# Decode back to text
./target/release/tcf-cli decode --input sample.tcf --output decoded.txt

# Compare files (currently fails due to synchronization issues)
diff sample.txt decoded.txt
```

### Advanced Options

```bash
# Encode with higher order model
./target/release/tcf-cli encode \
    --input sample.txt \
    --output sample.tcf \
    --max-order 6 \
    --use-escape

# Show file information
./target/release/tcf-cli info --input sample.tcf
```

## Testing

```bash
# Run all tests (some currently fail)
cargo test

# Run benchmarks
cargo bench

# Test specific components
cargo test test_empty_text  # This passes
cargo test test_tcf_roundtrip  # This currently fails
```

## Current Limitations

- Decoder synchronization issues prevent proper round-trip encoding
- Only ASCII text is properly supported
- Compression ratios are not optimized yet
- No dictionary support for repetitive content

## Next Steps

See IMPLEMENTATION.md for current status and planned improvements.