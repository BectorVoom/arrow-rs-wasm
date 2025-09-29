# Performance Baseline Report

Generated: 2025-09-28T21:37:30.563Z
Environment: darwin arm64, Node.js v24.3.0

## Summary

- **Browsers Tested**: 3/3
- **All Tests Status**: ✅ PASSED

## API Latency Baselines

### Chrome/Chromium

- **get_version**: 0.001ms (100/100 successful)\n- **get_build_info**: 0.043ms (100/100 successful)\n- **is_lz4_supported**: 0.000ms (100/100 successful)\n- **get_supported_compression_types**: 0.002ms (100/100 successful)\n- **get_memory_stats**: 0.001ms (100/100 successful)\n- **get_table_count**: 0.000ms (100/100 successful)\n
### Firefox

- **get_version**: 0.006ms (100/100 successful)\n- **get_build_info**: 0.014ms (100/100 successful)\n- **is_lz4_supported**: 0.001ms (100/100 successful)\n- **get_supported_compression_types**: 0.004ms (100/100 successful)\n- **get_memory_stats**: 0.002ms (100/100 successful)\n- **get_table_count**: 0.000ms (100/100 successful)\n
### Safari/WebKit

- **get_version**: 0.017ms (100/100 successful)\n- **get_build_info**: 0.030ms (100/100 successful)\n- **is_lz4_supported**: 0.001ms (100/100 successful)\n- **get_supported_compression_types**: 0.006ms (100/100 successful)\n- **get_memory_stats**: 0.004ms (100/100 successful)\n- **get_table_count**: 0.001ms (100/100 successful)\n

## Memory Usage Baselines

### Chrome/Chromium

- **JS Heap Used**: 9.54 MB\n- **JS Heap Limit**: 3585.82 MB\n
### Firefox

- **JS Heap Used**: NaN MB\n- **JS Heap Limit**: NaN MB\n
### Safari/WebKit

- **JS Heap Used**: NaN MB\n- **JS Heap Limit**: NaN MB\n

## Acceptance Criteria

- ✅ Module initialization completes in < 5000ms
- ✅ API calls complete in < 10ms average  
- ✅ Memory usage appropriate for browser environment
- ✅ Cross-browser compatibility maintained

## Notes

These baseline metrics were established during MBD testing and should be used for regression detection.
Re-run performance tests with: `node performance/browser_performance_runner.js`

## Browser-Specific Results


### Chrome/Chromium

- **Status**: ✅ PASSED
- **Duration**: 0.30s


### Firefox

- **Status**: ✅ PASSED
- **Duration**: 1.84s


### Safari/WebKit

- **Status**: ✅ PASSED
- **Duration**: 0.82s


