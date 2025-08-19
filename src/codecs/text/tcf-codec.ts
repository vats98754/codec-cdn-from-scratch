/**
 * Text Codec Format (TCF) Implementation
 * 
 * This module implements a custom text compression codec using:
 * - Unicode normalization (NFC)
 * - Range coding for entropy compression
 * - Custom TCF container format
 */

import { createHash } from 'crypto';

export interface TCFHeader {
  magic: string;        // "TCF1"
  version: number;      // Format version
  flags: number;        // Compression flags
  originalSize: number; // Original text size in bytes
  compressedSize: number; // Compressed data size
  checksum: string;     // SHA-256 of original data
}

export interface TCFChunk {
  offset: number;
  size: number;
  checksum: string;
  type: 'data' | 'metadata';
}

export class TextCodec {
  private static readonly MAGIC = 'TCF1';
  private static readonly VERSION = 1;
  
  /**
   * Encode text to TCF format
   */
  public static encode(text: string): Buffer {
    // Normalize text using NFC
    const normalizedText = text.normalize('NFC');
    const originalData = Buffer.from(normalizedText, 'utf8');
    
    // Simple compression using deflate (placeholder for custom range coding)
    const compressedData = this.simpleCompress(originalData);
    
    // Create header
    const header: TCFHeader = {
      magic: this.MAGIC,
      version: this.VERSION,
      flags: 0,
      originalSize: originalData.length,
      compressedSize: compressedData.length,
      checksum: createHash('sha256').update(originalData).digest('hex')
    };
    
    // Create container
    return this.createContainer(header, compressedData);
  }
  
  /**
   * Decode TCF format to text
   */
  public static decode(tcfData: Buffer): string {
    const { header, compressedData } = this.parseContainer(tcfData);
    
    // Validate header
    if (header.magic !== this.MAGIC) {
      throw new Error('Invalid TCF magic number');
    }
    
    if (header.version !== this.VERSION) {
      throw new Error(`Unsupported TCF version: ${header.version}`);
    }
    
    // Decompress data
    const decompressedData = this.simpleDecompress(compressedData);
    
    // Validate checksum
    const actualChecksum = createHash('sha256').update(decompressedData).digest('hex');
    if (actualChecksum !== header.checksum) {
      throw new Error('TCF checksum mismatch');
    }
    
    return decompressedData.toString('utf8');
  }
  
  /**
   * Simple compression using RLE and byte frequency (placeholder for range coding)
   */
  private static simpleCompress(data: Buffer): Buffer {
    const compressed: number[] = [];
    
    // Build frequency table
    const frequencies = new Map<number, number>();
    for (const byte of data) {
      frequencies.set(byte, (frequencies.get(byte) || 0) + 1);
    }
    
    // Simple run-length encoding with frequency optimization
    let i = 0;
    while (i < data.length) {
      const currentByte = data[i];
      let runLength = 1;
      
      // Count consecutive identical bytes
      while (i + runLength < data.length && data[i + runLength] === currentByte) {
        runLength++;
      }
      
      if (runLength > 3) {
        // Use RLE for runs > 3
        compressed.push(0xFF, currentByte, runLength);
      } else {
        // Store bytes directly for short runs
        for (let j = 0; j < runLength; j++) {
          compressed.push(currentByte);
        }
      }
      
      i += runLength;
    }
    
    return Buffer.from(compressed);
  }
  
  /**
   * Simple decompression
   */
  private static simpleDecompress(compressedData: Buffer): Buffer {
    const decompressed: number[] = [];
    
    let i = 0;
    while (i < compressedData.length) {
      if (compressedData[i] === 0xFF && i + 2 < compressedData.length) {
        // RLE encoded run
        const byte = compressedData[i + 1];
        const runLength = compressedData[i + 2];
        
        for (let j = 0; j < runLength; j++) {
          decompressed.push(byte);
        }
        
        i += 3;
      } else {
        // Direct byte
        decompressed.push(compressedData[i]);
        i++;
      }
    }
    
    return Buffer.from(decompressed);
  }
  
  /**
   * Create TCF container with header and data
   */
  private static createContainer(header: TCFHeader, compressedData: Buffer): Buffer {
    const headerJson = JSON.stringify(header);
    const headerData = Buffer.from(headerJson, 'utf8');
    const headerSize = Buffer.alloc(4);
    headerSize.writeUInt32LE(headerData.length, 0);
    
    return Buffer.concat([
      Buffer.from(this.MAGIC, 'ascii'),  // 4 bytes magic
      headerSize,                        // 4 bytes header size
      headerData,                        // Variable header data
      compressedData                     // Variable compressed data
    ]);
  }
  
  /**
   * Parse TCF container
   */
  private static parseContainer(tcfData: Buffer): { header: TCFHeader; compressedData: Buffer } {
    if (tcfData.length < 8) {
      throw new Error('Invalid TCF file: too small');
    }
    
    // Read magic
    const magic = tcfData.subarray(0, 4).toString('ascii');
    if (magic !== this.MAGIC) {
      throw new Error('Invalid TCF magic number');
    }
    
    // Read header size
    const headerSize = tcfData.readUInt32LE(4);
    
    if (tcfData.length < 8 + headerSize) {
      throw new Error('Invalid TCF file: header size mismatch');
    }
    
    // Read header
    const headerData = tcfData.subarray(8, 8 + headerSize);
    const header: TCFHeader = JSON.parse(headerData.toString('utf8'));
    
    // Read compressed data
    const compressedData = tcfData.subarray(8 + headerSize);
    
    return { header, compressedData };
  }
  
  /**
   * Get compression statistics
   */
  public static getStats(originalText: string, tcfData: Buffer): {
    originalSize: number;
    compressedSize: number;
    compressionRatio: number;
    savings: number;
  } {
    const originalSize = Buffer.from(originalText, 'utf8').length;
    const compressedSize = tcfData.length;
    
    return {
      originalSize,
      compressedSize,
      compressionRatio: originalSize / compressedSize,
      savings: ((originalSize - compressedSize) / originalSize) * 100
    };
  }
}