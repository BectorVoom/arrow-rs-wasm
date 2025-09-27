# Feather Compression Testing Guide

This document provides comprehensive guidance for testing compressed Feather file functionality in the Arrow WASM library.

## Overview

The Feather compression testing suite validates:
- Compressed Feather file creation (LZ4, ZSTD, uncompressed)
- Round-trip data integrity
- Compression ratios and performance
- Cross-compatibility with other Arrow implementations
- Edge case handling

## Quick Start

1. **Start the server:**
   ```bash
   cd browser
   python3 server.py
   ```

2. **Open the test page:**
   Navigate to `http://localhost:8080/browser/feather-compression-test.html`

3. **Run basic tests:**
   - Select a test dataset from the dropdown
   - Click "Test All Compression Types" to run comprehensive tests
   - View results in the output console and results table

## Test Components

### 1. Test Data Generator (`feather-test-data.js`)

Generates various types of test datasets:

- **Small Mixed Data**: Basic dataset with different data types
- **Repetitive Data**: Large datasets with repeated values (good for compression)
- **Sparse Data**: Datasets with many null values
- **Wide Data**: Tables with many columns
- **Highly Compressible**: Data designed to compress very well
- **Random Data**: Data that compresses poorly
- **Edge Cases**: Special characters, Unicode, extreme values

### 2. Compression Tester (`test-feather-compression.js`)

Main testing engine that provides:

- Single compression type testing
- Comprehensive compression comparison
- Performance benchmarking
- Round-trip validation
- File upload/download support

### 3. Browser Interface (`feather-compression-test.html`)

User-friendly interface with:

- Dataset selection and preview
- Compression test controls
- Real-time progress tracking
- Results visualization
- File download capabilities

## Test Scenarios

### Basic Functionality Tests

**Purpose**: Verify core compression features work correctly

**Steps**:
1. Select "Small Mixed Data" dataset
2. Click "Test All Compression Types"
3. Verify all three compression types (None, LZ4, ZSTD) complete successfully
4. Check data integrity shows "PASS" for all tests

**Expected Results**:
- All compression types complete without errors
- Data integrity verification passes
- LZ4 and ZSTD show some compression (ratio > 1.0)
- Files are available for download

### Compression Ratio Tests

**Purpose**: Validate compression effectiveness

**Test Datasets**:
- **Highly Compressible**: Should achieve 5x+ compression ratios
- **Random Data**: Should achieve <2x compression ratios
- **Repetitive Data**: Should achieve 3-10x compression ratios

**Expected Compression Ratios**:
- **Uncompressed**: 1.0x (baseline)
- **LZ4**: 1.5-5x (fast compression)
- **ZSTD**: 2-10x (better compression, slower)

### Performance Tests

**Purpose**: Measure read/write performance across different data sizes

**Steps**:
1. Click "Performance Benchmark"
2. Monitor write/read times for different dataset sizes
3. Compare performance across compression types

**Expected Performance**:
- Write times: None < LZ4 < ZSTD
- Read times: Should be similar across compression types
- Larger datasets should show proportional performance scaling

### Edge Case Tests

**Purpose**: Ensure robustness with unusual data

**Test Cases**:
- Single row tables
- Tables with null values
- Unicode characters
- Empty strings
- Very large values

**Expected Results**:
- No crashes or errors
- Data integrity maintained
- Appropriate error messages for unsupported cases

### Round-Trip Tests

**Purpose**: Verify complete data preservation

**Process**:
1. JSON → Table → Compressed Feather → Table → JSON
2. Compare original and final JSON data
3. Validate schema preservation
4. Check null value handling

### File Upload Tests

**Purpose**: Test compatibility with external Arrow files

**Steps**:
1. Upload a Feather file created by another Arrow implementation
2. Verify successful reading
3. Test re-compression
4. Validate schema and data

## Troubleshooting

### Common Issues

**Module Loading Errors**:
- Ensure WASM package is built: `wasm-pack build`
- Check browser console for detailed error messages
- Verify server is serving files with correct MIME types

**Compression Failures**:
- LZ4/ZSTD may not be available in all build configurations
- Check if compression features are enabled in Cargo.toml
- Verify Arrow version supports desired compression

**Memory Errors**:
- Large datasets may exceed browser memory limits
- Try smaller datasets for initial testing
- Monitor browser memory usage in developer tools

**Data Integrity Failures**:
- Check for floating-point precision issues
- Verify null value handling
- Review Unicode character support

**Performance Issues**:
- Large datasets (>10K rows) may be slow
- Consider using smaller datasets for development
- Monitor browser performance in developer tools

### Browser Compatibility

**Recommended Browsers**:
- Chrome 80+
- Firefox 75+
- Safari 14+
- Edge 80+

**Required Features**:
- WebAssembly support
- ES6 modules
- Sufficient memory for large datasets

### Debugging Tips

**Enable Verbose Logging**:
- Open browser developer console
- All test operations are logged with timestamps
- Error messages include stack traces

**Memory Monitoring**:
- Use browser dev tools "Memory" tab
- Look for memory leaks after repeated tests
- Ensure `table.dispose()` is called

**Performance Profiling**:
- Use browser dev tools "Performance" tab
- Identify bottlenecks in compression/decompression
- Monitor WASM memory allocation

## Test Data Specifications

### Dataset Sizes

| Dataset | Rows | Columns | Est. Size | Use Case |
|---------|------|---------|-----------|----------|
| Small Mixed | 5 | 5 | <1KB | Basic functionality |
| Repetitive 1K | 1,000 | 9 | ~50KB | Medium compression |
| Repetitive 5K | 5,000 | 9 | ~250KB | Large compression |
| Sparse Data | 500 | 5 | ~25KB | Null handling |
| Wide Data | 100 | 50 | ~50KB | Many columns |
| Highly Compressible | 2,000 | 6 | ~100KB | Max compression |
| Random Data | 1,000 | 6 | ~75KB | Min compression |

### Expected Compression Ratios

| Data Type | None | LZ4 | ZSTD |
|-----------|------|-----|------|
| Repetitive | 1.0x | 3-5x | 5-8x |
| Random | 1.0x | 1.2-1.5x | 1.3-1.8x |
| Sparse | 1.0x | 2-3x | 3-5x |
| Mixed | 1.0x | 1.5-2.5x | 2-3.5x |

## API Reference

### FeatherTestDataGenerator

```javascript
// Get all test datasets
const datasets = FeatherTestDataGenerator.getAllTestDatasets();

// Generate specific dataset types
const smallData = FeatherTestDataGenerator.generateSmallMixedData();
const largeData = FeatherTestDataGenerator.generateRepetitiveData(5000);
const sparseData = FeatherTestDataGenerator.generateSparseData(1000);

// Get dataset statistics
const stats = FeatherTestDataGenerator.getDatasetStats(data);
```

### FeatherCompressionTester

```javascript
// Initialize tester
const tester = new FeatherCompressionTester(wasmModule, logger, progressCallback);

// Test single compression type
await tester.testCompression(dataset, 'lz4');

// Test all compression types
await tester.testAllCompressions(dataset);

// Performance benchmark
await tester.performanceBenchmark();

// Round-trip test
await tester.completeRoundTripTest(dataset);

// Get test results
const results = tester.getTestResults();
```

### Write Options

```javascript
// Create write options for different compression types
const options = wasm.WriteOptions.new();

// Set compression type
options.set_compression(wasm.CompressionType.LZ4);    // LZ4 compression
options.set_compression(wasm.CompressionType.ZSTD);   // ZSTD compression  
options.set_compression(wasm.CompressionType.None);   // No compression
```

## Best Practices

### Test Development

1. **Start Small**: Begin with small datasets before testing large ones
2. **Validate Early**: Check data integrity on every test
3. **Monitor Memory**: Watch for memory leaks in repeated tests
4. **Handle Errors**: Gracefully handle compression failures
5. **Document Results**: Save test results for comparison

### Performance Optimization

1. **Batch Operations**: Process multiple files together when possible
2. **Memory Management**: Dispose of tables when finished
3. **Compression Selection**: Choose compression based on use case:
   - **None**: Fastest, largest files
   - **LZ4**: Good balance of speed and compression
   - **ZSTD**: Best compression, slower

### Production Deployment

1. **Test All Browsers**: Verify compatibility across target browsers
2. **Measure Performance**: Benchmark with realistic data sizes
3. **Error Handling**: Implement robust error recovery
4. **Memory Limits**: Set appropriate limits for dataset sizes
5. **Progress Feedback**: Provide user feedback for long operations

## Example Test Session

```javascript
// 1. Initialize
const module = await init();
await module.initialize();
const tester = new FeatherCompressionTester(module, console.log, updateProgress);

// 2. Generate test data
const dataset = {
    name: 'Test Dataset',
    data: FeatherTestDataGenerator.generateRepetitiveData(1000)
};

// 3. Run comprehensive tests
const results = await tester.testAllCompressions(dataset);

// 4. Validate results
console.log('Compression ratios:', results.map(r => 
    `${r.compression}: ${r.compressionRatio.toFixed(2)}x`
));

// 5. Download files
// Files automatically available for download in UI
```

## Support and Issues

For issues with the testing framework:

1. Check browser console for error messages
2. Verify WASM module is properly built and loaded
3. Test with smaller datasets to isolate issues
4. Review this documentation for troubleshooting guidance

For Arrow WASM library issues:
- Check the main project README
- Review API documentation
- Test with the basic Arrow functionality first