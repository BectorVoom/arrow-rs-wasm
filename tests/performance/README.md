# Performance Benchmarks and Baseline Metrics

This directory contains performance testing infrastructure and baseline metrics for the WASM Arrow library, ensuring consistent performance characteristics across browsers and detecting regressions.

## Performance Categories

### 1. Module Initialization (`init/`)
- **WASM Load Time**: Time to download and instantiate WASM module
- **Memory Setup**: Initial memory allocation and setup overhead
- **API Preparation**: Time to prepare JavaScript bindings

### 2. API Operation Latency (`operations/`)
- **Table Creation**: `tableFromJSON()` and `tableFromIPC()` performance
- **Data Access**: Column access and value retrieval timings
- **Data Conversion**: Arrow ↔ JavaScript conversion overhead
- **Memory Operations**: Handle allocation and disposal timing

### 3. Throughput Benchmarks (`throughput/`)
- **Large Dataset Handling**: Performance with various data sizes
- **Batch Processing**: Multi-batch operation efficiency
- **Streaming Performance**: Chunked data processing rates

### 4. Memory Characteristics (`memory/`)
- **Memory Usage Patterns**: Heap growth during operations
- **Leak Detection**: Long-running operation memory stability
- **Garbage Collection**: Impact of disposal operations

## Baseline Metrics

### Chrome 120+ (Primary Target)
```json
{
  "module_init": {
    "wasm_load": "< 50ms",
    "memory_setup": "< 10ms",
    "api_ready": "< 100ms"
  },
  "operations": {
    "table_from_json_1k": "< 5ms",
    "table_from_json_10k": "< 25ms",
    "column_access": "< 1ms",
    "value_retrieval": "< 0.1ms",
    "to_array_1k": "< 2ms"
  },
  "memory": {
    "baseline_usage": "< 2MB",
    "per_table_1k": "< 100KB",
    "leak_threshold": "0 bytes/operation"
  }
}
```

### Firefox 115+ (Secondary Target)
```json
{
  "module_init": {
    "wasm_load": "< 60ms",
    "memory_setup": "< 15ms", 
    "api_ready": "< 120ms"
  },
  "operations": {
    "table_from_json_1k": "< 8ms",
    "table_from_json_10k": "< 35ms",
    "column_access": "< 1.5ms",
    "value_retrieval": "< 0.2ms",
    "to_array_1k": "< 3ms"
  }
}
```

### Safari 17+ (Compatibility Target)
```json
{
  "module_init": {
    "wasm_load": "< 80ms",
    "memory_setup": "< 20ms",
    "api_ready": "< 150ms"
  },
  "operations": {
    "table_from_json_1k": "< 10ms",
    "table_from_json_10k": "< 45ms",
    "column_access": "< 2ms",
    "value_retrieval": "< 0.3ms",
    "to_array_1k": "< 4ms"
  }
}
```

## Benchmark Test Data

### Small Dataset (1K rows)
```javascript
const small_dataset = generateTestData({
  rows: 1000,
  columns: ["id", "name", "value", "active"],
  types: ["int32", "utf8", "float64", "bool"]
});
```

### Medium Dataset (10K rows)
```javascript
const medium_dataset = generateTestData({
  rows: 10000,
  columns: ["id", "timestamp", "sensor_value", "quality", "metadata"],
  types: ["int64", "timestamp", "float32", "uint8", "utf8"]
});
```

### Large Dataset (100K rows)
```javascript
const large_dataset = generateTestData({
  rows: 100000,
  columns: ["id", "location", "measurement", "tags"],
  types: ["int64", "utf8", "float64", "list<utf8>"]
});
```

## Performance Test Execution

### Automated Benchmarks
```bash
# Run all performance tests
npm run perf:all

# Run specific performance category
npm run perf:init
npm run perf:operations
npm run perf:memory

# Generate performance report
npm run perf:report

# Compare with baseline
npm run perf:compare
```

### Manual Performance Testing
```bash
# Interactive performance testing
npm run perf:interactive

# Profile specific operation
npm run perf:profile table_creation

# Memory leak detection
npm run perf:memory-check
```

## Regression Detection

### Automated Monitoring
- **CI Integration**: Performance tests run on every commit
- **Threshold Alerts**: Automatic alerts when metrics exceed baselines
- **Trend Analysis**: Performance trend tracking over time
- **Comparison Reports**: Before/after performance comparisons

### Performance Gates
- **Build Failure**: Block merges if performance degrades >20%
- **Warning Thresholds**: Alert if performance degrades >10%
- **Improvement Tracking**: Document performance improvements

## Tools and Utilities

### Performance Measurement (`measure/`)
- **High-Resolution Timing**: Microsecond precision measurements
- **Memory Profiling**: Heap usage tracking
- **Browser DevTools Integration**: Chrome/Firefox profiler integration

### Data Analysis (`analysis/`)
- **Statistical Analysis**: Mean, median, percentiles, standard deviation
- **Regression Analysis**: Performance trend detection
- **Comparison Tools**: Before/after analysis utilities

### Reporting (`reporting/`)
- **Performance Dashboards**: Visual performance metrics
- **Regression Reports**: Automated regression detection reports
- **Baseline Updates**: Tools for updating baseline metrics

## Best Practices

1. **Warm-up Runs**: Discard initial measurements to account for JIT compilation
2. **Multiple Samples**: Take multiple measurements and use statistical analysis
3. **Controlled Environment**: Minimize external factors affecting performance
4. **Consistent Test Data**: Use identical datasets across test runs
5. **Browser State**: Reset browser state between tests
6. **Resource Isolation**: Ensure adequate system resources during testing