
// Generated test suite runner
// Executes all model-derived tests

import { execSync } from 'child_process';
import path from 'path';

const testSuites = [
  {
    "model_id": "data_conversion_flow",
    "test_count": 40,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/data_conversion_flow_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/data_conversion_flow_tests.spec.js"
    ]
  },
  {
    "model_id": "array_builder_lifecycle",
    "test_count": 22,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/array_builder_lifecycle_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/array_builder_lifecycle_tests.spec.js"
    ]
  },
  {
    "model_id": "column_lifecycle",
    "test_count": 24,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/column_lifecycle_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/column_lifecycle_tests.spec.js"
    ]
  },
  {
    "model_id": "memory_management",
    "test_count": 22,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/memory_management_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/memory_management_tests.spec.js"
    ]
  },
  {
    "model_id": "module_lifecycle",
    "test_count": 21,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/module_lifecycle_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/module_lifecycle_tests.spec.js"
    ]
  },
  {
    "model_id": "table_lifecycle",
    "test_count": 25,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/table_lifecycle_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/table_lifecycle_tests.spec.js"
    ]
  },
  {
    "model_id": "error_handling_flow",
    "test_count": 33,
    "output_files": [
      "/Users/ods/Documents/arrow-wasm/tests/generated/error_handling_flow_tests.json",
      "/Users/ods/Documents/arrow-wasm/tests/generated/error_handling_flow_tests.spec.js"
    ]
  }
];

async function runAllTests() {
    console.log('🚀 Running model-derived test suite...');
    
    let totalPassed = 0;
    let totalFailed = 0;
    
    for (const suite of testSuites) {
        console.log(`\n📋 Running tests for ${suite.model_id} (${suite.test_count} tests)...`);
        
        try {
            const testFile = suite.output_files.find(f => f.endsWith('.spec.js'));
            const result = execSync(`npx mocha "${testFile}"`, { 
                cwd: process.cwd(),
                encoding: 'utf8' 
            });
            
            console.log(`✅ ${suite.model_id}: All tests passed`);
            totalPassed += suite.test_count;
            
        } catch (error) {
            console.error(`❌ ${suite.model_id}: Tests failed`);
            console.error(error.stdout || error.message);
            totalFailed += suite.test_count;
        }
    }
    
    console.log(`\n📊 Test Summary:`);
    console.log(`  ✅ Passed: ${totalPassed}`);
    console.log(`  ❌ Failed: ${totalFailed}`);
    console.log(`  📈 Success Rate: ${(totalPassed / (totalPassed + totalFailed) * 100).toFixed(1)}%`);
    
    return totalFailed === 0;
}

if (import.meta.url === `file://${process.argv[1]}`) {
    runAllTests().then(success => {
        process.exit(success ? 0 : 1);
    });
}

export { runAllTests, testSuites };
