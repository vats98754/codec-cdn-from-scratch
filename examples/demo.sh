#!/bin/bash

# TCF Codec Demo Script
# Demonstrates the basic functionality of the Text Codec Format

echo "=== TCF Codec Demo ==="
echo

# Build the project
echo "Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Build successful!"
echo

# Create test file
echo "Creating sample text file..."
cat > demo.txt << 'EOF'
Hello, World!

This is a demonstration of the TCF (Text Codec Format) implementation.
The codec includes:
- Unicode normalization (NFC)
- Range coding for entropy compression
- Custom file format with headers and checksums
- Command-line tools for encoding and decoding

Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
EOF

echo "Sample file created with $(wc -c < demo.txt) bytes"
echo

# Encode the file
echo "Encoding to TCF format..."
./target/release/tcf-cli encode --input demo.txt --output demo.tcf

if [ $? -eq 0 ]; then
    echo "Encoding successful!"
    echo "Original size: $(wc -c < demo.txt) bytes"
    echo "Compressed size: $(wc -c < demo.tcf) bytes"
    echo "Compression ratio: $(echo "scale=2; $(wc -c < demo.tcf) / $(wc -c < demo.txt)" | bc)"
else
    echo "Encoding failed!"
    exit 1
fi

echo

# Decode the file
echo "Decoding from TCF format..."
./target/release/tcf-cli decode --input demo.tcf --output decoded.txt

if [ $? -eq 0 ]; then
    echo "Decoding completed!"
    echo "Note: Currently there are synchronization issues between encoder/decoder"
    echo "This is a known issue being worked on."
else
    echo "Decoding failed!"
    exit 1
fi

echo

# Compare files
echo "Comparing original and decoded files..."
if diff demo.txt decoded.txt > /dev/null; then
    echo "✅ Files are identical - perfect round-trip!"
else
    echo "❌ Files differ - decoder synchronization issue (known problem)"
    echo "First few characters comparison:"
    echo "Original: $(head -c 50 demo.txt)"
    echo "Decoded:  $(head -c 50 decoded.txt)"
fi

echo
echo "Demo completed!"
echo "Files created: demo.txt, demo.tcf, decoded.txt"