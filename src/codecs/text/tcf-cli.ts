#!/usr/bin/env node

/**
 * TCF (Text Codec Format) CLI Tool
 * 
 * Command-line interface for encoding and decoding text files using the TCF format.
 */

import * as fs from 'fs';
import * as path from 'path';
import { TextCodec } from './tcf-codec';

function showUsage(): void {
  console.log(`
TCF Text Codec CLI

Usage:
  tcf encode <input.txt> <output.tcf>  - Encode text file to TCF format
  tcf decode <input.tcf> <output.txt>  - Decode TCF file to text
  tcf stats <input.txt>                - Show compression statistics
  tcf help                             - Show this help message

Examples:
  tcf encode document.txt document.tcf
  tcf decode document.tcf document_decoded.txt
  tcf stats document.txt
`);
}

function encodeFile(inputPath: string, outputPath: string): void {
  try {
    console.log(`Encoding ${inputPath} to ${outputPath}...`);
    
    const inputText = fs.readFileSync(inputPath, 'utf8');
    const tcfData = TextCodec.encode(inputText);
    
    fs.writeFileSync(outputPath, tcfData);
    
    const stats = TextCodec.getStats(inputText, tcfData);
    console.log(`✓ Encoding completed successfully!`);
    console.log(`  Original size: ${stats.originalSize} bytes`);
    console.log(`  Compressed size: ${stats.compressedSize} bytes`);
    console.log(`  Compression ratio: ${stats.compressionRatio.toFixed(2)}:1`);
    console.log(`  Space savings: ${stats.savings.toFixed(1)}%`);
    
  } catch (error: any) {
    console.error(`✗ Encoding failed: ${error.message}`);
    process.exit(1);
  }
}

function decodeFile(inputPath: string, outputPath: string): void {
  try {
    console.log(`Decoding ${inputPath} to ${outputPath}...`);
    
    const tcfData = fs.readFileSync(inputPath);
    const decodedText = TextCodec.decode(tcfData);
    
    fs.writeFileSync(outputPath, decodedText, 'utf8');
    
    console.log(`✓ Decoding completed successfully!`);
    console.log(`  Output size: ${Buffer.from(decodedText, 'utf8').length} bytes`);
    
  } catch (error: any) {
    console.error(`✗ Decoding failed: ${error.message}`);
    process.exit(1);
  }
}

function showStats(inputPath: string): void {
  try {
    console.log(`Analyzing ${inputPath}...`);
    
    const inputText = fs.readFileSync(inputPath, 'utf8');
    const tcfData = TextCodec.encode(inputText);
    const stats = TextCodec.getStats(inputText, tcfData);
    
    console.log(`\nCompression Statistics:`);
    console.log(`  Original size: ${stats.originalSize} bytes`);
    console.log(`  Compressed size: ${stats.compressedSize} bytes`);
    console.log(`  Compression ratio: ${stats.compressionRatio.toFixed(2)}:1`);
    console.log(`  Space savings: ${stats.savings.toFixed(1)}%`);
    
    // Character analysis
    const charCount = inputText.length;
    const uniqueChars = new Set(inputText).size;
    const entropy = calculateEntropy(inputText);
    
    console.log(`\nText Analysis:`);
    console.log(`  Character count: ${charCount}`);
    console.log(`  Unique characters: ${uniqueChars}`);
    console.log(`  Entropy: ${entropy.toFixed(2)} bits/char`);
    
  } catch (error: any) {
    console.error(`✗ Analysis failed: ${error.message}`);
    process.exit(1);
  }
}

function calculateEntropy(text: string): number {
  const frequencies = new Map<string, number>();
  
  // Count character frequencies
  for (const char of text) {
    frequencies.set(char, (frequencies.get(char) || 0) + 1);
  }
  
  // Calculate entropy
  let entropy = 0;
  const length = text.length;
  
  for (const frequency of frequencies.values()) {
    const probability = frequency / length;
    entropy -= probability * Math.log2(probability);
  }
  
  return entropy;
}

// Main CLI logic
function main(): void {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args[0] === 'help') {
    showUsage();
    return;
  }
  
  const command = args[0];
  
  switch (command) {
    case 'encode':
      if (args.length !== 3) {
        console.error('Error: encode command requires input and output file paths');
        showUsage();
        process.exit(1);
      }
      encodeFile(args[1], args[2]);
      break;
      
    case 'decode':
      if (args.length !== 3) {
        console.error('Error: decode command requires input and output file paths');
        showUsage();
        process.exit(1);
      }
      decodeFile(args[1], args[2]);
      break;
      
    case 'stats':
      if (args.length !== 2) {
        console.error('Error: stats command requires input file path');
        showUsage();
        process.exit(1);
      }
      showStats(args[1]);
      break;
      
    default:
      console.error(`Error: Unknown command '${command}'`);
      showUsage();
      process.exit(1);
  }
}

if (require.main === module) {
  main();
}