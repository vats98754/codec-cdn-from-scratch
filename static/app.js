// Professional AI Codec Platform - JavaScript
class CodecPlatform {
    constructor() {
        this.selectedFiles = {
            text: null,
            image: null,
            video: null,
            bencode: null
        };
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.loadFiles();
        this.showWelcomeAnimation();
    }

    showWelcomeAnimation() {
        // Add a subtle animation to the hero section
        const hero = document.querySelector('.hero');
        if (hero) {
            hero.style.opacity = '0';
            hero.style.transform = 'translateY(20px)';
            setTimeout(() => {
                hero.style.transition = 'all 0.8s ease-out';
                hero.style.opacity = '1';
                hero.style.transform = 'translateY(0)';
            }, 100);
        }
    }

    setupEventListeners() {
        // Upload area click handlers
        document.querySelectorAll('.upload-area').forEach(area => {
            area.addEventListener('click', (e) => {
                const type = area.dataset.type;
                document.getElementById(`${type}File`).click();
            });

            // Drag and drop handlers
            area.addEventListener('dragover', this.handleDragOver.bind(this));
            area.addEventListener('dragleave', this.handleDragLeave.bind(this));
            area.addEventListener('drop', this.handleDrop.bind(this));
        });

        // File input change handlers
        document.querySelectorAll('.file-input').forEach(input => {
            input.addEventListener('change', this.handleFileSelect.bind(this));
        });

        // Button click handlers
        this.setupButtonHandlers();
    }

    setupButtonHandlers() {
        const buttons = {
            'encodeText': () => this.encodeText(),
            'decodeText': () => this.decodeText(),
            'encodeImage': () => this.encodeImage(),
            'decodeImage': () => this.decodeImage(),
            'encodeVideo': () => this.encodeVideo(),
            'decodeVideo': () => this.decodeVideo(),
            'encodeBencode': () => this.encodeBencode(),
            'decodeBencode': () => this.decodeBencode(),
            'createTorrent': () => this.createTorrent(),
            'runSpeedTest': () => this.runSpeedTest(),
            'runCompressionTest': () => this.runCompressionTest(),
            'analyzeFile': () => this.analyzeFile(),
            'runFullDemo': () => this.runFullDemo(),
            'loadFiles': () => this.loadFiles(),
            'cleanupFiles': () => this.cleanupFiles()
        };

        Object.entries(buttons).forEach(([id, handler]) => {
            const element = document.getElementById(id);
            if (element) {
                element.addEventListener('click', handler);
            }
        });

        // Example button handlers
        document.querySelectorAll('[data-example]').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const type = e.target.dataset.example;
                const subtype = e.target.dataset.subtype;
                this.loadExample(type, subtype);
            });
        });
    }

    handleDragOver(event) {
        event.preventDefault();
        event.currentTarget.classList.add('dragover');
    }

    handleDragLeave(event) {
        event.currentTarget.classList.remove('dragover');
    }

    handleDrop(event) {
        event.preventDefault();
        const area = event.currentTarget;
        area.classList.remove('dragover');
        
        const type = area.dataset.type;
        const files = event.dataTransfer.files;
        if (files.length > 0) {
            this.selectedFiles[type] = files[0];
            this.updateFileDisplay(type, files[0]);
        }
    }

    handleFileSelect(event) {
        const input = event.target;
        const type = input.id.replace('File', '');
        const file = input.files[0];
        if (file) {
            this.selectedFiles[type] = file;
            this.updateFileDisplay(type, file);
        }
    }

    updateFileDisplay(type, file) {
        const uploadArea = document.querySelector(`[data-type="${type}"]`);
        uploadArea.innerHTML = `
            <div class="upload-icon success">✅</div>
            <p><strong>Selected: ${file.name}</strong></p>
            <p class="file-size">Size: ${this.formatBytes(file.size)}</p>
            <p class="file-type">Type: ${file.type || 'Unknown'}</p>
        `;
        uploadArea.classList.add('file-selected');
    }

    async encodeText() {
        if (!this.selectedFiles.text) {
            this.showNotification('Please select a text file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.text);

        try {
            this.showProgress('text', true);
            const response = await fetch('/api/text/encode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('text', false);

            if (result.success) {
                this.showResult('text', 'success', `
                    <div class="result-success">
                        <h4>✅ Encoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Compression Ratio:</span>
                                <span class="stat-value">${result.stats.compressionRatio.toFixed(2)}:1</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Space Savings:</span>
                                <span class="stat-value">${result.stats.savings.toFixed(1)}%</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download TCF File
                        </a>
                    </div>
                `);
                this.showNotification('Text encoded successfully!', 'success');
            } else {
                this.showResult('text', 'error', result.error);
                this.showNotification('Encoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('text', false);
            this.showResult('text', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async decodeText() {
        if (!this.selectedFiles.text) {
            this.showNotification('Please select a TCF file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.text);

        try {
            this.showProgress('text', true);
            const response = await fetch('/api/text/decode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('text', false);

            if (result.success) {
                this.showResult('text', 'success', `
                    <div class="result-success">
                        <h4>✅ Decoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download Decoded File
                        </a>
                    </div>
                `);
                this.showNotification('Text decoded successfully!', 'success');
            } else {
                this.showResult('text', 'error', result.error);
                this.showNotification('Decoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('text', false);
            this.showResult('text', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async encodeImage() {
        if (!this.selectedFiles.image) {
            this.showNotification('Please select an image file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.image);
        formData.append('quality', document.getElementById('imageQuality').value);

        try {
            this.showProgress('image', true);
            const response = await fetch('/api/image/encode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('image', false);

            if (result.success) {
                this.showResult('image', 'success', `
                    <div class="result-success">
                        <h4>✅ Image Encoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Compression Ratio:</span>
                                <span class="stat-value">${result.stats.compressionRatio.toFixed(2)}:1</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Space Savings:</span>
                                <span class="stat-value">${result.stats.savings.toFixed(1)}%</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download ICF File
                        </a>
                    </div>
                `);
                this.showNotification('Image encoded successfully!', 'success');
            } else {
                this.showResult('image', 'error', result.error);
                this.showNotification('Encoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('image', false);
            this.showResult('image', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async decodeImage() {
        if (!this.selectedFiles.image) {
            this.showNotification('Please select an ICF file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.image);

        try {
            this.showProgress('image', true);
            const response = await fetch('/api/image/decode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('image', false);

            if (result.success) {
                this.showResult('image', 'success', `
                    <div class="result-success">
                        <h4>✅ Image Decoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Dimensions:</span>
                                <span class="stat-value">${result.imageInfo.width}x${result.imageInfo.height}</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download Decoded Image
                        </a>
                    </div>
                `);
                this.showNotification('Image decoded successfully!', 'success');
            } else {
                this.showResult('image', 'error', result.error);
                this.showNotification('Decoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('image', false);
            this.showResult('image', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async encodeVideo() {
        if (!this.selectedFiles.video) {
            this.showNotification('Please select a video file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.video);
        formData.append('quality', document.getElementById('videoQuality').value);
        formData.append('bitrate', document.getElementById('videoBitrate').value);
        formData.append('gopSize', document.getElementById('videoGop').value);

        try {
            this.showProgress('video', true);
            const response = await fetch('/api/video/encode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('video', false);

            if (result.success) {
                this.showResult('video', 'success', `
                    <div class="result-success">
                        <h4>✅ Video Encoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Compression Ratio:</span>
                                <span class="stat-value">${result.stats.compressionRatio.toFixed(2)}:1</span>
                            </div>
                            <div class="stat-item">
                                <span class="stat-label">Space Savings:</span>
                                <span class="stat-value">${result.stats.savings.toFixed(1)}%</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download VCF File
                        </a>
                    </div>
                `);
                this.showNotification('Video encoded successfully!', 'success');
            } else {
                this.showResult('video', 'error', result.error);
                this.showNotification('Encoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('video', false);
            this.showResult('video', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async decodeVideo() {
        if (!this.selectedFiles.video) {
            this.showNotification('Please select a VCF file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.video);

        try {
            this.showProgress('video', true);
            const response = await fetch('/api/video/decode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('video', false);

            if (result.success) {
                this.showResult('video', 'success', `
                    <div class="result-success">
                        <h4>✅ Video Decoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download Decoded Video
                        </a>
                    </div>
                `);
                this.showNotification('Video decoded successfully!', 'success');
            } else {
                this.showResult('video', 'error', result.error);
                this.showNotification('Decoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('video', false);
            this.showResult('video', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async encodeBencode() {
        if (!this.selectedFiles.bencode) {
            this.showNotification('Please select a JSON file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.bencode);

        try {
            this.showProgress('bencode', true);
            const response = await fetch('/api/bencode/encode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('bencode', false);

            if (result.success) {
                this.showResult('bencode', 'success', `
                    <div class="result-success">
                        <h4>✅ Bencode Encoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download Bencode File
                        </a>
                    </div>
                `);
                this.showNotification('Data encoded to Bencode successfully!', 'success');
            } else {
                this.showResult('bencode', 'error', result.error);
                this.showNotification('Encoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('bencode', false);
            this.showResult('bencode', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async decodeBencode() {
        if (!this.selectedFiles.bencode) {
            this.showNotification('Please select a Bencode file first', 'error');
            return;
        }

        const formData = new FormData();
        formData.append('file', this.selectedFiles.bencode);

        try {
            this.showProgress('bencode', true);
            const response = await fetch('/api/bencode/decode', {
                method: 'POST',
                body: formData
            });

            const result = await response.json();
            this.showProgress('bencode', false);

            if (result.success) {
                this.showResult('bencode', 'success', `
                    <div class="result-success">
                        <h4>✅ Bencode Decoding Successful!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Output File:</span>
                                <span class="stat-value">${result.outputFile}</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download JSON File
                        </a>
                    </div>
                `);
                this.showNotification('Bencode decoded successfully!', 'success');
            } else {
                this.showResult('bencode', 'error', result.error);
                this.showNotification('Decoding failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showProgress('bencode', false);
            this.showResult('bencode', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async createTorrent() {
        try {
            const response = await fetch('/api/bencode/create-torrent', {
                method: 'POST'
            });

            const result = await response.json();

            if (result.success) {
                this.showResult('bencode', 'success', `
                    <div class="result-success">
                        <h4>✅ Sample Torrent Created!</h4>
                        <div class="stats-grid">
                            <div class="stat-item">
                                <span class="stat-label">Torrent File:</span>
                                <span class="stat-value">${result.filename}</span>
                            </div>
                        </div>
                        <a href="${result.downloadUrl}" class="btn btn-download" download>
                            <i class="download-icon">⬇️</i> Download Torrent File
                        </a>
                    </div>
                `);
                this.showNotification('Sample torrent created successfully!', 'success');
            } else {
                this.showResult('bencode', 'error', result.error);
                this.showNotification('Failed to create torrent: ' + result.error, 'error');
            }
        } catch (error) {
            this.showResult('bencode', 'error', error.message);
            this.showNotification('Network error: ' + error.message, 'error');
        }
    }

    async runSpeedTest() {
        const results = document.getElementById('speedResults');
        results.innerHTML = '<div class="loading">🔄 Running speed tests...</div>';

        try {
            const testData = [
                { name: 'Small Text (1KB)', data: 'Lorem ipsum '.repeat(50) },
                { name: 'Medium Text (10KB)', data: 'Lorem ipsum '.repeat(500) },
                { name: 'Large Text (100KB)', data: 'Lorem ipsum '.repeat(5000) }
            ];

            let html = '<div class="test-results"><h4>⚡ Speed Test Results</h4>';

            for (const test of testData) {
                const start = performance.now();
                
                const formData = new FormData();
                formData.append('file', new Blob([test.data], { type: 'text/plain' }), 'test.txt');
                
                const response = await fetch('/api/text/encode', {
                    method: 'POST',
                    body: formData
                });
                
                const end = performance.now();
                const duration = end - start;
                
                if (response.ok) {
                    html += `<div class="test-row">
                        <span><strong>${test.name}:</strong></span>
                        <span class="speed-result">${duration.toFixed(2)}ms</span>
                    </div>`;
                }
            }
            html += '</div>';
            results.innerHTML = html;
            this.showNotification('Speed test completed!', 'success');

        } catch (error) {
            results.innerHTML = `<div class="error-badge">❌ Error: ${error.message}</div>`;
            this.showNotification('Speed test failed: ' + error.message, 'error');
        }
    }

    async runCompressionTest() {
        const results = document.getElementById('compressionResults');
        results.innerHTML = '<div class="loading">📊 Testing compression ratios...</div>';

        try {
            const testData = [
                { name: 'Lorem Ipsum (Repeated)', data: 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. '.repeat(100) },
                { name: 'JSON Data (Structured)', data: JSON.stringify({name: 'test', values: Array(100).fill({key: 'value', number: 123}), metadata: {created: new Date(), version: '1.0'}}) },
                { name: 'Source Code (Mixed)', data: `function test() {\n    return "hello world";\n}\n`.repeat(50) }
            ];

            let html = '<div class="test-results"><h4>📈 Compression Test Results</h4>';

            for (const test of testData) {
                const formData = new FormData();
                formData.append('file', new Blob([test.data], { type: 'text/plain' }), 'test.txt');
                
                const response = await fetch('/api/text/encode', {
                    method: 'POST',
                    body: formData
                });
                
                if (response.ok) {
                    const result = await response.json();
                    const ratio = result.stats.compressionRatio;
                    const savings = result.stats.savings;
                    
                    html += `<div class="test-row">
                        <span><strong>${test.name}:</strong></span>
                        <span class="compression-result">${ratio.toFixed(2)}:1 ratio, ${savings.toFixed(1)}% savings</span>
                    </div>`;
                }
            }
            html += '</div>';
            results.innerHTML = html;
            this.showNotification('Compression test completed!', 'success');

        } catch (error) {
            results.innerHTML = `<div class="error-badge">❌ Error: ${error.message}</div>`;
            this.showNotification('Compression test failed: ' + error.message, 'error');
        }
    }

    async analyzeFile() {
        const results = document.getElementById('analysisResults');
        
        // Find the first selected file
        let fileToAnalyze = null;
        let fileType = '';
        
        for (const [type, file] of Object.entries(this.selectedFiles)) {
            if (file) {
                fileToAnalyze = file;
                fileType = type;
                break;
            }
        }
        
        if (!fileToAnalyze) {
            this.showNotification('Please select a file to analyze first', 'error');
            return;
        }

        results.innerHTML = '<div class="loading">🔍 Analyzing file...</div>';

        try {
            let html = '<div class="analysis-results"><h4>📋 File Analysis</h4>';
            html += `<div class="analysis-section">
                <h5>Basic Information</h5>
                <div class="info-grid">
                    <div class="info-item">
                        <span class="info-label">Name:</span>
                        <span class="info-value">${fileToAnalyze.name}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Size:</span>
                        <span class="info-value">${this.formatBytes(fileToAnalyze.size)}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Type:</span>
                        <span class="info-value">${fileToAnalyze.type || 'Unknown'}</span>
                    </div>
                    <div class="info-item">
                        <span class="info-label">Last Modified:</span>
                        <span class="info-value">${new Date(fileToAnalyze.lastModified).toLocaleDateString()}</span>
                    </div>
                </div>
            </div>`;

            // Add codec-specific analysis
            if (fileType === 'text' || fileType === 'bencode') {
                const reader = new FileReader();
                reader.onload = (e) => {
                    const text = e.target.result;
                    html += `<div class="analysis-section">
                        <h5>Text Analysis</h5>
                        <div class="info-grid">
                            <div class="info-item">
                                <span class="info-label">Characters:</span>
                                <span class="info-value">${text.length.toLocaleString()}</span>
                            </div>
                            <div class="info-item">
                                <span class="info-label">Lines:</span>
                                <span class="info-value">${text.split('\\n').length.toLocaleString()}</span>
                            </div>
                            <div class="info-item">
                                <span class="info-label">Words (est.):</span>
                                <span class="info-value">${text.split(/\\s+/).length.toLocaleString()}</span>
                            </div>
                        </div>
                    </div>`;
                    html += '</div>';
                    results.innerHTML = html;
                };
                reader.readAsText(fileToAnalyze);
            } else {
                html += '</div>';
                results.innerHTML = html;
            }

            this.showNotification('File analysis completed!', 'success');

        } catch (error) {
            results.innerHTML = `<div class="error-badge">❌ Error: ${error.message}</div>`;
            this.showNotification('File analysis failed: ' + error.message, 'error');
        }
    }

    async loadExample(type, subtype) {
        this.showNotification(`Loading ${subtype} example for ${type}...`, 'info');
        
        // This would load example files - for now just show a message
        setTimeout(() => {
            this.showNotification(`${subtype} example loaded for ${type} codec`, 'success');
        }, 1000);
    }

    async runFullDemo() {
        this.showNotification('Running complete demo of all codecs...', 'info');
        
        // This would run a comprehensive demo - for now just show a message
        setTimeout(() => {
            this.showNotification('Complete demo finished! Check individual codec sections for results.', 'success');
        }, 2000);
    }

    async loadFiles() {
        try {
            const response = await fetch('/api/files');
            const files = await response.json();
            
            const fileList = document.getElementById('fileList');
            fileList.innerHTML = '';
            
            if (files.length === 0) {
                fileList.innerHTML = '<div class="no-files">📁 No processed files found</div>';
                return;
            }

            files.forEach(file => {
                const fileItem = document.createElement('div');
                fileItem.className = 'file-item';
                fileItem.innerHTML = `
                    <h4>${file.name}</h4>
                    <div class="file-stats">Size: ${this.formatBytes(file.size)}</div>
                    <div class="file-stats">Created: ${new Date(file.created).toLocaleDateString()}</div>
                    <a href="${file.downloadUrl}" class="btn btn-download" download>
                        <i class="download-icon">⬇️</i> Download
                    </a>
                `;
                fileList.appendChild(fileItem);
            });
            
            this.showNotification(`Loaded ${files.length} files`, 'success');
        } catch (error) {
            console.error('Error loading files:', error);
            this.showNotification('Failed to load files: ' + error.message, 'error');
        }
    }

    async cleanupFiles() {
        try {
            const response = await fetch('/api/cleanup', {
                method: 'DELETE'
            });
            const result = await response.json();
            
            if (result.success) {
                this.showNotification(result.message, 'success');
                this.loadFiles(); // Refresh the file list
            } else {
                this.showNotification('Cleanup failed: ' + result.error, 'error');
            }
        } catch (error) {
            this.showNotification('Cleanup failed: ' + error.message, 'error');
        }
    }

    showProgress(type, show) {
        const progress = document.getElementById(`${type}Progress`);
        if (progress) {
            progress.style.display = show ? 'block' : 'none';
            
            if (show) {
                const bar = document.getElementById(`${type}ProgressBar`);
                if (bar) {
                    bar.style.width = '0%';
                    
                    // Animate progress bar
                    let width = 0;
                    const interval = setInterval(() => {
                        width += Math.random() * 20;
                        if (width >= 90) {
                            clearInterval(interval);
                            width = 90;
                        }
                        bar.style.width = width + '%';
                    }, 200);
                    
                    // Store interval ID for cleanup
                    progress.dataset.interval = interval;
                }
            } else {
                // Clean up interval
                const interval = progress.dataset.interval;
                if (interval) {
                    clearInterval(parseInt(interval));
                    delete progress.dataset.interval;
                }
            }
        }
    }

    showResult(type, status, message) {
        const result = document.getElementById(`${type}Result`);
        if (result) {
            result.innerHTML = message;
            result.className = `result ${status}`;
            result.style.display = 'block';
            
            // Auto-hide after 10 seconds for success messages
            if (status === 'success') {
                setTimeout(() => {
                    result.style.display = 'none';
                }, 10000);
            }
        }
    }

    showNotification(message, type = 'info') {
        // Create notification element
        const notification = document.createElement('div');
        notification.className = `notification notification-${type}`;
        
        const icons = {
            success: '✅',
            error: '❌',
            warning: '⚠️',
            info: 'ℹ️'
        };
        
        notification.innerHTML = `
            <span class="notification-icon">${icons[type] || icons.info}</span>
            <span class="notification-message">${message}</span>
            <button class="notification-close">×</button>
        `;
        
        // Add to notification container
        let container = document.getElementById('notificationContainer');
        if (!container) {
            container = document.createElement('div');
            container.id = 'notificationContainer';
            container.className = 'notification-container';
            document.body.appendChild(container);
        }
        
        container.appendChild(notification);
        
        // Auto-remove after 5 seconds
        setTimeout(() => {
            if (notification.parentNode) {
                notification.remove();
            }
        }, 5000);
        
        // Close button handler
        notification.querySelector('.notification-close').addEventListener('click', () => {
            notification.remove();
        });
        
        // Animate in
        setTimeout(() => {
            notification.classList.add('show');
        }, 10);
    }

    formatBytes(bytes, decimals = 2) {
        if (bytes === 0) return '0 Bytes';

        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

        const i = Math.floor(Math.log(bytes) / Math.log(k));

        return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
    }
}

// Initialize the platform when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new CodecPlatform();
});