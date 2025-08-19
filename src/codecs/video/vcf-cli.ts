#!/usr/bin/env node

/**
 * VCF (Video Codec Format) CLI Tool
 * 
 * Command-line interface for encoding and decoding videos using the VCF format.
 */

import * as fs from 'fs';
import * as path from 'path';
import { VideoCodec } from './vcf-codec';

function showUsage(): void {
  console.log(`
VCF Video Codec CLI

Usage:
  vcf encode <input.mp4> <output.vcf> [options]  - Encode video to VCF format
  vcf decode <input.vcf> <output.mp4>            - Decode VCF file to video
  vcf info <input.vcf>                           - Show VCF file information
  vcf help                                       - Show this help message

Options for encode:
  --quality <1-100>     - Compression quality (default: 85)
  --bitrate <number>    - Target bitrate in bps (default: 1000000)
  --gop <number>        - GOP size (I-frame interval, default: 30)

Examples:
  vcf encode video.mp4 video.vcf --quality 90 --gop 60
  vcf decode video.vcf video_decoded.mp4
  vcf info video.vcf
`);
}

async function encodeVideo(inputPath: string, outputPath: string, options: {
  quality?: number;
  bitrate?: number;
  gopSize?: number;
} = {}): Promise<void> {
  try {
    console.log(`Encoding ${inputPath} to ${outputPath}...`);
    
    // Check if input file exists
    if (!fs.existsSync(inputPath)) {
      throw new Error(`Input file not found: ${inputPath}`);
    }
    
    // Check for FFmpeg availability
    await checkFFmpegAvailability();
    
    const startTime = Date.now();
    
    // Get original file size
    const originalStats = fs.statSync(inputPath);
    const originalSize = originalStats.size;
    
    // Encode to VCF
    await VideoCodec.encode(inputPath, outputPath, options);
    
    const endTime = Date.now();
    const encodingTime = (endTime - startTime) / 1000;
    
    const compressedStats = fs.statSync(outputPath);
    const compressedSize = compressedStats.size;
    const compressionRatio = originalSize / compressedSize;
    const savings = ((originalSize - compressedSize) / originalSize) * 100;
    
    console.log(`✓ Video encoding completed successfully!`);
    console.log(`  Encoding time: ${encodingTime.toFixed(2)}s`);
    console.log(`  Original size: ${formatBytes(originalSize)}`);
    console.log(`  Compressed size: ${formatBytes(compressedSize)}`);
    console.log(`  Compression ratio: ${compressionRatio.toFixed(2)}:1`);
    console.log(`  Space savings: ${savings.toFixed(1)}%`);
    
  } catch (error: any) {
    console.error(`✗ Encoding failed: ${error.message}`);
    process.exit(1);
  }
}

async function decodeVideo(inputPath: string, outputPath: string): Promise<void> {
  try {
    console.log(`Decoding ${inputPath} to ${outputPath}...`);
    
    // Check if input file exists
    if (!fs.existsSync(inputPath)) {
      throw new Error(`Input file not found: ${inputPath}`);
    }
    
    // Check for FFmpeg availability
    await checkFFmpegAvailability();
    
    const startTime = Date.now();
    
    // Decode VCF
    await VideoCodec.decode(inputPath, outputPath);
    
    const endTime = Date.now();
    const decodingTime = (endTime - startTime) / 1000;
    
    const outputStats = fs.statSync(outputPath);
    
    console.log(`✓ Video decoding completed successfully!`);
    console.log(`  Decoding time: ${decodingTime.toFixed(2)}s`);
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
    
    // Read VCF file
    const vcfData = fs.readFileSync(inputPath);
    
    // Parse header to get information
    const { header } = parseVCFHeader(vcfData);
    
    console.log(`\nVCF File Information:`);
    console.log(`  Format: ${header.magic} v${header.version}`);
    console.log(`  Resolution: ${header.width}x${header.height}`);
    console.log(`  Frame Rate: ${header.fps} fps`);
    console.log(`  Duration: ${header.duration.toFixed(2)}s`);
    console.log(`  Frame Count: ${header.frameCount}`);
    console.log(`  GOP Size: ${header.gopSize}`);
    console.log(`  Quality: ${header.quality}%`);
    console.log(`  Target Bitrate: ${formatBitrate(header.bitrate)}`);
    console.log(`  Compressed Size: ${formatBytes(header.compressedSize)}`);
    console.log(`  File Size: ${formatBytes(vcfData.length)}`);
    console.log(`  Checksum: ${header.checksum.substring(0, 16)}...`);
    
    // Calculate statistics
    const estimatedOriginalSize = header.width * header.height * 3 * header.frameCount; // RGB estimate
    const compressionRatio = estimatedOriginalSize / header.compressedSize;
    const actualBitrate = (vcfData.length * 8) / header.duration;
    
    console.log(`\nCompression Statistics:`);
    console.log(`  Estimated raw size: ${formatBytes(estimatedOriginalSize)}`);
    console.log(`  Compression ratio: ${compressionRatio.toFixed(2)}:1`);
    console.log(`  Actual bitrate: ${formatBitrate(actualBitrate)}`);
    console.log(`  I-frames: ~${Math.ceil(header.frameCount / header.gopSize)}`);
    console.log(`  P-frames: ~${header.frameCount - Math.ceil(header.frameCount / header.gopSize)}`);
    
  } catch (error: any) {
    console.error(`✗ Analysis failed: ${error.message}`);
    process.exit(1);
  }
}

function parseVCFHeader(vcfData: Buffer): { header: any } {
  if (vcfData.length < 8) {
    throw new Error('Invalid VCF file: too small');
  }
  
  const magic = vcfData.subarray(0, 4).toString('ascii');
  if (magic !== 'VCF1') {
    throw new Error('Invalid VCF magic number');
  }
  
  const headerSize = vcfData.readUInt32LE(4);
  if (vcfData.length < 8 + headerSize) {
    throw new Error('Invalid VCF file: header size mismatch');
  }
  
  const headerData = vcfData.subarray(8, 8 + headerSize);
  const header = JSON.parse(headerData.toString('utf8'));
  
  return { header };
}

async function checkFFmpegAvailability(): Promise<void> {
  return new Promise((resolve, reject) => {
    const { spawn } = require('child_process');
    const ffmpeg = spawn('ffmpeg', ['-version']);
    
    ffmpeg.on('close', (code: number) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error('FFmpeg not found. Please install FFmpeg to use VCF codec.'));
      }
    });
    
    ffmpeg.on('error', () => {
      reject(new Error('FFmpeg not found. Please install FFmpeg to use VCF codec.'));
    });
  });
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function formatBitrate(bitrate: number): string {
  if (bitrate === 0) return '0 bps';
  
  const k = 1000;
  const sizes = ['bps', 'Kbps', 'Mbps', 'Gbps'];
  const i = Math.floor(Math.log(bitrate) / Math.log(k));
  
  return parseFloat((bitrate / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function parseOptions(args: string[]): {
  quality?: number;
  bitrate?: number;
  gopSize?: number;
  remainingArgs: string[];
} {
  const options: any = {};
  const remainingArgs: string[] = [];
  
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    
    if (arg === '--quality' && i + 1 < args.length) {
      const quality = parseInt(args[i + 1], 10);
      if (isNaN(quality) || quality < 1 || quality > 100) {
        throw new Error('Quality must be a number between 1 and 100');
      }
      options.quality = quality;
      i++; // Skip next argument
    } else if (arg === '--bitrate' && i + 1 < args.length) {
      const bitrate = parseInt(args[i + 1], 10);
      if (isNaN(bitrate) || bitrate < 1) {
        throw new Error('Bitrate must be a positive number');
      }
      options.bitrate = bitrate;
      i++; // Skip next argument
    } else if (arg === '--gop' && i + 1 < args.length) {
      const gopSize = parseInt(args[i + 1], 10);
      if (isNaN(gopSize) || gopSize < 1) {
        throw new Error('GOP size must be a positive number');
      }
      options.gopSize = gopSize;
      i++; // Skip next argument
    } else if (!arg.startsWith('--')) {
      remainingArgs.push(arg);
    } else {
      throw new Error(`Unknown option: ${arg}`);
    }
  }
  
  return { ...options, remainingArgs };
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
      case 'encode': {
        const { remainingArgs, ...options } = parseOptions(args.slice(1));
        
        if (remainingArgs.length !== 2) {
          console.error('Error: encode command requires input and output file paths');
          showUsage();
          process.exit(1);
        }
        
        await encodeVideo(remainingArgs[0], remainingArgs[1], options);
        break;
      }
      
      case 'decode':
        if (args.length !== 3) {
          console.error('Error: decode command requires input and output file paths');
          showUsage();
          process.exit(1);
        }
        await decodeVideo(args[1], args[2]);
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