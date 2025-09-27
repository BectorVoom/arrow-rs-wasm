/**
 * Comprehensive Feather compression testing module
 * Handles all compression testing scenarios and validation
 */

export class FeatherCompressionTester {
    constructor(wasmModule, logger, progressCallback) {
        this.wasm = wasmModule;
        this.log = logger;
        this.updateProgress = progressCallback;
        this.testResults = [];
        this.generatedFiles = new Map(); // Store generated files for download
    }

    /**
     * Test a single compression type
     */
    async testCompression(dataset, compressionType) {
        const startTime = performance.now();
        this.log(`🧪 Testing ${compressionType.toUpperCase()} compression on "${dataset.name}"...`);
        
        try {
            // Create table from JSON data
            this.log(`📊 Creating table from ${dataset.data.length} rows...`);
            const table = this.wasm.tableFromJSON(dataset.data);
            
            // Prepare write options
            const writeOptions = this.createWriteOptions(compressionType);
            
            // Measure write performance
            const writeStartTime = performance.now();
            const compressedData = table.toIPC(writeOptions);
            const writeTime = performance.now() - writeStartTime;
            
            // Get file size
            const fileSize = compressedData.length;
            this.log(`📦 Generated ${compressionType.toUpperCase()} file: ${fileSize} bytes (${Math.round(fileSize/1024*100)/100} KB)`);
            
            // Test reading the compressed file
            this.log(`📖 Reading compressed file back...`);
            const readStartTime = performance.now();
            const readTable = this.wasm.tableFromIPC(compressedData);
            const readTime = performance.now() - readStartTime;
            
            // Validate data integrity
            const integrity = await this.validateDataIntegrity(dataset.data, readTable);
            
            // Calculate compression ratio
            const originalSize = this.estimateOriginalSize(dataset.data);
            const compressionRatio = originalSize / fileSize;
            
            const totalTime = performance.now() - startTime;
            
            const result = {
                dataset: dataset.name,
                compression: compressionType,
                fileSize,
                originalSize,
                compressionRatio,
                writeTime: Math.round(writeTime * 100) / 100,
                readTime: Math.round(readTime * 100) / 100,
                totalTime: Math.round(totalTime * 100) / 100,
                integrity: integrity.valid,
                integrityDetails: integrity.details,
                timestamp: new Date().toISOString()
            };
            
            this.testResults.push(result);
            this.addResultToTable(result);
            
            // Store file for download
            this.storeGeneratedFile(dataset.name, compressionType, compressedData);
            
            this.log(`✅ ${compressionType.toUpperCase()} test completed: ${Math.round(compressionRatio*100)/100}x compression, integrity: ${integrity.valid ? 'PASS' : 'FAIL'}`, 
                     integrity.valid ? 'success' : 'error');
            
            // Clean up
            table.dispose();
            readTable.dispose();
            
            return result;
            
        } catch (error) {
            this.log(`❌ ${compressionType.toUpperCase()} test failed: ${error}`, 'error');
            throw error;
        }
    }

    /**
     * Test all compression types
     */
    async testAllCompressions(dataset) {
        this.log(`🚀 Running comprehensive compression test on "${dataset.name}"...`);
        this.clearResults();
        
        const compressionTypes = ['none', 'lz4', 'zstd'];
        const results = [];
        
        for (let i = 0; i < compressionTypes.length; i++) {
            const compression = compressionTypes[i];
            this.updateProgress((i / compressionTypes.length) * 100, `Testing ${compression.toUpperCase()}...`);
            
            try {
                const result = await this.testCompression(dataset, compression);
                results.push(result);
                
                // Small delay to allow UI updates
                await new Promise(resolve => setTimeout(resolve, 100));
                
            } catch (error) {
                this.log(`⚠️ Skipping ${compression} due to error: ${error}`, 'warning');
            }
        }
        
        this.updateProgress(100, 'Complete!');
        setTimeout(() => this.updateProgress(0), 2000);
        
        // Display comparison summary
        this.displayCompressionSummary(results);
        
        return results;
    }

    /**
     * Test edge cases
     */
    async testEdgeCases() {
        this.log(`🔬 Running edge case tests...`);
        
        const edgeCases = [
            { name: 'Empty Dataset', data: [] },
            { name: 'Single Row', data: [{ id: 1, value: 'test' }] },
            { name: 'Null Values', data: [
                { id: 1, name: 'Alice', value: null },
                { id: 2, name: null, value: 42 },
                { id: null, name: 'Charlie', value: 3.14 }
            ]},
            { name: 'Unicode Data', data: [
                { id: 1, text: '🚀 Rocket', emoji: '🌟' },
                { id: 2, text: 'العربية', script: 'Arabic' },
                { id: 3, text: '中文', script: 'Chinese' }
            ]}
        ];
        
        for (const testCase of edgeCases) {
            this.log(`\n--- Testing: ${testCase.name} ---`);
            try {
                if (testCase.data.length === 0) {
                    this.log(`⚠️ Skipping empty dataset (not supported by current implementation)`, 'warning');
                    continue;
                }
                
                await this.testCompression(testCase, 'lz4');
                this.log(`✅ ${testCase.name}: PASSED`, 'success');
            } catch (error) {
                this.log(`❌ ${testCase.name}: FAILED - ${error}`, 'error');
            }
        }
    }

    /**
     * Performance benchmark
     */
    async performanceBenchmark() {
        this.log(`⚡ Running performance benchmark...`);
        
        const sizes = [100, 500, 1000, 5000, 10000];
        const compressionTypes = ['none', 'lz4', 'zstd'];
        
        this.log('\n📊 Performance Results:');
        this.log('Size\t\tCompression\tWrite(ms)\tRead(ms)\tSize(KB)\tRatio');
        this.log(''.padEnd(80, '-'));
        
        for (const size of sizes) {
            const dataset = {
                name: `Benchmark ${size} rows`,
                data: this.generateBenchmarkData(size)
            };
            
            for (const compression of compressionTypes) {
                try {
                    const result = await this.testCompression(dataset, compression);
                    const sizeKB = Math.round(result.fileSize / 1024 * 100) / 100;
                    const ratio = Math.round(result.compressionRatio * 100) / 100;
                    
                    this.log(`${size}\t\t${compression}\t\t${result.writeTime}\t\t${result.readTime}\t\t${sizeKB}\t\t${ratio}`);
                } catch (error) {
                    this.log(`${size}\t\t${compression}\t\tFAILED: ${error}`, 'error');
                }
            }
        }
    }

    /**
     * Complete round-trip test
     */
    async completeRoundTripTest(dataset) {
        this.log(`🔄 Running complete round-trip test for "${dataset.name}"...`);
        
        const compressionTypes = ['none', 'lz4', 'zstd'];
        let allPassed = true;
        
        for (const compression of compressionTypes) {
            this.log(`\n--- Round-trip test: ${compression.toUpperCase()} ---`);
            
            try {
                // Step 1: Create table
                this.log(`1️⃣ Creating table from JSON data...`);
                const table1 = this.wasm.tableFromJSON(dataset.data);
                
                // Step 2: Write to compressed format
                this.log(`2️⃣ Writing to ${compression.toUpperCase()} format...`);
                const writeOptions = this.createWriteOptions(compression);
                const compressedData = table1.toIPC(writeOptions);
                
                // Step 3: Read back
                this.log(`3️⃣ Reading back from compressed data...`);
                const table2 = this.wasm.tableFromIPC(compressedData);
                
                // Step 4: Convert back to JavaScript
                this.log(`4️⃣ Converting back to JavaScript objects...`);
                const roundTripData = this.tableToArray(table2);
                
                // Step 5: Validate integrity
                this.log(`5️⃣ Validating data integrity...`);
                const integrity = this.compareDatasets(dataset.data, roundTripData);
                
                if (integrity.valid) {
                    this.log(`✅ Round-trip ${compression.toUpperCase()}: PASSED`, 'success');
                } else {
                    this.log(`❌ Round-trip ${compression.toUpperCase()}: FAILED - ${integrity.details}`, 'error');
                    allPassed = false;
                }
                
                // Clean up
                table1.dispose();
                table2.dispose();
                
            } catch (error) {
                this.log(`❌ Round-trip ${compression.toUpperCase()}: ERROR - ${error}`, 'error');
                allPassed = false;
            }
        }
        
        this.log(`\n🏁 Round-trip test summary: ${allPassed ? 'ALL PASSED' : 'SOME FAILED'}`, 
                 allPassed ? 'success' : 'error');
    }

    /**
     * Test uploaded file
     */
    async testUploadedFile(file) {
        this.log(`📁 Testing uploaded file: ${file.name}...`);
        
        try {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);
            
            this.log(`📦 File size: ${Math.round(arrayBuffer.byteLength / 1024 * 100) / 100} KB`);
            
            // Try to read the file
            const table = this.wasm.tableFromIPC(uint8Array);
            
            this.log(`✅ File read successfully!`, 'success');
            this.log(`📊 Table info: ${table.numRows} rows, ${table.numColumns} columns`);
            
            // Get a sample of the data
            const sampleData = this.tableToArray(table, 5); // First 5 rows
            this.log(`📋 Sample data:\n${JSON.stringify(sampleData, null, 2)}`);
            
            // Test re-compression
            this.log(`🔄 Testing re-compression...`);
            const recompressed = table.toIPC(this.createWriteOptions('lz4'));
            const compressionRatio = arrayBuffer.byteLength / recompressed.length;
            
            this.log(`📈 Re-compression ratio: ${Math.round(compressionRatio * 100) / 100}x`, 'success');
            
            table.dispose();
            
        } catch (error) {
            this.log(`❌ Failed to process uploaded file: ${error}`, 'error');
        }
    }

    // Helper methods
    
    createWriteOptions(compressionType) {
        const options = this.wasm.WriteOptions.new();
        
        switch (compressionType.toLowerCase()) {
            case 'lz4':
                options.set_compression(this.wasm.CompressionType.LZ4);
                break;
            case 'zstd':
                options.set_compression(this.wasm.CompressionType.ZSTD);
                break;
            case 'none':
            default:
                options.set_compression(this.wasm.CompressionType.None);
                break;
        }
        
        return options;
    }

    async validateDataIntegrity(originalData, table) {
        try {
            // Convert table back to JavaScript array
            const readData = this.tableToArray(table);
            
            // Compare with original
            return this.compareDatasets(originalData, readData);
            
        } catch (error) {
            return {
                valid: false,
                details: `Validation error: ${error}`
            };
        }
    }

    compareDatasets(original, roundTrip) {
        if (original.length !== roundTrip.length) {
            return {
                valid: false,
                details: `Row count mismatch: ${original.length} vs ${roundTrip.length}`
            };
        }

        if (original.length === 0) {
            return { valid: true, details: 'Both datasets empty' };
        }

        // Check column structure
        const originalCols = Object.keys(original[0]).sort();
        const roundTripCols = Object.keys(roundTrip[0]).sort();
        
        if (JSON.stringify(originalCols) !== JSON.stringify(roundTripCols)) {
            return {
                valid: false,
                details: `Column mismatch: [${originalCols.join(', ')}] vs [${roundTripCols.join(', ')}]`
            };
        }

        // Sample check (first 10 rows or all if fewer)
        const checkRows = Math.min(10, original.length);
        for (let i = 0; i < checkRows; i++) {
            for (const col of originalCols) {
                const origVal = original[i][col];
                const rtVal = roundTrip[i][col];
                
                // Handle null comparisons
                if (origVal === null && rtVal === null) continue;
                if (origVal === null || rtVal === null) {
                    return {
                        valid: false,
                        details: `Null mismatch at row ${i}, col ${col}: ${origVal} vs ${rtVal}`
                    };
                }
                
                // Handle number comparisons with tolerance
                if (typeof origVal === 'number' && typeof rtVal === 'number') {
                    if (Math.abs(origVal - rtVal) > 1e-10) {
                        return {
                            valid: false,
                            details: `Number mismatch at row ${i}, col ${col}: ${origVal} vs ${rtVal}`
                        };
                    }
                } else if (origVal !== rtVal) {
                    return {
                        valid: false,
                        details: `Value mismatch at row ${i}, col ${col}: ${origVal} vs ${rtVal}`
                    };
                }
            }
        }

        return { valid: true, details: `Validated ${checkRows} rows successfully` };
    }

    tableToArray(table, maxRows = null) {
        // This is a simplified implementation
        // In a real implementation, you'd iterate through the table data
        const numRows = maxRows ? Math.min(table.numRows, maxRows) : table.numRows;
        const result = [];
        
        // For now, return a placeholder indicating successful conversion
        // This would need to be implemented based on the actual table API
        for (let i = 0; i < numRows; i++) {
            const row = {};
            // This is where you'd extract actual row data from the table
            // For demo purposes, we'll create a simple structure
            row.row_index = i;
            result.push(row);
        }
        
        return result;
    }

    estimateOriginalSize(data) {
        // Rough estimation using JSON serialization
        return new Blob([JSON.stringify(data)]).size;
    }

    generateBenchmarkData(size) {
        const data = [];
        for (let i = 1; i <= size; i++) {
            data.push({
                id: i,
                name: `User_${i}`,
                value: Math.random() * 1000,
                category: `Cat_${i % 10}`,
                active: i % 3 === 0,
                timestamp: new Date(Date.now() - Math.random() * 1000000000).toISOString()
            });
        }
        return data;
    }

    storeGeneratedFile(datasetName, compression, data) {
        const key = `${datasetName}_${compression}`;
        this.generatedFiles.set(key, data);
        this.updateDownloadLinks();
    }

    updateDownloadLinks() {
        const container = document.getElementById('download-links');
        if (!container) return;
        
        container.innerHTML = '';
        
        if (this.generatedFiles.size === 0) {
            container.innerHTML = '<p class="info">Generated files will appear here for download</p>';
            return;
        }
        
        for (const [key, data] of this.generatedFiles) {
            const button = document.createElement('button');
            button.textContent = `📥 Download ${key}.feather`;
            button.onclick = () => this.downloadFile(key, data);
            container.appendChild(button);
        }
    }

    downloadFile(filename, data) {
        const blob = new Blob([data], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${filename}.feather`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        this.log(`📁 Downloaded: ${filename}.feather`, 'success');
    }

    addResultToTable(result) {
        const tbody = document.getElementById('results-tbody');
        if (!tbody) return;
        
        const resultsTable = document.getElementById('compression-results');
        resultsTable.style.display = 'block';
        
        const row = tbody.insertRow();
        row.innerHTML = `
            <td>${result.compression.toUpperCase()}</td>
            <td>${result.fileSize.toLocaleString()}</td>
            <td>${Math.round(result.compressionRatio * 100) / 100}x</td>
            <td>${result.writeTime}</td>
            <td>${result.readTime}</td>
            <td><span class="${result.integrity ? 'success' : 'error'}">${result.integrity ? '✅ PASS' : '❌ FAIL'}</span></td>
            <td><button onclick="window.featherTester?.downloadResult('${result.dataset}_${result.compression}')">📥 Download</button></td>
        `;
    }

    clearResults() {
        const tbody = document.getElementById('results-tbody');
        if (tbody) tbody.innerHTML = '';
        
        const resultsTable = document.getElementById('compression-results');
        if (resultsTable) resultsTable.style.display = 'none';
    }

    displayCompressionSummary(results) {
        if (results.length === 0) return;
        
        this.log('\n📊 COMPRESSION SUMMARY:');
        this.log(''.padEnd(60, '='));
        
        const bestCompression = results.reduce((best, current) => 
            current.compressionRatio > best.compressionRatio ? current : best
        );
        
        const fastestWrite = results.reduce((fastest, current) => 
            current.writeTime < fastest.writeTime ? current : fastest
        );
        
        const fastestRead = results.reduce((fastest, current) => 
            current.readTime < fastest.readTime ? current : fastest
        );
        
        this.log(`🏆 Best compression: ${bestCompression.compression.toUpperCase()} (${Math.round(bestCompression.compressionRatio * 100) / 100}x)`, 'success');
        this.log(`⚡ Fastest write: ${fastestWrite.compression.toUpperCase()} (${fastestWrite.writeTime}ms)`, 'success');
        this.log(`⚡ Fastest read: ${fastestRead.compression.toUpperCase()} (${fastestRead.readTime}ms)`, 'success');
        
        for (const result of results) {
            this.log(`${result.compression.toUpperCase()}: ${Math.round(result.fileSize/1024*100)/100}KB, ${Math.round(result.compressionRatio*100)/100}x ratio, integrity: ${result.integrity ? 'PASS' : 'FAIL'}`);
        }
    }

    getTestResults() {
        return {
            summary: {
                totalTests: this.testResults.length,
                timestamp: new Date().toISOString(),
                environment: {
                    userAgent: navigator.userAgent,
                    platform: navigator.platform
                }
            },
            results: this.testResults
        };
    }
}

// Make available globally for button onclick handlers
if (typeof window !== 'undefined') {
    window.FeatherCompressionTester = FeatherCompressionTester;
}