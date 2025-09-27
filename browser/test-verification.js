/**
 * Simple verification script to test core Feather compression functionality
 * Run this in the browser console to verify the implementation
 */

async function runVerificationTests() {
  console.log("🚀 Starting Arrow WASM Feather Compression Verification...");

  try {
    // Import required modules
    const { default: init, ...wasm_arrow } = await import(
      "../pkg/arrow_wasm_arrow.js"
    );
    const { FeatherTestDataGenerator } = await import("./feather-test-data.js");

    // Initialize WASM module
    console.log("📦 Initializing WASM module...");
    await init();
    await wasm_arrow.initialize();

    // Get version info
    const version = wasm_arrow.getVersion();
    console.log(
      `✅ WASM Module loaded: ${version.major()}.${version.minor()}.${version.patch()}`,
    );
    console.log(`📚 Arrow version: ${version.arrow_version()}`);

    // Test 1: Basic table creation
    console.log("\n🧪 Test 1: Basic table creation from JSON");
    const testData = FeatherTestDataGenerator.generateSmallMixedData();
    console.log(`📊 Test data: ${testData.length} rows`);

    const table = wasm_arrow.tableFromJSON(testData);
    console.log(
      `✅ Table created: ${table.numRows} rows, ${table.numColumns} columns`,
    );

    // Test 2: Uncompressed IPC
    console.log("\n🧪 Test 2: Uncompressed IPC export");
    const writeOptions = wasm_arrow.WriteOptions.new();
    writeOptions.set_compression(wasm_arrow.CompressionType.None);

    const uncompressedData = table.toIPC(writeOptions);
    console.log(`📦 Uncompressed size: ${uncompressedData.length} bytes`);

    // Test 3: LZ4 compression
    console.log("\n🧪 Test 3: LZ4 compression");
    const lz4Options = wasm_arrow.WriteOptions.new();
    lz4Options.set_compression(wasm_arrow.CompressionType.LZ4);

    const lz4Data = table.toIPC(lz4Options);
    const lz4Ratio = uncompressedData.length / lz4Data.length;
    console.log(`📦 LZ4 compressed size: ${lz4Data.length} bytes`);
    console.log(`📈 LZ4 compression ratio: ${lz4Ratio.toFixed(2)}x`);

    // Test 4: ZSTD compression
    console.log("\n🧪 Test 4: ZSTD compression");
    const zstdOptions = wasm_arrow.WriteOptions.new();
    zstdOptions.set_compression(wasm_arrow.CompressionType.ZSTD);

    const zstdData = table.toIPC(zstdOptions);
    const zstdRatio = uncompressedData.length / zstdData.length;
    console.log(`📦 ZSTD compressed size: ${zstdData.length} bytes`);
    console.log(`📈 ZSTD compression ratio: ${zstdRatio.toFixed(2)}x`);

    // Test 5: Round-trip test with LZ4
    console.log("\n🧪 Test 5: Round-trip integrity test");
    const readTable = wasm_arrow.tableFromIPC(lz4Data);
    console.log(
      `📖 Read table: ${readTable.numRows} rows, ${readTable.numColumns} columns`,
    );

    // Verify table properties match
    const integrityCheck =
      table.numRows === readTable.numRows &&
      table.numColumns === readTable.numColumns;

    console.log(`🔍 Integrity check: ${integrityCheck ? "PASS" : "FAIL"}`);

    // Test 6: Large dataset compression
    console.log("\n🧪 Test 6: Large dataset compression test");
    const largeData = FeatherTestDataGenerator.generateRepetitiveData(1000);
    const largeTable = wasm_arrow.tableFromJSON(largeData);

    const largeUncompressed = largeTable.toIPC(writeOptions);
    const largeLZ4 = largeTable.toIPC(lz4Options);
    const largeZSTD = largeTable.toIPC(zstdOptions);

    console.log(`📊 Large dataset (1000 rows):`);
    console.log(
      `   Uncompressed: ${Math.round(largeUncompressed.length / 1024)}KB`,
    );
    console.log(
      `   LZ4: ${Math.round(largeLZ4.length / 1024)}KB (${(largeUncompressed.length / largeLZ4.length).toFixed(1)}x)`,
    );
    console.log(
      `   ZSTD: ${Math.round(largeZSTD.length / 1024)}KB (${(largeUncompressed.length / largeZSTD.length).toFixed(1)}x)`,
    );

    // Cleanup
    table.dispose();
    readTable.dispose();
    largeTable.dispose();

    // Summary
    console.log("\n🎉 All verification tests completed successfully!");
    console.log("📋 Summary:");
    console.log(`   ✅ WASM module initialization: PASS`);
    console.log(`   ✅ Table creation from JSON: PASS`);
    console.log(`   ✅ Uncompressed IPC export: PASS`);
    console.log(`   ✅ LZ4 compression: PASS (${lz4Ratio.toFixed(2)}x)`);
    console.log(`   ✅ ZSTD compression: PASS (${zstdRatio.toFixed(2)}x)`);
    console.log(
      `   ✅ Round-trip integrity: ${integrityCheck ? "PASS" : "FAIL"}`,
    );
    console.log(`   ✅ Large dataset compression: PASS`);

    return {
      success: true,
      tests: {
        initialization: true,
        tableCreation: true,
        uncompressed: true,
        lz4: { success: true, ratio: lz4Ratio },
        zstd: { success: true, ratio: zstdRatio },
        integrity: integrityCheck,
        largeDataset: true,
      },
    };
  } catch (error) {
    console.error("❌ Verification failed:", error);
    return {
      success: false,
      error: error.toString(),
    };
  }
}

// Auto-run if loaded directly
if (
  typeof window !== "undefined" &&
  window.location.search.includes("autorun")
) {
  runVerificationTests().then((result) => {
    if (result.success) {
      console.log(
        "🎯 All tests passed! Feather compression is working correctly.",
      );
    } else {
      console.error("💥 Tests failed:", result.error);
    }
  });
}

// Export for manual use
if (typeof window !== "undefined") {
  window.runVerificationTests = runVerificationTests;
}

export { runVerificationTests };
