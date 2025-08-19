#!/usr/bin/env node

/**
 * ICF (Image Codec Format) CLI Tool
 * 
 * Command-line interface for encoding and decoding images using the ICF format.
 */

import * as fs from 'fs';
import * as path from 'path';
import sharp from 'sharp';
import { ImageCodec } from './icf-codec';

function showUsage(): void {
  console.log(`
ICF Image Codec CLI

Usage:
  icf encode <input.jpg|png> <output.icf> [quality]  - Encode image to ICF format
  icf decode <input.icf> <output.jpg|png>            - Decode ICF file to image
  icf info <input.icf>                               - Show ICF file information
  icf help                                           - Show this help message

Parameters:
  quality    - Compression quality (1-100, default: 85)

Examples:
  icf encode photo.jpg photo.icf 90
  icf decode photo.icf photo_decoded.jpg
  icf info photo.icf
`);
}

async function encodeImage(inputPath: string, outputPath: string, quality: number = 85): Promise<void> {
  try {
    console.log(`Encoding ${inputPath} to ${outputPath} (quality: ${quality})...`);
    
    // Check if input file exists
    if (!fs.existsSync(inputPath)) {
      throw new Error(`Input file not found: ${inputPath}`);
    }
    
    // Get original file size
    const originalStats = fs.statSync(inputPath);
    const originalSize = originalStats.size;
    
    // Encode to ICF
    const icfData = await ImageCodec.encode(inputPath, quality);
    
    // Write output file
    fs.writeFileSync(outputPath, icfData);
    
    const compressedSize = icfData.length;
    const compressionRatio = originalSize / compressedSize;
    const savings = ((originalSize - compressedSize) / originalSize) * 100;
    
    console.log(`✓ Encoding completed successfully!`);
    console.log(`  Original size: ${formatBytes(originalSize)}`);
    console.log(`  Compressed size: ${formatBytes(compressedSize)}`);
    console.log(`  Compression ratio: ${compressionRatio.toFixed(2)}:1`);
    console.log(`  Space savings: ${savings.toFixed(1)}%`);
    
  } catch (error: any) {
    console.error(`✗ Encoding failed: ${error.message}`);
    process.exit(1);
  }
}

async function decodeImage(inputPath: string, outputPath: string): Promise<void> {
  try {
    console.log(`Decoding ${inputPath} to ${outputPath}...`);
    
    // Check if input file exists
    if (!fs.existsSync(inputPath)) {
      throw new Error(`Input file not found: ${inputPath}`);
    }
    
    // Read ICF file
    const icfData = fs.readFileSync(inputPath);
    
    // Decode ICF data
    const { data, width, height, channels } = await ImageCodec.decode(icfData);
    
    // Determine output format from extension
    const ext = path.extname(outputPath).toLowerCase();
    let format: 'jpeg' | 'png' | 'webp' = 'jpeg';
    
    switch (ext) {
      case '.png':
        format = 'png';
        break;
      case '.webp':
        format = 'webp';
        break;
      case '.jpg':
      case '.jpeg':
      default:
        format = 'jpeg';
        break;
    }
    
    // Create Sharp image from raw data
    await sharp(data, {
      raw: {
        width,
        height,
        channels: channels as 1 | 2 | 3 | 4
      }
    })
    .toFormat(format)
    .toFile(outputPath);
    
    const outputStats = fs.statSync(outputPath);
    
    console.log(`✓ Decoding completed successfully!`);
    console.log(`  Image dimensions: ${width}x${height}`);
    console.log(`  Channels: ${channels}`);
    console.log(`  Output size: ${formatBytes(outputStats.size)}`);
    
  } catch (error: any) {
    console.error(`✗ Decoding failed: ${error.message}`);
    process.exit(1);
  }
}

async function showFileInfo(inputPath: string): Promise<void> {
  try {
    console.log(`Analyzing ${inputPath}...`);
    
    // Check if input file exists
    if (!fs.existsSync(inputPath)) {
      throw new Error(`Input file not found: ${inputPath}`);
    }
    
    // Read ICF file
    const icfData = fs.readFileSync(inputPath);
    
    // Parse header to get information
    const { header } = parseICFHeader(icfData);
    
    console.log(`\nICF File Information:`);
    console.log(`  Format: ${header.magic} v${header.version}`);
    console.log(`  Dimensions: ${header.width}x${header.height}`);
    console.log(`  Channels: ${header.channels}`);
    console.log(`  Color Space: ${header.colorSpace}`);
    console.log(`  Quality: ${header.quality}%`);
    console.log(`  Compressed Size: ${formatBytes(header.compressedSize)}`);
    console.log(`  File Size: ${formatBytes(icfData.length)}`);
    console.log(`  Checksum: ${header.checksum.substring(0, 16)}...`);
    
    // Calculate estimated original size
    const estimatedOriginalSize = header.width * header.height * header.channels;
    const compressionRatio = estimatedOriginalSize / header.compressedSize;
    
    console.log(`\nCompression Statistics:`);
    console.log(`  Estimated original: ${formatBytes(estimatedOriginalSize)}`);
    console.log(`  Compression ratio: ${compressionRatio.toFixed(2)}:1`);
    
  } catch (error: any) {
    console.error(`✗ Analysis failed: ${error.message}`);
    process.exit(1);
  }
}

function parseICFHeader(icfData: Buffer): { header: any } {
  if (icfData.length < 8) {
    throw new Error('Invalid ICF file: too small');
  }
  
  const magic = icfData.subarray(0, 4).toString('ascii');
  if (magic !== 'ICF1') {
    throw new Error('Invalid ICF magic number');
  }
  
  const headerSize = icfData.readUInt32LE(4);
  if (icfData.length < 8 + headerSize) {
    throw new Error('Invalid ICF file: header size mismatch');
  }
  
  const headerData = icfData.subarray(8, 8 + headerSize);
  const header = JSON.parse(headerData.toString('utf8'));
  
  return { header };
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function validateQuality(quality: string): number {
  const q = parseInt(quality, 10);
  if (isNaN(q) || q < 1 || q > 100) {
    throw new Error('Quality must be a number between 1 and 100');
  }
  return q;
}

// Main CLI logic
async function main(): Promise<void> {
  const args = process.argv.slice(2);
  
  if (args.length === 0 || args[0] === 'help') {
    showUsage();
    return;
  }
  
  const command = args[0];
  
  try {
    switch (command) {
      case 'encode':
        if (args.length < 3) {
          console.error('Error: encode command requires input and output file paths');
          showUsage();
          process.exit(1);
        }
        
        const quality = args[3] ? validateQuality(args[3]) : 85;
        await encodeImage(args[1], args[2], quality);
        break;
        
      case 'decode':
        if (args.length !== 3) {
          console.error('Error: decode command requires input and output file paths');
          showUsage();
          process.exit(1);
        }
        await decodeImage(args[1], args[2]);
        break;
        
      case 'info':
        if (args.length !== 2) {
          console.error('Error: info command requires input file path');
          showUsage();
          process.exit(1);
        }
        await showFileInfo(args[1]);
        break;
        
      default:
        console.error(`Error: Unknown command '${command}'`);
        showUsage();
        process.exit(1);
    }
  } catch (error: any) {
    console.error(`Error: ${error.message}`);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}