/**
 * CDN Server Implementation
 * 
 * This module implements a basic CDN server that can:
 * - Serve static files with caching
 * - Encode/decode text, image, and video files
 * - Stream video content with ABR support
 * - Provide REST API for codec operations
 */

import express from 'express';
import cors from 'cors';
import compression from 'compression';
import helmet from 'helmet';
import multer from 'multer';
import * as fs from 'fs';
import * as path from 'path';
import { TextCodec } from './codecs/text/tcf-codec';
import { ImageCodec } from './codecs/image/icf-codec';
import { VideoCodec } from './codecs/video/vcf-codec';

const app = express();
const PORT = process.env.PORT || 3000;

// Create necessary directories
const UPLOAD_DIR = path.join(__dirname, '../uploads');
const CACHE_DIR = path.join(__dirname, '../cache');
const STATIC_DIR = path.join(__dirname, '../static');

[UPLOAD_DIR, CACHE_DIR, STATIC_DIR].forEach(dir => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
});

// Middleware
app.use(helmet());
app.use(cors());
app.use(compression());
app.use(express.json({ limit: '50mb' }));
app.use(express.urlencoded({ extended: true, limit: '50mb' }));

// Configure multer for file uploads
const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, UPLOAD_DIR);
  },
  filename: (req, file, cb) => {
    const timestamp = Date.now();
    const originalName = file.originalname;
    cb(null, `${timestamp}_${originalName}`);
  }
});

const upload = multer({ 
  storage,
  limits: { fileSize: 100 * 1024 * 1024 } // 100MB limit
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    codecs: ['TCF', 'ICF', 'VCF']
  });
});

// Static file serving with caching
app.use('/static', express.static(STATIC_DIR, {
  maxAge: '1d',
  etag: true,
  lastModified: true
}));

// Serve web interface at root
app.get('/', (req, res) => {
  res.redirect('/static/');
});

// Text Codec API endpoints
app.post('/api/text/encode', upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const textContent = fs.readFileSync(req.file.path, 'utf8');
    const tcfData = TextCodec.encode(textContent);
    
    const outputPath = path.join(CACHE_DIR, `${path.parse(req.file.filename).name}.tcf`);
    fs.writeFileSync(outputPath, tcfData);
    
    const stats = TextCodec.getStats(textContent, tcfData);
    
    // Cleanup uploaded file
    fs.unlinkSync(req.file.path);
    
    res.json({
      success: true,
      outputFile: path.basename(outputPath),
      downloadUrl: `/api/download/${path.basename(outputPath)}`,
      stats
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

app.post('/api/text/decode', upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const tcfData = fs.readFileSync(req.file.path);
    const textContent = TextCodec.decode(tcfData);
    
    const outputPath = path.join(CACHE_DIR, `${path.parse(req.file.filename).name}.txt`);
    fs.writeFileSync(outputPath, textContent, 'utf8');
    
    // Cleanup uploaded file
    fs.unlinkSync(req.file.path);
    
    res.json({
      success: true,
      outputFile: path.basename(outputPath),
      downloadUrl: `/api/download/${path.basename(outputPath)}`,
      content: textContent.substring(0, 1000) + (textContent.length > 1000 ? '...' : '')
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

// Image Codec API endpoints
app.post('/api/image/encode', upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const quality = parseInt(req.body.quality) || 85;
    const icfData = await ImageCodec.encode(req.file.path, quality);
    
    const outputPath = path.join(CACHE_DIR, `${path.parse(req.file.filename).name}.icf`);
    fs.writeFileSync(outputPath, icfData);
    
    const originalSize = fs.statSync(req.file.path).size;
    const compressedSize = icfData.length;
    
    // Cleanup uploaded file
    fs.unlinkSync(req.file.path);
    
    res.json({
      success: true,
      outputFile: path.basename(outputPath),
      downloadUrl: `/api/download/${path.basename(outputPath)}`,
      stats: {
        originalSize,
        compressedSize,
        compressionRatio: originalSize / compressedSize,
        savings: ((originalSize - compressedSize) / originalSize) * 100
      }
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

app.post('/api/image/decode', upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const icfData = fs.readFileSync(req.file.path);
    const { data, width, height, channels } = await ImageCodec.decode(icfData);
    
    // Use Sharp to convert raw data to PNG
    const sharp = require('sharp');
    const outputPath = path.join(CACHE_DIR, `${path.parse(req.file.filename).name}.png`);
    
    await sharp(data, {
      raw: {
        width,
        height,
        channels: channels as 1 | 2 | 3 | 4
      }
    })
    .png()
    .toFile(outputPath);
    
    // Cleanup uploaded file
    fs.unlinkSync(req.file.path);
    
    res.json({
      success: true,
      outputFile: path.basename(outputPath),
      downloadUrl: `/api/download/${path.basename(outputPath)}`,
      imageInfo: {
        width,
        height,
        channels
      }
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

// Video Codec API endpoints
app.post('/api/video/encode', upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const quality = parseInt(req.body.quality) || 85;
    const bitrate = parseInt(req.body.bitrate) || 1000000;
    const gopSize = parseInt(req.body.gopSize) || 30;
    
    const outputPath = path.join(CACHE_DIR, `${path.parse(req.file.filename).name}.vcf`);
    
    await VideoCodec.encode(req.file.path, outputPath, {
      quality,
      bitrate,
      gopSize
    });
    
    const originalSize = fs.statSync(req.file.path).size;
    const compressedSize = fs.statSync(outputPath).size;
    
    // Cleanup uploaded file
    fs.unlinkSync(req.file.path);
    
    res.json({
      success: true,
      outputFile: path.basename(outputPath),
      downloadUrl: `/api/download/${path.basename(outputPath)}`,
      stats: {
        originalSize,
        compressedSize,
        compressionRatio: originalSize / compressedSize,
        savings: ((originalSize - compressedSize) / originalSize) * 100
      }
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

app.post('/api/video/decode', upload.single('file'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No file uploaded' });
    }

    const outputPath = path.join(CACHE_DIR, `${path.parse(req.file.filename).name}.mp4`);
    
    await VideoCodec.decode(req.file.path, outputPath);
    
    // Cleanup uploaded file
    fs.unlinkSync(req.file.path);
    
    res.json({
      success: true,
      outputFile: path.basename(outputPath),
      downloadUrl: `/api/download/${path.basename(outputPath)}`
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

// File download endpoint
app.get('/api/download/:filename', (req, res) => {
  const filename = req.params.filename;
  const filePath = path.join(CACHE_DIR, filename);
  
  if (!fs.existsSync(filePath)) {
    return res.status(404).json({ error: 'File not found' });
  }
  
  // Set appropriate headers
  const ext = path.extname(filename).toLowerCase();
  const contentTypes: { [key: string]: string } = {
    '.tcf': 'application/octet-stream',
    '.icf': 'application/octet-stream',
    '.vcf': 'application/octet-stream',
    '.txt': 'text/plain',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.mp4': 'video/mp4',
    '.avi': 'video/x-msvideo',
    '.mkv': 'video/x-matroska'
  };
  
  const contentType = contentTypes[ext] || 'application/octet-stream';
  
  res.setHeader('Content-Type', contentType);
  res.setHeader('Content-Disposition', `attachment; filename="${filename}"`);
  res.setHeader('Cache-Control', 'max-age=3600'); // 1 hour cache
  
  const fileStream = fs.createReadStream(filePath);
  fileStream.pipe(res);
});

// List available files endpoint
app.get('/api/files', (req, res) => {
  try {
    const files = fs.readdirSync(CACHE_DIR).map(filename => {
      const filePath = path.join(CACHE_DIR, filename);
      const stats = fs.statSync(filePath);
      const ext = path.extname(filename).toLowerCase();
      
      let fileType = 'unknown';
      if (['.tcf'].includes(ext)) fileType = 'text';
      else if (['.icf'].includes(ext)) fileType = 'image';
      else if (['.vcf'].includes(ext)) fileType = 'video';
      else if (['.txt'].includes(ext)) fileType = 'text';
      else if (['.png', '.jpg', '.jpeg'].includes(ext)) fileType = 'image';
      else if (['.mp4', '.avi', '.mkv'].includes(ext)) fileType = 'video';
      
      return {
        filename,
        size: stats.size,
        created: stats.birthtime,
        modified: stats.mtime,
        type: fileType,
        downloadUrl: `/api/download/${filename}`
      };
    });
    
    res.json(files);
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

// Streaming endpoint for video files
app.get('/api/stream/:filename', (req, res) => {
  const filename = req.params.filename;
  const filePath = path.join(CACHE_DIR, filename);
  
  if (!fs.existsSync(filePath)) {
    return res.status(404).json({ error: 'File not found' });
  }
  
  const stat = fs.statSync(filePath);
  const fileSize = stat.size;
  const range = req.headers.range;
  
  if (range) {
    // Support range requests for streaming
    const parts = range.replace(/bytes=/, "").split("-");
    const start = parseInt(parts[0], 10);
    const end = parts[1] ? parseInt(parts[1], 10) : fileSize - 1;
    const chunksize = (end - start) + 1;
    
    const fileStream = fs.createReadStream(filePath, { start, end });
    
    res.writeHead(206, {
      'Content-Range': `bytes ${start}-${end}/${fileSize}`,
      'Accept-Ranges': 'bytes',
      'Content-Length': chunksize,
      'Content-Type': 'application/octet-stream',
    });
    
    fileStream.pipe(res);
  } else {
    // No range request, send entire file
    res.writeHead(200, {
      'Content-Length': fileSize,
      'Content-Type': 'application/octet-stream',
    });
    
    fs.createReadStream(filePath).pipe(res);
  }
});

// Cleanup old files endpoint
app.delete('/api/cleanup', (req, res) => {
  try {
    const maxAge = parseInt(req.query.maxAge as string) || 3600000; // 1 hour default
    const now = Date.now();
    
    const files = fs.readdirSync(CACHE_DIR);
    let deletedCount = 0;
    
    files.forEach(filename => {
      const filePath = path.join(CACHE_DIR, filename);
      const stats = fs.statSync(filePath);
      
      if (now - stats.birthtime.getTime() > maxAge) {
        fs.unlinkSync(filePath);
        deletedCount++;
      }
    });
    
    res.json({
      success: true,
      deletedFiles: deletedCount,
      message: `Deleted ${deletedCount} files older than ${maxAge}ms`
    });
    
  } catch (error: any) {
    res.status(500).json({ error: error.message });
  }
});

// API documentation endpoint
app.get('/api/docs', (req, res) => {
  res.json({
    title: 'Custom Codec CDN API',
    version: '1.0.0',
    description: 'REST API for custom text, image, and video codec operations',
    endpoints: {
      'POST /api/text/encode': 'Encode text file to TCF format',
      'POST /api/text/decode': 'Decode TCF file to text',
      'POST /api/image/encode': 'Encode image to ICF format',
      'POST /api/image/decode': 'Decode ICF file to image',
      'POST /api/video/encode': 'Encode video to VCF format',
      'POST /api/video/decode': 'Decode VCF file to video',
      'GET /api/download/:filename': 'Download processed file',
      'GET /api/stream/:filename': 'Stream video file with range support',
      'GET /api/files': 'List all processed files',
      'DELETE /api/cleanup': 'Clean up old files',
      'GET /health': 'Health check',
      'GET /api/docs': 'This documentation'
    },
    examples: {
      textEncode: 'curl -X POST -F "file=@document.txt" http://localhost:3000/api/text/encode',
      imageEncode: 'curl -X POST -F "file=@photo.jpg" -F "quality=90" http://localhost:3000/api/image/encode',
      videoEncode: 'curl -X POST -F "file=@video.mp4" -F "quality=85" -F "gopSize=30" http://localhost:3000/api/video/encode'
    }
  });
});

// Error handling middleware
app.use((error: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
  console.error('Server error:', error);
  res.status(500).json({
    error: 'Internal server error',
    message: error.message
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({
    error: 'Not found',
    message: `Route ${req.method} ${req.path} not found`,
    availableRoutes: [
      'GET /health',
      'GET /api/docs',
      'GET /api/files',
      'POST /api/text/encode',
      'POST /api/text/decode',
      'POST /api/image/encode',
      'POST /api/image/decode',
      'POST /api/video/encode',
      'POST /api/video/decode'
    ]
  });
});

// Start server
app.listen(PORT, () => {
  console.log(`🚀 Custom Codec CDN Server running on port ${PORT}`);
  console.log(`📖 API Documentation: http://localhost:${PORT}/api/docs`);
  console.log(`💚 Health Check: http://localhost:${PORT}/health`);
  console.log(`📁 Upload directory: ${UPLOAD_DIR}`);
  console.log(`🗄️  Cache directory: ${CACHE_DIR}`);
});

export default app;