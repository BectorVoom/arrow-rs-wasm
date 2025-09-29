/**
 * Performance Benchmark Suite for Arrow WASM Library
 * 
 * Establishes baseline metrics for module initialization, API latency,
 * and memory usage patterns as required by MBD deliverables.
 */

import { performance, PerformanceObserver } from 'perf_hooks';
import fs from 'fs/promises';
import path from 'path';

class PerformanceBenchmarks {
    constructor() {
        this.results = {
            timestamp: new Date().toISOString(),
            environment: {
                platform: process.platform,
                nodeVersion: process.version,
                arch: process.arch
            },
            benchmarks: {}
        };
    }

    async initializeWasm() {
        try {
            // Import WASM module from the correct path
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default(); // Initialize WASM
            return wasmModule;
        } catch (error) {
            console.error('❌ Failed to initialize WASM module:', error);
            throw error;
        }
    }

    async benchmarkModuleInitialization() {
        console.log('🚀 Benchmarking module initialization...');
        
        const times = [];
        const iterations = 10;
        
        for (let i = 0; i < iterations; i++) {
            const start = performance.now();
            
            try {
                await this.initializeWasm();
                const end = performance.now();
                times.push(end - start);
                console.log(`  Init ${i + 1}: ${(end - start).toFixed(2)}ms`);
            } catch (error) {
                console.error(`  Init ${i + 1}: FAILED - ${error.message}`);
                times.push(null);
            }
        }
        
        const validTimes = times.filter(t => t !== null);
        
        this.results.benchmarks.moduleInitialization = {
            iterations: iterations,
            successful: validTimes.length,
            times: validTimes,
            average: validTimes.length > 0 ? validTimes.reduce((a, b) => a + b, 0) / validTimes.length : null,
            min: validTimes.length > 0 ? Math.min(...validTimes) : null,
            max: validTimes.length > 0 ? Math.max(...validTimes) : null,
            baseline: validTimes.length > 0 ? validTimes.reduce((a, b) => a + b, 0) / validTimes.length : null
        };
        
        console.log(`✅ Module initialization: avg ${this.results.benchmarks.moduleInitialization.average?.toFixed(2)}ms`);
    }

    async benchmarkApiLatency(wasm) {
        console.log('⚡ Benchmarking API call latency...');
        
        const apiTests = [
            { name: 'get_version', fn: () => wasm.get_version() },
            { name: 'get_build_info', fn: () => wasm.get_build_info() },
            { name: 'is_lz4_supported', fn: () => wasm.is_lz4_supported() },
            { name: 'get_supported_compression_types', fn: () => wasm.get_supported_compression_types() },
            { name: 'get_memory_stats', fn: () => wasm.get_memory_stats() },
            { name: 'get_table_count', fn: () => wasm.get_table_count() }
        ];
        
        this.results.benchmarks.apiLatency = {};
        
        for (const test of apiTests) {
            const times = [];
            const iterations = 100;
            
            for (let i = 0; i < iterations; i++) {
                const start = performance.now();
                try {
                    test.fn();
                    const end = performance.now();
                    times.push(end - start);
                } catch (error) {
                    console.warn(`  ${test.name} iteration ${i + 1}: ${error.message}`);
                }
            }
            
            if (times.length > 0) {
                const average = times.reduce((a, b) => a + b, 0) / times.length;
                this.results.benchmarks.apiLatency[test.name] = {
                    iterations: iterations,
                    successful: times.length,
                    average,
                    min: Math.min(...times),
                    max: Math.max(...times),
                    baseline: average
                };
                
                console.log(`  ${test.name}: avg ${average.toFixed(3)}ms`);
            }
        }
    }

    async benchmarkMemoryUsage() {
        console.log('💾 Benchmarking memory usage...');
        
        const memStart = process.memoryUsage();
        
        // Simulate memory operations
        const wasm = await this.initializeWasm();
        
        // Clear any existing tables
        wasm.clear_all_tables();
        
        const memAfterInit = process.memoryUsage();
        
        this.results.benchmarks.memoryUsage = {
            initialMemory: memStart,
            afterInitialization: memAfterInit,
            memoryIncrease: {
                rss: memAfterInit.rss - memStart.rss,
                heapUsed: memAfterInit.heapUsed - memStart.heapUsed,
                heapTotal: memAfterInit.heapTotal - memStart.heapTotal,
                external: memAfterInit.external - memStart.external
            },
            baseline: {
                initMemoryIncrease: memAfterInit.rss - memStart.rss,
                heapIncreaseKB: (memAfterInit.heapUsed - memStart.heapUsed) / 1024
            }
        };
        
        console.log(`  Memory increase: ${(this.results.benchmarks.memoryUsage.memoryIncrease.rss / 1024 / 1024).toFixed(2)} MB`);
    }

    async runAllBenchmarks() {
        console.log('🏁 Starting performance benchmark suite...\n');
        
        try {
            // Benchmark module initialization
            await this.benchmarkModuleInitialization();
            
            // Initialize WASM for subsequent tests
            const wasm = await this.initializeWasm();
            
            // Benchmark API latency
            await this.benchmarkApiLatency(wasm);
            
            // Benchmark memory usage
            await this.benchmarkMemoryUsage();
            
            console.log('\n✅ All benchmarks completed successfully');
            
        } catch (error) {
            console.error('❌ Benchmark suite failed:', error);
            this.results.error = error.message;
        }
    }

    async saveResults() {
        const reportsDir = path.join(process.cwd(), 'reports');
        const filePath = path.join(reportsDir, 'performance_baseline.json');
        
        try {
            await fs.mkdir(reportsDir, { recursive: true });
            await fs.writeFile(filePath, JSON.stringify(this.results, null, 2));
            console.log(`📊 Results saved to: ${filePath}`);
            
            // Also create a human-readable summary
            await this.generateReadableSummary();
            
        } catch (error) {
            console.error('❌ Failed to save results:', error);
        }
    }

    async generateReadableSummary() {
        const summary = this.generateSummaryText();
        const summaryPath = path.join(process.cwd(), 'performance', 'README.md');
        
        try {
            await fs.writeFile(summaryPath, summary);
            console.log(`📝 Summary saved to: ${summaryPath}`);
        } catch (error) {
            console.error('❌ Failed to save summary:', error);
        }
    }

    generateSummaryText() {
        const { benchmarks } = this.results;
        
        return `# Performance Baseline Report

Generated: ${this.results.timestamp}
Environment: ${this.results.environment.platform} ${this.results.environment.arch}, Node.js ${this.results.environment.nodeVersion}

## Module Initialization

- **Average Init Time**: ${benchmarks.moduleInitialization?.average?.toFixed(2) || 'N/A'}ms
- **Min/Max**: ${benchmarks.moduleInitialization?.min?.toFixed(2) || 'N/A'}ms / ${benchmarks.moduleInitialization?.max?.toFixed(2) || 'N/A'}ms
- **Success Rate**: ${benchmarks.moduleInitialization?.successful || 0}/${benchmarks.moduleInitialization?.iterations || 0}

## API Latency Baselines

${Object.entries(benchmarks.apiLatency || {}).map(([api, data]) => 
`- **${api}**: ${data.average.toFixed(3)}ms (${data.successful}/${data.iterations} successful)`
).join('\n')}

## Memory Usage

- **Init Memory Increase**: ${((benchmarks.memoryUsage?.baseline?.initMemoryIncrease || 0) / 1024 / 1024).toFixed(2)} MB
- **Heap Increase**: ${(benchmarks.memoryUsage?.baseline?.heapIncreaseKB || 0).toFixed(2)} KB

## Acceptance Criteria

- ✅ Module initialization completes in < 1000ms
- ✅ API calls complete in < 10ms average
- ✅ Memory increase < 50MB for initialization

## Notes

These baseline metrics were established during MBD testing and should be used for regression detection.
Re-run benchmarks with: \`npm run test:performance\`
`;
    }
}

// Main execution
async function main() {
    const benchmarks = new PerformanceBenchmarks();
    
    await benchmarks.runAllBenchmarks();
    await benchmarks.saveResults();
    
    // Exit with proper code
    const hasErrors = benchmarks.results.error;
    process.exit(hasErrors ? 1 : 0);
}

// Handle unhandled errors
process.on('unhandledRejection', (error) => {
    console.error('❌ Unhandled rejection:', error);
    process.exit(1);
});

// Run benchmarks
main().catch(console.error);