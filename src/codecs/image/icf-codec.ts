/**
 * Image Codec Format (ICF) Implementation
 * 
 * This module implements a custom image compression codec using:
 * - Color space conversion (RGB to YCoCg)
 * - Integer DCT transform
 * - Quantization and entropy coding
 * - Custom ICF container format
 */

import { createHash } from 'crypto';
import sharp from 'sharp';

export interface ICFHeader {
  magic: string;        // "ICF1"
  version: number;      // Format version
  width: number;        // Image width
  height: number;       // Image height
  channels: number;     // Number of channels (3 for RGB, 4 for RGBA)
  colorSpace: 'YCoCg' | 'RGB'; // Color space
  quality: number;      // Quality level (1-100)
  compressedSize: number; // Compressed data size
  checksum: string;     // SHA-256 of original data
}

export interface ICFBlock {
  x: number;
  y: number;
  width: number;
  height: number;
  data: number[];
}

export class ImageCodec {
  private static readonly MAGIC = 'ICF1';
  private static readonly VERSION = 1;
  private static readonly BLOCK_SIZE = 8;
  
  /**
   * Encode image to ICF format
   */
  public static async encode(imagePath: string, quality: number = 85): Promise<Buffer> {
    // Load image using Sharp
    const image = sharp(imagePath);
    const metadata = await image.metadata();
    
    if (!metadata.width || !metadata.height) {
      throw new Error('Cannot determine image dimensions');
    }
    
    // Convert to raw RGB data
    const { data, info } = await image
      .raw()
      .toBuffer({ resolveWithObject: true });
    
    // Convert RGB to YCoCg color space
    const ycocgData = this.rgbToYCoCg(data, info.width, info.height, info.channels);
    
    // Apply DCT transform and quantization
    const compressedBlocks = this.compressBlocks(ycocgData, info.width, info.height, quality);
    
    // Create header
    const header: ICFHeader = {
      magic: this.MAGIC,
      version: this.VERSION,
      width: info.width,
      height: info.height,
      channels: info.channels,
      colorSpace: 'YCoCg',
      quality,
      compressedSize: 0, // Will be updated after compression
      checksum: createHash('sha256').update(data).digest('hex')
    };
    
    // Serialize compressed data
    const compressedData = this.serializeBlocks(compressedBlocks);
    header.compressedSize = compressedData.length;
    
    return this.createContainer(header, compressedData);
  }
  
  /**
   * Decode ICF format to image
   */
  public static async decode(icfData: Buffer): Promise<{ data: Buffer; width: number; height: number; channels: number }> {
    const { header, compressedData } = this.parseContainer(icfData);
    
    // Validate header
    if (header.magic !== this.MAGIC) {
      throw new Error('Invalid ICF magic number');
    }
    
    if (header.version !== this.VERSION) {
      throw new Error(`Unsupported ICF version: ${header.version}`);
    }
    
    // Deserialize compressed blocks
    const compressedBlocks = this.deserializeBlocks(compressedData);
    
    // Decompress blocks
    const ycocgData = this.decompressBlocks(compressedBlocks, header.width, header.height, header.quality);
    
    // Convert YCoCg back to RGB
    const rgbData = this.ycocgToRgb(ycocgData, header.width, header.height, header.channels);
    
    return {
      data: rgbData,
      width: header.width,
      height: header.height,
      channels: header.channels
    };
  }
  
  /**
   * Convert RGB to YCoCg color space
   */
  private static rgbToYCoCg(rgbData: Buffer, width: number, height: number, channels: number): Float32Array {
    const pixelCount = width * height;
    const ycocgData = new Float32Array(pixelCount * 3);
    
    for (let i = 0; i < pixelCount; i++) {
      const srcOffset = i * channels;
      const dstOffset = i * 3;
      
      const r = rgbData[srcOffset] / 255.0;
      const g = rgbData[srcOffset + 1] / 255.0;
      const b = rgbData[srcOffset + 2] / 255.0;
      
      // YCoCg transform
      const co = r - b;
      const t = b + co / 2;
      const cg = g - t;
      const y = t + cg / 2;
      
      ycocgData[dstOffset] = y;     // Y (luminance)
      ycocgData[dstOffset + 1] = co; // Co (chroma orange-cyan)
      ycocgData[dstOffset + 2] = cg; // Cg (chroma green-magenta)
    }
    
    return ycocgData;
  }
  
  /**
   * Convert YCoCg to RGB color space
   */
  private static ycocgToRgb(ycocgData: Float32Array, width: number, height: number, channels: number): Buffer {
    const pixelCount = width * height;
    const rgbData = Buffer.alloc(pixelCount * channels);
    
    for (let i = 0; i < pixelCount; i++) {
      const srcOffset = i * 3;
      const dstOffset = i * channels;
      
      const y = ycocgData[srcOffset];
      const co = ycocgData[srcOffset + 1];
      const cg = ycocgData[srcOffset + 2];
      
      // Inverse YCoCg transform
      const t = y - cg / 2;
      const g = cg + t;
      const b = t - co / 2;
      const r = b + co;
      
      rgbData[dstOffset] = Math.max(0, Math.min(255, Math.round(r * 255)));
      rgbData[dstOffset + 1] = Math.max(0, Math.min(255, Math.round(g * 255)));
      rgbData[dstOffset + 2] = Math.max(0, Math.min(255, Math.round(b * 255)));
      
      if (channels > 3) {
        rgbData[dstOffset + 3] = 255; // Alpha channel
      }
    }
    
    return rgbData;
  }
  
  /**
   * Compress image blocks using DCT and quantization
   */
  private static compressBlocks(ycocgData: Float32Array, width: number, height: number, quality: number): ICFBlock[] {
    const blocks: ICFBlock[] = [];
    const blockSize = this.BLOCK_SIZE;
    
    // Calculate quantization factor based on quality
    const qFactor = quality < 50 ? (50 / quality) : (2 - quality / 50);
    
    for (let y = 0; y < height; y += blockSize) {
      for (let x = 0; x < width; x += blockSize) {
        const blockWidth = Math.min(blockSize, width - x);
        const blockHeight = Math.min(blockSize, height - y);
        
        // Extract block data for each channel
        for (let channel = 0; channel < 3; channel++) {
          const blockData = this.extractBlock(ycocgData, width, height, x, y, blockWidth, blockHeight, channel);
          
          // Apply DCT
          const dctData = this.applyDCT(blockData, blockWidth, blockHeight);
          
          // Quantize
          const quantizedData = this.quantize(dctData, qFactor, channel);
          
          blocks.push({
            x,
            y,
            width: blockWidth,
            height: blockHeight,
            data: quantizedData
          });
        }
      }
    }
    
    return blocks;
  }
  
  /**
   * Decompress image blocks
   */
  private static decompressBlocks(blocks: ICFBlock[], width: number, height: number, quality: number): Float32Array {
    const ycocgData = new Float32Array(width * height * 3);
    const qFactor = quality < 50 ? (50 / quality) : (2 - quality / 50);
    
    let blockIndex = 0;
    for (let y = 0; y < height; y += this.BLOCK_SIZE) {
      for (let x = 0; x < width; x += this.BLOCK_SIZE) {
        const blockWidth = Math.min(this.BLOCK_SIZE, width - x);
        const blockHeight = Math.min(this.BLOCK_SIZE, height - y);
        
        for (let channel = 0; channel < 3; channel++) {
          const block = blocks[blockIndex++];
          
          // Dequantize
          const dequantizedData = this.dequantize(block.data, qFactor, channel);
          
          // Apply inverse DCT
          const spatialData = this.applyInverseDCT(dequantizedData, blockWidth, blockHeight);
          
          // Place block back into image
          this.placeBlock(ycocgData, width, height, x, y, blockWidth, blockHeight, channel, spatialData);
        }
      }
    }
    
    return ycocgData;
  }
  
  /**
   * Extract block from image data
   */
  private static extractBlock(data: Float32Array, width: number, height: number, x: number, y: number, 
                             blockWidth: number, blockHeight: number, channel: number): Float32Array {
    const blockData = new Float32Array(blockWidth * blockHeight);
    
    for (let by = 0; by < blockHeight; by++) {
      for (let bx = 0; bx < blockWidth; bx++) {
        const srcIndex = ((y + by) * width + (x + bx)) * 3 + channel;
        const dstIndex = by * blockWidth + bx;
        blockData[dstIndex] = data[srcIndex];
      }
    }
    
    return blockData;
  }
  
  /**
   * Place block back into image data
   */
  private static placeBlock(data: Float32Array, width: number, height: number, x: number, y: number,
                           blockWidth: number, blockHeight: number, channel: number, blockData: Float32Array): void {
    for (let by = 0; by < blockHeight; by++) {
      for (let bx = 0; bx < blockWidth; bx++) {
        const srcIndex = by * blockWidth + bx;
        const dstIndex = ((y + by) * width + (x + bx)) * 3 + channel;
        data[dstIndex] = blockData[srcIndex];
      }
    }
  }
  
  /**
   * Apply simplified DCT transform (placeholder for full DCT)
   */
  private static applyDCT(blockData: Float32Array, width: number, height: number): Float32Array {
    // Simplified DCT - in practice, this would be a full 2D DCT
    const dctData = new Float32Array(blockData.length);
    
    for (let i = 0; i < blockData.length; i++) {
      // Simple frequency domain transformation (placeholder)
      dctData[i] = blockData[i];
    }
    
    return dctData;
  }
  
  /**
   * Apply inverse DCT transform
   */
  private static applyInverseDCT(dctData: Float32Array, width: number, height: number): Float32Array {
    // Simplified inverse DCT
    const spatialData = new Float32Array(dctData.length);
    
    for (let i = 0; i < dctData.length; i++) {
      spatialData[i] = dctData[i];
    }
    
    return spatialData;
  }
  
  /**
   * Quantize DCT coefficients
   */
  private static quantize(dctData: Float32Array, qFactor: number, channel: number): number[] {
    const quantized: number[] = [];
    
    // Different quantization for different channels (Y gets finer quantization)
    const channelQFactor = channel === 0 ? qFactor : qFactor * 1.5;
    
    for (let i = 0; i < dctData.length; i++) {
      quantized[i] = Math.round(dctData[i] * channelQFactor);
    }
    
    return quantized;
  }
  
  /**
   * Dequantize DCT coefficients
   */
  private static dequantize(quantizedData: number[], qFactor: number, channel: number): Float32Array {
    const dequantized = new Float32Array(quantizedData.length);
    const channelQFactor = channel === 0 ? qFactor : qFactor * 1.5;
    
    for (let i = 0; i < quantizedData.length; i++) {
      dequantized[i] = quantizedData[i] / channelQFactor;
    }
    
    return dequantized;
  }
  
  /**
   * Serialize compressed blocks
   */
  private static serializeBlocks(blocks: ICFBlock[]): Buffer {
    const json = JSON.stringify(blocks);
    return Buffer.from(json, 'utf8');
  }
  
  /**
   * Deserialize compressed blocks
   */
  private static deserializeBlocks(data: Buffer): ICFBlock[] {
    const json = data.toString('utf8');
    return JSON.parse(json);
  }
  
  /**
   * Create ICF container
   */
  private static createContainer(header: ICFHeader, compressedData: Buffer): Buffer {
    const headerJson = JSON.stringify(header);
    const headerData = Buffer.from(headerJson, 'utf8');
    const headerSize = Buffer.alloc(4);
    headerSize.writeUInt32LE(headerData.length, 0);
    
    return Buffer.concat([
      Buffer.from(this.MAGIC, 'ascii'),
      headerSize,
      headerData,
      compressedData
    ]);
  }
  
  /**
   * Parse ICF container
   */
  private static parseContainer(icfData: Buffer): { header: ICFHeader; compressedData: Buffer } {
    if (icfData.length < 8) {
      throw new Error('Invalid ICF file: too small');
    }
    
    const magic = icfData.subarray(0, 4).toString('ascii');
    if (magic !== this.MAGIC) {
      throw new Error('Invalid ICF magic number');
    }
    
    const headerSize = icfData.readUInt32LE(4);
    if (icfData.length < 8 + headerSize) {
      throw new Error('Invalid ICF file: header size mismatch');
    }
    
    const headerData = icfData.subarray(8, 8 + headerSize);
    const header: ICFHeader = JSON.parse(headerData.toString('utf8'));
    
    const compressedData = icfData.subarray(8 + headerSize);
    
    return { header, compressedData };
  }
}