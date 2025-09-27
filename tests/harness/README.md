# Test Automation Framework

This directory contains the test execution infrastructure and automation scripts for running the model-derived test suite across multiple browser environments.

## Components

### Browser Automation (`browser/`)
- **Headless Testing**: Puppeteer/Playwright scripts for CI environments
- **Cross-Browser Support**: Chrome, Firefox, Safari automation
- **Mobile Testing**: iOS Safari, Chrome Mobile compatibility
- **Performance Monitoring**: Automated performance data collection

### Test Runners (`runners/`)
- **Local Development**: Scripts for running tests during development
- **CI Integration**: Automated test execution for continuous integration
- **Parallel Execution**: Multi-browser concurrent testing
- **Failure Isolation**: Individual test isolation and retry logic

### Environment Setup (`setup/`)
- **Test Server**: Local HTTP server for serving WASM modules
- **Mock Data**: Test data generation and fixtures
- **Configuration**: Environment-specific test configurations
- **Utilities**: Helper functions and test utilities

## Usage

### Local Development
```bash
# Start test server
npm run test:server

# Run tests in headed browser (for debugging)
npm run test:headed

# Run specific test suite
npm run test:suite table_operations

# Watch mode for development
npm run test:watch
```

### Cross-Browser Testing
```bash
# Run on all supported browsers
npm run test:browsers

# Run on specific browser
npm run test:chrome
npm run test:firefox
npm run test:safari

# Mobile browser testing
npm run test:mobile
```

### CI Integration
```bash
# Full test suite (headless)
npm run test:ci

# Generate reports
npm run test:report

# Performance benchmarks
npm run test:performance
```

## Test Execution Flow

1. **Environment Setup**: Start test server, load WASM module
2. **Test Discovery**: Load generated test cases from `/generated/`
3. **Browser Launch**: Initialize browser automation
4. **Test Execution**: Run tests with proper isolation
5. **Result Collection**: Gather pass/fail results, logs, artifacts
6. **Report Generation**: Create detailed test reports
7. **Cleanup**: Close browsers, stop servers

## Browser Compatibility Matrix

| Browser | Version | Status | Notes |
|---------|---------|---------|-------|
| Chrome | ≥80 | ✅ Full Support | Primary development target |
| Firefox | ≥75 | ✅ Full Support | Complete WASM compatibility |
| Safari | ≥14 | ✅ Full Support | WebAssembly support verified |
| Edge | ≥80 | ✅ Full Support | Chromium-based compatibility |
| iOS Safari | ≥14 | ⚠️ Basic Support | Mobile-specific testing |
| Chrome Mobile | ≥80 | ⚠️ Basic Support | Mobile-specific testing |

## Failure Analysis

When tests fail, the framework automatically captures:
- **Console Logs**: All browser console output
- **Network Traces**: HTTP requests and responses
- **Stack Traces**: WASM and JavaScript stack traces
- **DOM Snapshots**: Page state at failure time
- **Screenshots**: Visual state for debugging
- **Performance Data**: Timing and memory usage

## Configuration

### Test Configuration (`config/test.json`)
```json
{
  "browsers": ["chrome", "firefox", "safari"],
  "headless": true,
  "timeout": 30000,
  "retries": 2,
  "parallel": true,
  "performance": {
    "enabled": true,
    "metrics": ["init_time", "api_latency", "memory_usage"]
  }
}
```

### Browser-Specific Settings
- **Chrome**: Optimized for performance testing
- **Firefox**: Standard compliance validation
- **Safari**: WebKit compatibility verification
- **Mobile**: Touch and viewport considerations