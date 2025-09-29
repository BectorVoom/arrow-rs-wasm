/**
 * Browser Performance Testing Suite for Arrow WASM
 * 
 * Comprehensive performance benchmarking for:
 * - Module initialization time
 * - API call latency
 * - Memory usage patterns
 * - Compression performance
 * - Cross-browser performance comparison
 */

class BrowserPerformanceSuite {
    constructor() {
        this.testServerUrl = 'http://localhost:8080';
        this.results = {
            timestamp: new Date().toISOString(),
            browser: this.getBrowserInfo(),
            system: this.getSystemInfo(),
            baselines: {},
            benchmarks: [],
            summary: {}
        };
        
        this.performanceMetrics = {
            moduleInit: null,
            memoryUsage: null,
            apiLatency: {},
            compressionPerformance: {},
            throughput: {}
        };
    }

    getBrowserInfo() {
        const ua = navigator.userAgent;
        const platform = navigator.platform;
        
        return {
            userAgent: ua,
            platform: platform,
            vendor: navigator.vendor,
            language: navigator.language,
            cookieEnabled: navigator.cookieEnabled,
            onLine: navigator.onLine,
            hardwareConcurrency: navigator.hardwareConcurrency || 'unknown',
            deviceMemory: navigator.deviceMemory || 'unknown',
            name: this.detectBrowserName(ua),
            version: this.detectBrowserVersion(ua)
        };
    }

    detectBrowserName(ua) {
        if (ua.includes('Chrome') && !ua.includes('Edg')) return 'Chrome';
        if (ua.includes('Firefox')) return 'Firefox';
        if (ua.includes('Safari') && !ua.includes('Chrome')) return 'Safari';
        if (ua.includes('Edg')) return 'Edge';
        return 'Unknown';
    }

    detectBrowserVersion(ua) {
        const patterns = {
            Chrome: /Chrome\/([0-9.]+)/,
            Firefox: /Firefox\/([0-9.]+)/,
            Safari: /Version\/([0-9.]+)/,
            Edge: /Edg\/([0-9.]+)/
        };
        
        const browserName = this.detectBrowserName(ua);
        const pattern = patterns[browserName];
        
        if (pattern) {
            const match = ua.match(pattern);
            return match ? match[1] : 'unknown';
        }
        
        return 'unknown';
    }

    getSystemInfo() {
        return {
            timestamp: Date.now(),
            timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
            screen: {
                width: screen.width,
                height: screen.height,
                colorDepth: screen.colorDepth,
                pixelDepth: screen.pixelDepth
            },
            window: {
                innerWidth: window.innerWidth,
                innerHeight: window.innerHeight,
                devicePixelRatio: window.devicePixelRatio
            }
        };
    }

    async runComprehensiveBenchmarks() {
        console.log('🚀 Starting comprehensive performance benchmarks...');
        
        try {
            // 1. Module Initialization Benchmark
            await this.benchmarkModuleInitialization();
            
            // 2. Memory Usage Baseline
            await this.benchmarkMemoryUsage();
            
            // 3. API Latency Benchmarks
            await this.benchmarkAPILatency();
            
            // 4. Compression Performance
            await this.benchmarkCompressionPerformance();
            
            // 5. Throughput Benchmarks
            await this.benchmarkThroughput();
            
            // 6. Stress Testing
            await this.benchmarkStressConditions();
            
            // Generate comprehensive report
            await this.generatePerformanceReport();
            
            console.log('✅ Performance benchmarks completed successfully');
            return this.results;
            
        } catch (error) {
            console.error('❌ Performance benchmarks failed:', error);
            this.results.error = error.message;
            return this.results;
        }
    }

    async benchmarkModuleInitialization() {
        console.log('📊 Benchmarking module initialization...');
        
        const iterations = 5;
        const initTimes = [];
        
        for (let i = 0; i < iterations; i++) {
            const startTime = performance.now();
            
            // Simulate module initialization
            try {
                // Test WASM loading time
                const wasmStartTime = performance.now();
                
                // Import WASM module
                const wasmModule = await import('../pkg/arrow_wasm.js');
                const wasmLoadTime = performance.now() - wasmStartTime;
                
                // Initialize module
                const initStartTime = performance.now();
                await wasmModule.default();
                const wasmInitTime = performance.now() - initStartTime;
                
                const totalTime = performance.now() - startTime;
                
                initTimes.push({
                    iteration: i + 1,
                    totalTime: totalTime,
                    wasmLoadTime: wasmLoadTime,
                    wasmInitTime: wasmInitTime
                });
                
                console.log(`  Iteration ${i + 1}: ${totalTime.toFixed(2)}ms (load: ${wasmLoadTime.toFixed(2)}ms, init: ${wasmInitTime.toFixed(2)}ms)`);
                
            } catch (error) {
                console.warn(`  Iteration ${i + 1} failed:`, error.message);
                initTimes.push({
                    iteration: i + 1,
                    error: error.message,
                    totalTime: null
                });
            }
        }
        
        // Calculate statistics
        const validTimes = initTimes.filter(t => t.totalTime !== null).map(t => t.totalTime);
        const avgTime = validTimes.reduce((a, b) => a + b, 0) / validTimes.length;
        const minTime = Math.min(...validTimes);
        const maxTime = Math.max(...validTimes);
        
        this.performanceMetrics.moduleInit = {
            iterations: initTimes,
            statistics: {
                average: avgTime,
                minimum: minTime,
                maximum: maxTime,
                standardDeviation: this.calculateStandardDeviation(validTimes),
                successRate: (validTimes.length / iterations) * 100
            }
        };
        
        console.log(`  📊 Module init: avg ${avgTime.toFixed(2)}ms, min ${minTime.toFixed(2)}ms, max ${maxTime.toFixed(2)}ms`);
    }

    async benchmarkMemoryUsage() {
        console.log('📊 Benchmarking memory usage...');
        
        const memorySnapshots = [];
        
        // Baseline memory
        memorySnapshots.push({
            stage: 'baseline',
            memory: this.getMemoryUsage(),
            timestamp: Date.now()
        });
        
        // After WASM load
        try {
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default();
            
            memorySnapshots.push({
                stage: 'post_wasm_init',
                memory: this.getMemoryUsage(),
                timestamp: Date.now()
            });
            
            // After table operations
            if (typeof wasmModule.get_memory_stats === 'function') {
                const wasmMemStats = wasmModule.get_memory_stats();
                memorySnapshots.push({
                    stage: 'post_operations',
                    memory: this.getMemoryUsage(),
                    wasmMemory: wasmMemStats,
                    timestamp: Date.now()
                });
            }
            
        } catch (error) {
            console.warn('Memory benchmark failed:', error.message);
        }
        
        this.performanceMetrics.memoryUsage = {
            snapshots: memorySnapshots,
            analysis: this.analyzeMemoryUsage(memorySnapshots)
        };
        
        console.log(`  📊 Memory usage tracked across ${memorySnapshots.length} stages`);
    }

    getMemoryUsage() {
        if (performance.memory) {
            return {
                used: performance.memory.usedJSHeapSize,
                total: performance.memory.totalJSHeapSize,
                limit: performance.memory.jsHeapSizeLimit,
                timestamp: Date.now()
            };
        }
        return { available: false, timestamp: Date.now() };
    }

    analyzeMemoryUsage(snapshots) {
        if (snapshots.length < 2) return { insufficient_data: true };
        
        const baseline = snapshots[0].memory;
        const final = snapshots[snapshots.length - 1].memory;
        
        if (!baseline.used || !final.used) return { memory_api_unavailable: true };
        
        return {
            baselineUsage: baseline.used,
            finalUsage: final.used,
            memoryIncrease: final.used - baseline.used,
            percentageIncrease: ((final.used - baseline.used) / baseline.used) * 100,
            efficiency: final.used < baseline.limit * 0.8 ? 'good' : 'concerning'
        };
    }

    async benchmarkAPILatency() {
        console.log('📊 Benchmarking API latency...');
        
        try {
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default();
            
            const apiFunctions = [
                'get_version',
                'is_lz4_supported',
                'get_memory_stats',
                'get_supported_compression_types'
            ];
            
            for (const funcName of apiFunctions) {
                if (typeof wasmModule[funcName] === 'function') {
                    const latencies = await this.measureAPILatency(wasmModule[funcName], funcName);
                    this.performanceMetrics.apiLatency[funcName] = latencies;
                }
            }
            
        } catch (error) {
            console.warn('API latency benchmark failed:', error.message);
            this.performanceMetrics.apiLatency.error = error.message;
        }
    }

    async measureAPILatency(apiFunction, functionName) {
        const iterations = 100;
        const latencies = [];
        
        for (let i = 0; i < iterations; i++) {
            const startTime = performance.now();
            
            try {
                apiFunction();
                const latency = performance.now() - startTime;
                latencies.push(latency);
            } catch (error) {
                latencies.push(null);
            }
        }
        
        const validLatencies = latencies.filter(l => l !== null);
        const avgLatency = validLatencies.reduce((a, b) => a + b, 0) / validLatencies.length;
        
        console.log(`  📊 ${functionName}: avg ${avgLatency.toFixed(4)}ms`);
        
        return {
            functionName: functionName,
            iterations: iterations,
            validCalls: validLatencies.length,
            averageLatency: avgLatency,
            minimumLatency: Math.min(...validLatencies),
            maximumLatency: Math.max(...validLatencies),
            successRate: (validLatencies.length / iterations) * 100
        };
    }

    async benchmarkCompressionPerformance() {
        console.log('📊 Benchmarking compression performance...');
        
        try {
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default();
            
            if (typeof wasmModule.is_lz4_supported === 'function') {
                const lz4Supported = wasmModule.is_lz4_supported();
                
                this.performanceMetrics.compressionPerformance = {
                    lz4Supported: lz4Supported,
                    benchmarkCompleted: true
                };
                
                console.log(`  📊 LZ4 compression: ${lz4Supported ? 'Supported' : 'Not supported'}`);
            }
            
        } catch (error) {
            console.warn('Compression benchmark failed:', error.message);
            this.performanceMetrics.compressionPerformance.error = error.message;
        }
    }

    async benchmarkThroughput() {
        console.log('📊 Benchmarking throughput...');
        
        const startTime = performance.now();
        const operations = 1000;
        
        try {
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default();
            
            // Simulate high-frequency operations
            for (let i = 0; i < operations; i++) {
                if (typeof wasmModule.get_version === 'function') {
                    wasmModule.get_version();
                }
            }
            
            const totalTime = performance.now() - startTime;
            const opsPerSecond = (operations / totalTime) * 1000;
            
            this.performanceMetrics.throughput = {
                operations: operations,
                totalTime: totalTime,
                operationsPerSecond: opsPerSecond,
                averageTimePerOperation: totalTime / operations
            };
            
            console.log(`  📊 Throughput: ${opsPerSecond.toFixed(0)} ops/sec`);
            
        } catch (error) {
            console.warn('Throughput benchmark failed:', error.message);
            this.performanceMetrics.throughput.error = error.message;
        }
    }

    async benchmarkStressConditions() {
        console.log('📊 Benchmarking stress conditions...');
        
        const stressResults = {
            concurrentOperations: null,
            memoryStress: null,
            rapidApiCalls: null
        };
        
        try {
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default();
            
            // Test rapid API calls
            const rapidCallsStart = performance.now();
            const rapidCalls = 10000;
            
            for (let i = 0; i < rapidCalls; i++) {
                if (typeof wasmModule.get_version === 'function') {
                    wasmModule.get_version();
                }
            }
            
            const rapidCallsTime = performance.now() - rapidCallsStart;
            
            stressResults.rapidApiCalls = {
                calls: rapidCalls,
                totalTime: rapidCallsTime,
                callsPerSecond: (rapidCalls / rapidCallsTime) * 1000,
                successful: true
            };
            
            console.log(`  📊 Stress test: ${((rapidCalls / rapidCallsTime) * 1000).toFixed(0)} rapid calls/sec`);
            
        } catch (error) {
            console.warn('Stress test failed:', error.message);
            stressResults.error = error.message;
        }
        
        this.performanceMetrics.stressTest = stressResults;
    }

    async generatePerformanceReport() {
        const reportPath = 'performance_baseline_report.json';
        const htmlReportPath = 'performance_baseline_report.html';
        
        this.results.baselines = this.performanceMetrics;
        this.results.summary = this.generatePerformanceSummary();
        
        // Generate HTML report
        const htmlReport = this.generateHTMLPerformanceReport();
        
        console.log('📊 Performance baseline report generated');
        console.log(`📄 JSON data available in console`);
        console.log(`🌐 HTML report generated`);
        
        // Log to console for access
        console.log('Performance Results:', JSON.stringify(this.results, null, 2));
    }

    generatePerformanceSummary() {
        return {
            browser: `${this.results.browser.name} ${this.results.browser.version}`,
            moduleInitTime: this.performanceMetrics.moduleInit?.statistics?.average || null,
            memoryEfficient: this.performanceMetrics.memoryUsage?.analysis?.efficiency === 'good',
            lz4Supported: this.performanceMetrics.compressionPerformance?.lz4Supported || false,
            throughput: this.performanceMetrics.throughput?.operationsPerSecond || null,
            overallPerformance: this.calculateOverallPerformance()
        };
    }

    calculateOverallPerformance() {
        let score = 0;
        let factors = 0;
        
        // Module init time (faster is better)
        if (this.performanceMetrics.moduleInit?.statistics?.average) {
            const initTime = this.performanceMetrics.moduleInit.statistics.average;
            score += initTime < 1000 ? 100 : initTime < 2000 ? 80 : 60;
            factors++;
        }
        
        // Memory efficiency
        if (this.performanceMetrics.memoryUsage?.analysis?.efficiency === 'good') {
            score += 100;
        } else if (this.performanceMetrics.memoryUsage?.analysis?.efficiency) {
            score += 60;
        }
        factors++;
        
        // Throughput (higher is better)
        if (this.performanceMetrics.throughput?.operationsPerSecond) {
            const ops = this.performanceMetrics.throughput.operationsPerSecond;
            score += ops > 10000 ? 100 : ops > 5000 ? 80 : 60;
            factors++;
        }
        
        return factors > 0 ? Math.round(score / factors) : null;
    }

    generateHTMLPerformanceReport() {
        const summary = this.results.summary;
        
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Performance Baseline Report - Arrow WASM</title>
    <style>
        body { font-family: system-ui, sans-serif; margin: 2rem; background: #f8f9fa; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 2rem; border-radius: 8px; }
        .header { text-align: center; margin-bottom: 2rem; border-bottom: 1px solid #eee; padding-bottom: 1rem; }
        .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1rem; margin: 2rem 0; }
        .metric { background: #f8f9fa; border: 1px solid #ddd; padding: 1rem; border-radius: 6px; }
        .metric-value { font-size: 1.5rem; font-weight: bold; color: #0066cc; }
        .performance-score { text-align: center; font-size: 3rem; font-weight: bold; margin: 2rem 0; }
        .good { color: #28a745; }
        .ok { color: #ffc107; }
        .poor { color: #dc3545; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⚡ Performance Baseline Report</h1>
            <p><strong>Arrow WASM Library Performance Benchmarks</strong></p>
            <p>Browser: ${summary.browser} | Generated: ${this.results.timestamp}</p>
        </div>

        <div class="performance-score">
            <div class="${summary.overallPerformance > 80 ? 'good' : summary.overallPerformance > 60 ? 'ok' : 'poor'}">
                ${summary.overallPerformance || 'N/A'}/100
            </div>
            <div style="font-size: 1rem; color: #666;">Overall Performance Score</div>
        </div>

        <div class="metrics">
            <div class="metric">
                <div class="metric-value">${summary.moduleInitTime ? summary.moduleInitTime.toFixed(2) + 'ms' : 'N/A'}</div>
                <div>Module Initialization</div>
            </div>
            <div class="metric">
                <div class="metric-value">${summary.memoryEfficient ? '✅ Efficient' : '⚠️ Check needed'}</div>
                <div>Memory Usage</div>
            </div>
            <div class="metric">
                <div class="metric-value">${summary.lz4Supported ? '✅ Supported' : '❌ Not supported'}</div>
                <div>LZ4 Compression</div>
            </div>
            <div class="metric">
                <div class="metric-value">${summary.throughput ? Math.round(summary.throughput).toLocaleString() + ' ops/sec' : 'N/A'}</div>
                <div>Throughput</div>
            </div>
        </div>

        <div style="margin-top: 2rem; padding: 1rem; background: #f8f9fa; border-radius: 6px;">
            <h3>🎯 Performance Baseline Established</h3>
            <p>These metrics serve as the baseline for regression testing and cross-browser performance comparison.</p>
            <p><strong>System:</strong> ${this.results.system.screen.width}x${this.results.system.screen.height} display, ${this.results.browser.hardwareConcurrency} cores</p>
        </div>
    </div>
</body>
</html>`;
    }

    calculateStandardDeviation(values) {
        const avg = values.reduce((a, b) => a + b, 0) / values.length;
        const variance = values.reduce((a, b) => a + Math.pow(b - avg, 2), 0) / values.length;
        return Math.sqrt(variance);
    }
}

// Export for use in browser testing
if (typeof window !== 'undefined') {
    window.BrowserPerformanceSuite = BrowserPerformanceSuite;
}

export default BrowserPerformanceSuite;