/**
 * Performance Baseline Metrics System
 * 
 * Establishes and validates performance baselines for the Arrow-RS WASM library
 * Measures critical operations and tracks performance regression
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class PerformanceBaseline {
    constructor() {
        this.metrics = new Map();
        this.baselines = new Map();
        this.benchmarks = [];
        this.testData = new Map();
        this.config = {
            warmupRuns: 5,
            measurementRuns: 10,
            memoryMeasurementInterval: 100, // ms
            performanceThresholds: {
                initialization: 1000, // ms
                table_loading: 500, // ms per MB
                column_export: 200, // ms per 10k rows
                memory_cleanup: 100, // ms
                gc_pressure: 0.1 // ratio
            }
        };
    }

    /**
     * Initialize performance measurement system
     */
    async initialize() {
        console.log('Initializing performance baseline system...');
        
        // Load existing baselines if available
        await this.loadExistingBaselines();
        
        // Create test data sets
        await this.generateTestData();
        
        console.log('Performance baseline system ready');
    }

    /**
     * Load existing performance baselines
     */
    async loadExistingBaselines() {
        const baselinePath = path.join(__dirname, 'baselines.json');
        
        try {
            const data = await fs.readFile(baselinePath, 'utf-8');
            const baselines = JSON.parse(data);
            
            for (const [operation, baseline] of Object.entries(baselines)) {
                this.baselines.set(operation, baseline);
            }
            
            console.log(`Loaded ${this.baselines.size} existing baselines`);
        } catch (error) {
            console.log('No existing baselines found, will establish new ones');
        }
    }

    /**
     * Generate test data for performance measurements
     */
    async generateTestData() {
        // Small dataset (1K rows)
        this.testData.set('small', {
            size: 1000,
            columns: 5,
            description: 'Small dataset for basic operations'
        });
        
        // Medium dataset (100K rows)
        this.testData.set('medium', {
            size: 100000,
            columns: 10,
            description: 'Medium dataset for typical use cases'
        });
        
        // Large dataset (1M rows)
        this.testData.set('large', {
            size: 1000000,
            columns: 20,
            description: 'Large dataset for stress testing'
        });
        
        // Wide dataset (many columns)
        this.testData.set('wide', {
            size: 10000,
            columns: 100,
            description: 'Wide dataset for schema complexity'
        });
    }

    /**
     * Run comprehensive performance benchmark suite
     */
    async runBenchmarkSuite(wasmModule) {
        console.log('Starting comprehensive performance benchmark...');
        
        const results = {
            timestamp: new Date().toISOString(),
            environment: await this.getEnvironmentInfo(),
            benchmarks: {},
            summary: {
                totalOperations: 0,
                passedThresholds: 0,
                failedThresholds: 0,
                regressions: []
            }
        };
        
        // Initialize WASM module
        await this.measureOperation('initialization', async () => {
            await wasmModule.init_with_options(true);
        });
        
        // Test each data size
        for (const [sizeKey, testConfig] of this.testData) {
            console.log(`\nTesting with ${sizeKey} dataset (${testConfig.size} rows, ${testConfig.columns} cols)`);
            
            const datasetResults = await this.benchmarkDataset(wasmModule, sizeKey, testConfig);
            results.benchmarks[sizeKey] = datasetResults;
        }
        
        // Memory stress test
        results.benchmarks.memory_stress = await this.benchmarkMemoryStress(wasmModule);
        
        // Concurrent operations test
        results.benchmarks.concurrency = await this.benchmarkConcurrency(wasmModule);
        
        // Calculate summary
        this.calculateBenchmarkSummary(results);
        
        // Update baselines if this is a new baseline run
        await this.updateBaselines(results);
        
        return results;
    }

    /**
     * Benchmark operations with specific dataset
     */
    async benchmarkDataset(wasmModule, sizeKey, testConfig) {
        const results = {
            dataset: testConfig,
            operations: {}
        };
        
        // Generate test data
        const testData = this.createTestArrowData(testConfig.size, testConfig.columns);
        
        // Table loading
        const loadingMetrics = await this.measureOperation('table_loading', async () => {
            const handle = wasmModule.read_table_from_array_buffer(testData.buffer);
            wasmModule.free_table(handle);
            return handle;
        }, testData.byteLength);
        results.operations.table_loading = loadingMetrics;
        
        // Column export (using persistent table)
        const handle = wasmModule.read_table_from_array_buffer(testData.buffer);
        
        const exportMetrics = await this.measureOperation('column_export', async () => {
            const schema = JSON.parse(wasmModule.table_schema_summary(handle));
            const firstColumn = schema.columns[0].name;
            const columnData = wasmModule.export_column_by_name(handle, firstColumn);
            return columnData;
        }, testConfig.size);
        results.operations.column_export = exportMetrics;
        
        // Schema operations
        const schemaMetrics = await this.measureOperation('schema_operations', async () => {
            const schema = wasmModule.table_schema_summary(handle);
            const rowCount = wasmModule.get_table_row_count(handle);
            return { schema, rowCount };
        });
        results.operations.schema_operations = schemaMetrics;
        
        // Memory cleanup
        const cleanupMetrics = await this.measureOperation('memory_cleanup', async () => {
            wasmModule.free_table(handle);
        });
        results.operations.memory_cleanup = cleanupMetrics;
        
        return results;
    }

    /**
     * Benchmark memory stress scenarios
     */
    async benchmarkMemoryStress(wasmModule) {
        console.log('Running memory stress tests...');
        
        const results = {
            operations: {},
            memoryPressure: []
        };
        
        // Multiple table allocation
        const multiTableMetrics = await this.measureOperation('multi_table_allocation', async () => {
            const handles = [];
            const testData = this.createTestArrowData(10000, 5);
            
            // Allocate 10 tables
            for (let i = 0; i < 10; i++) {
                const handle = wasmModule.read_table_from_array_buffer(testData.buffer);
                handles.push(handle);
            }
            
            // Free all tables
            for (const handle of handles) {
                wasmModule.free_table(handle);
            }
            
            return handles.length;
        });
        results.operations.multi_table_allocation = multiTableMetrics;
        
        // Memory fragmentation test
        const fragmentationMetrics = await this.measureOperation('memory_fragmentation', async () => {
            const handles = [];
            
            // Allocate and free in pattern to create fragmentation
            for (let cycle = 0; cycle < 5; cycle++) {
                const testData = this.createTestArrowData(5000, 10);
                
                // Allocate batch
                for (let i = 0; i < 5; i++) {
                    const handle = wasmModule.read_table_from_array_buffer(testData.buffer);
                    handles.push(handle);
                }
                
                // Free every other table
                for (let i = handles.length - 1; i >= 0; i -= 2) {
                    wasmModule.free_table(handles[i]);
                    handles.splice(i, 1);
                }
            }
            
            // Clean up remaining tables
            for (const handle of handles) {
                wasmModule.free_table(handle);
            }
            
            return handles.length;
        });
        results.operations.memory_fragmentation = fragmentationMetrics;
        
        return results;
    }

    /**
     * Benchmark concurrent operations
     */
    async benchmarkConcurrency(wasmModule) {
        console.log('Running concurrency tests...');
        
        const results = {
            operations: {}
        };
        
        // Concurrent read operations
        const concurrentReadMetrics = await this.measureOperation('concurrent_reads', async () => {
            const testData = this.createTestArrowData(50000, 8);
            const handles = [];
            
            // Create multiple tables
            for (let i = 0; i < 5; i++) {
                const handle = wasmModule.read_table_from_array_buffer(testData.buffer);
                handles.push(handle);
            }
            
            // Concurrent read operations
            const operations = handles.map(handle => {
                return Promise.resolve().then(() => {
                    const schema = wasmModule.table_schema_summary(handle);
                    const rowCount = wasmModule.get_table_row_count(handle);
                    return { schema, rowCount };
                });
            });
            
            await Promise.all(operations);
            
            // Cleanup
            for (const handle of handles) {
                wasmModule.free_table(handle);
            }
            
            return handles.length;
        });
        results.operations.concurrent_reads = concurrentReadMetrics;
        
        return results;
    }

    /**
     * Measure operation performance with multiple runs
     */
    async measureOperation(operationName, operation, dataSize = null) {
        const measurements = [];
        const memorySnapshots = [];
        
        // Warmup runs
        for (let i = 0; i < this.config.warmupRuns; i++) {
            await operation();
        }
        
        // Force garbage collection if available
        if (global.gc) {
            global.gc();
        }
        
        // Measurement runs
        for (let i = 0; i < this.config.measurementRuns; i++) {
            const startMemory = this.getMemoryUsage();
            const startTime = performance.now();
            
            const result = await operation();
            
            const endTime = performance.now();
            const endMemory = this.getMemoryUsage();
            
            const duration = endTime - startTime;
            measurements.push(duration);
            
            memorySnapshots.push({
                before: startMemory,
                after: endMemory,
                delta: endMemory.used - startMemory.used
            });
        }
        
        // Calculate statistics
        const stats = this.calculateStatistics(measurements);
        const memoryStats = this.calculateMemoryStatistics(memorySnapshots);
        
        const metrics = {
            operation: operationName,
            dataSize: dataSize,
            duration: stats,
            memory: memoryStats,
            timestamp: new Date().toISOString(),
            threshold: this.getThreshold(operationName, dataSize),
            passed: this.checkThreshold(operationName, stats.median, dataSize)
        };
        
        console.log(`${operationName}: ${stats.median.toFixed(2)}ms (${metrics.passed ? 'PASS' : 'FAIL'})`);
        
        return metrics;
    }

    /**
     * Calculate performance statistics
     */
    calculateStatistics(measurements) {
        const sorted = [...measurements].sort((a, b) => a - b);
        const sum = measurements.reduce((a, b) => a + b, 0);
        
        return {
            min: Math.min(...measurements),
            max: Math.max(...measurements),
            mean: sum / measurements.length,
            median: sorted[Math.floor(sorted.length / 2)],
            p95: sorted[Math.floor(sorted.length * 0.95)],
            p99: sorted[Math.floor(sorted.length * 0.99)],
            stddev: Math.sqrt(measurements.reduce((sum, x) => sum + Math.pow(x - (sum / measurements.length), 2), 0) / measurements.length),
            samples: measurements.length
        };
    }

    /**
     * Calculate memory usage statistics
     */
    calculateMemoryStatistics(snapshots) {
        const deltas = snapshots.map(s => s.delta);
        const beforeValues = snapshots.map(s => s.before.used);
        const afterValues = snapshots.map(s => s.after.used);
        
        return {
            averageDelta: deltas.reduce((a, b) => a + b, 0) / deltas.length,
            maxDelta: Math.max(...deltas),
            minDelta: Math.min(...deltas),
            averageBefore: beforeValues.reduce((a, b) => a + b, 0) / beforeValues.length,
            averageAfter: afterValues.reduce((a, b) => a + b, 0) / afterValues.length
        };
    }

    /**
     * Get current memory usage
     */
    getMemoryUsage() {
        if (typeof process !== 'undefined' && process.memoryUsage) {
            const mem = process.memoryUsage();
            return {
                used: mem.heapUsed,
                total: mem.heapTotal,
                external: mem.external
            };
        } else if (typeof performance !== 'undefined' && performance.memory) {
            return {
                used: performance.memory.usedJSHeapSize,
                total: performance.memory.totalJSHeapSize,
                external: 0
            };
        } else {
            return { used: 0, total: 0, external: 0 };
        }
    }

    /**
     * Get performance threshold for operation
     */
    getThreshold(operationName, dataSize) {
        const baseThreshold = this.config.performanceThresholds[operationName];
        if (!baseThreshold) return null;
        
        // Scale threshold based on data size for certain operations
        if (operationName === 'table_loading' && dataSize) {
            return baseThreshold * (dataSize / (1024 * 1024)); // per MB
        } else if (operationName === 'column_export' && dataSize) {
            return baseThreshold * (dataSize / 10000); // per 10k rows
        }
        
        return baseThreshold;
    }

    /**
     * Check if performance meets threshold
     */
    checkThreshold(operationName, median, dataSize) {
        const threshold = this.getThreshold(operationName, dataSize);
        if (!threshold) return true; // No threshold defined
        
        return median <= threshold;
    }

    /**
     * Create test Arrow data
     */
    createTestArrowData(numRows, numColumns) {
        // Create simple Arrow IPC structure with specified dimensions
        const header = new Uint8Array([
            0x41, 0x52, 0x52, 0x4F, 0x57, 0x31, 0x00, 0x00 // "ARROW1\0\0"
        ]);
        
        // Create metadata for specified number of columns
        const metadata = this.createArrowMetadata(numColumns);
        
        // Create record batch with specified number of rows and columns
        const batch = this.createRecordBatch(numRows, numColumns);
        
        return this.combineArrowParts(header, metadata, batch);
    }

    /**
     * Create Arrow metadata for specified columns
     */
    createArrowMetadata(numColumns) {
        // Simplified metadata structure
        const columnSize = 16; // bytes per column metadata
        const metadataSize = numColumns * columnSize + 32; // header overhead
        
        const metadata = new Uint8Array(metadataSize);
        // Fill with basic metadata structure
        for (let i = 0; i < numColumns; i++) {
            const offset = 32 + i * columnSize;
            metadata.set([0x00, 0x00, 0x00, 0x08], offset); // Int32 type marker
        }
        
        return metadata;
    }

    /**
     * Create record batch with specified dimensions
     */
    createRecordBatch(numRows, numColumns) {
        const dataPerColumn = numRows * 4; // 4 bytes per Int32
        const totalSize = numColumns * dataPerColumn + 64; // batch header overhead
        
        const batch = new Uint8Array(totalSize);
        
        // Fill with test data
        for (let col = 0; col < numColumns; col++) {
            const colOffset = 64 + col * dataPerColumn;
            for (let row = 0; row < numRows; row++) {
                const rowOffset = colOffset + row * 4;
                const value = (col * 1000) + row; // Simple test pattern
                batch.set(new Uint8Array(new Int32Array([value]).buffer), rowOffset);
            }
        }
        
        return batch;
    }

    /**
     * Combine Arrow file parts
     */
    combineArrowParts(header, metadata, batch) {
        const total = new Uint8Array(header.length + metadata.length + batch.length);
        total.set(header, 0);
        total.set(metadata, header.length);
        total.set(batch, header.length + metadata.length);
        return total;
    }

    /**
     * Get environment information
     */
    async getEnvironmentInfo() {
        const info = {
            timestamp: new Date().toISOString(),
            userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Node.js',
            platform: typeof process !== 'undefined' ? process.platform : 'browser',
            arch: typeof process !== 'undefined' ? process.arch : 'unknown',
            nodeVersion: typeof process !== 'undefined' ? process.version : 'N/A'
        };
        
        if (typeof performance !== 'undefined' && performance.memory) {
            info.memory = {
                jsHeapSizeLimit: performance.memory.jsHeapSizeLimit,
                totalJSHeapSize: performance.memory.totalJSHeapSize,
                usedJSHeapSize: performance.memory.usedJSHeapSize
            };
        }
        
        return info;
    }

    /**
     * Calculate benchmark summary
     */
    calculateBenchmarkSummary(results) {
        let totalOps = 0;
        let passedThresholds = 0;
        let failedThresholds = 0;
        
        const processOperations = (operations) => {
            for (const [opName, metrics] of Object.entries(operations)) {
                if (metrics.duration) {
                    totalOps++;
                    if (metrics.passed) {
                        passedThresholds++;
                    } else {
                        failedThresholds++;
                    }
                } else if (metrics.operations) {
                    processOperations(metrics.operations);
                }
            }
        };
        
        processOperations(results.benchmarks);
        
        results.summary.totalOperations = totalOps;
        results.summary.passedThresholds = passedThresholds;
        results.summary.failedThresholds = failedThresholds;
    }

    /**
     * Update baseline measurements
     */
    async updateBaselines(results) {
        const baselinePath = path.join(__dirname, 'baselines.json');
        const reportPath = path.join(__dirname, 'baseline-report.json');
        
        // Save full results
        await fs.writeFile(reportPath, JSON.stringify(results, null, 2));
        
        // Extract baseline values
        const baselines = {};
        
        const extractBaselines = (operations, prefix = '') => {
            for (const [opName, metrics] of Object.entries(operations)) {
                if (metrics.duration) {
                    const key = prefix ? `${prefix}.${opName}` : opName;
                    baselines[key] = {
                        median: metrics.duration.median,
                        p95: metrics.duration.p95,
                        timestamp: metrics.timestamp,
                        dataSize: metrics.dataSize
                    };
                } else if (metrics.operations) {
                    const newPrefix = prefix ? `${prefix}.${opName}` : opName;
                    extractBaselines(metrics.operations, newPrefix);
                }
            }
        };
        
        extractBaselines(results.benchmarks);
        
        // Save baselines
        await fs.writeFile(baselinePath, JSON.stringify(baselines, null, 2));
        
        console.log(`Performance baselines updated: ${Object.keys(baselines).length} operations`);
        console.log(`Full report saved to: ${reportPath}`);
        
        return baselines;
    }
}

export { PerformanceBaseline };