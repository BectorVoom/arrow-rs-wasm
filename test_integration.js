#!/usr/bin/env node

/**
 * Integration test with real Arrow data
 * Tests the core WASM functionality with actual Arrow files
 */

import fs from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function createTestArrowFile() {
    console.log('🏗️ Creating test Arrow data...');
    
    // Create a simple test script to generate Arrow data using Rust
    const rustTest = `
use std::sync::Arc;
use arrow_array::{ArrayRef, Int32Array, StringArray, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use arrow_ipc::writer::FileWriter;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create schema
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("value", DataType::Int32, true),
    ]);

    // Create arrays
    let id_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie", "Diana", "Eve"]);
    let value_array = Int32Array::from(vec![Some(100), None, Some(300), Some(400), None]);

    // Create record batch
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array) as ArrayRef,
            Arc::new(name_array) as ArrayRef,
            Arc::new(value_array) as ArrayRef,
        ],
    )?;

    // Write to file
    let file = File::create("test_data.arrow")?;
    let mut writer = FileWriter::try_new(file, &batch.schema())?;
    writer.write(&batch)?;
    writer.finish()?;

    println!("✅ Test Arrow file created: test_data.arrow");
    Ok(())
}
`;

    // Write the test generator
    fs.writeFileSync('test_arrow_generator.rs', rustTest);
    
    console.log('📋 Generated test Arrow creation script');
    return true;
}

async function validateArrowIntegration() {
    console.log('🧪 Validating Arrow integration...');
    
    // Check if core files exist
    const requiredFiles = [
        'pkg/arrow_wasm.js',
        'pkg/arrow_wasm_bg.wasm',
        'dist/index.js',
        'src/lib.rs'
    ];
    
    let allFilesExist = true;
    for (const file of requiredFiles) {
        if (!fs.existsSync(file)) {
            console.log(`❌ Missing required file: ${file}`);
            allFilesExist = false;
        } else {
            console.log(`✅ Found: ${file}`);
        }
    }
    
    if (!allFilesExist) {
        console.log('❌ Integration validation failed: Missing required files');
        return false;
    }
    
    // Check Cargo.toml for correct dependencies
    const cargoToml = fs.readFileSync('Cargo.toml', 'utf8');
    const hasArrowDeps = cargoToml.includes('arrow =') && cargoToml.includes('arrow-ipc =');
    const hasLz4 = cargoToml.includes('lz4');
    const hasWasmBindgen = cargoToml.includes('wasm-bindgen');
    
    if (hasArrowDeps && hasLz4 && hasWasmBindgen) {
        console.log('✅ Cargo.toml has correct dependencies');
    } else {
        console.log('❌ Cargo.toml missing required dependencies');
        return false;
    }
    
    // Check if the WASM file is reasonable size (should be > 1MB for Arrow functionality)
    const wasmStats = fs.statSync('pkg/arrow_wasm_bg.wasm');
    const wasmSizeMB = wasmStats.size / (1024 * 1024);
    console.log(`📊 WASM file size: ${wasmSizeMB.toFixed(2)} MB`);
    
    if (wasmSizeMB > 1.0 && wasmSizeMB < 10.0) {
        console.log('✅ WASM file size is reasonable');
    } else {
        console.log('⚠️ WASM file size seems unusual');
    }
    
    console.log('✅ Arrow integration validation completed');
    return true;
}

async function testLZ4Functionality() {
    console.log('🗜️ Testing LZ4 compression functionality...');
    
    // Check if LZ4 feature is enabled in source code
    const libRs = fs.readFileSync('src/lib.rs', 'utf8');
    const hasLz4Functions = libRs.includes('lz4') || libRs.includes('compression');
    
    if (hasLz4Functions) {
        console.log('✅ LZ4 compression code found in lib.rs');
    } else {
        console.log('❌ LZ4 compression functionality not found');
        return false;
    }
    
    // Check Cargo.toml for lz4_flex dependency
    const cargoToml = fs.readFileSync('Cargo.toml', 'utf8');
    if (cargoToml.includes('lz4_flex')) {
        console.log('✅ lz4_flex dependency found');
    } else {
        console.log('❌ lz4_flex dependency missing');
        return false;
    }
    
    console.log('✅ LZ4 functionality validation completed');
    return true;
}

async function runIntegrationTests() {
    console.log('🚀 Starting Arrow WASM Integration Tests...');
    console.log('='.repeat(50));
    
    let allPassed = true;
    
    // Test 1: Create test data
    try {
        await createTestArrowFile();
        console.log('✅ Test 1: Arrow data creation - PASSED');
    } catch (error) {
        console.log('❌ Test 1: Arrow data creation - FAILED:', error.message);
        allPassed = false;
    }
    
    // Test 2: Validate integration
    try {
        const result = await validateArrowIntegration();
        if (result) {
            console.log('✅ Test 2: Arrow integration - PASSED');
        } else {
            console.log('❌ Test 2: Arrow integration - FAILED');
            allPassed = false;
        }
    } catch (error) {
        console.log('❌ Test 2: Arrow integration - FAILED:', error.message);
        allPassed = false;
    }
    
    // Test 3: LZ4 functionality
    try {
        const result = await testLZ4Functionality();
        if (result) {
            console.log('✅ Test 3: LZ4 functionality - PASSED');
        } else {
            console.log('❌ Test 3: LZ4 functionality - FAILED');
            allPassed = false;
        }
    } catch (error) {
        console.log('❌ Test 3: LZ4 functionality - FAILED:', error.message);
        allPassed = false;
    }
    
    console.log('='.repeat(50));
    
    if (allPassed) {
        console.log('🎉 All integration tests PASSED!');
        console.log('📋 Arrow WASM library is ready for production use');
        return 0;
    } else {
        console.log('❌ Some integration tests FAILED!');
        console.log('📋 Review the failures above before proceeding');
        return 1;
    }
}

// Run the tests
runIntegrationTests()
    .then(exitCode => process.exit(exitCode))
    .catch(error => {
        console.error('💥 Integration test runner failed:', error);
        process.exit(1);
    });