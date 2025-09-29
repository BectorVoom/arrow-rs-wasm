/**
 * Debug WASM Loading
 * Test WASM module loading in browser to identify issues
 */

import { chromium } from 'playwright';

async function debugWasmLoading() {
    console.log('🔍 Debugging WASM loading...');
    
    const browser = await chromium.launch({ headless: false }); // Run in headed mode to see what happens
    const context = await browser.newContext();
    const page = await context.newPage();
    
    // Capture all console messages
    page.on('console', msg => {
        console.log(`[BROWSER ${msg.type().toUpperCase()}]:`, msg.text());
    });
    
    // Capture all errors
    page.on('pageerror', error => {
        console.error(`[BROWSER ERROR]:`, error.message);
        console.error(`[BROWSER STACK]:`, error.stack);
    });
    
    // Capture network failures
    page.on('response', response => {
        if (!response.ok()) {
            console.error(`[NETWORK ERROR]: ${response.status()} ${response.url()}`);
        }
    });
    
    try {
        console.log('📱 Navigating to test runner...');
        await page.goto('http://localhost:8080/test_runner_wasm.html');
        
        console.log('⏳ Waiting for WASM to load (or timeout)...');
        
        // Wait for either success or timeout
        const result = await Promise.race([
            page.waitForFunction(() => window.wasmLoaded === true, { timeout: 30000 })
                .then(() => ({ success: true, wasmLoaded: true })),
            page.waitForTimeout(31000).then(() => ({ success: false, timeout: true }))
        ]);
        
        if (result.success) {
            console.log('✅ WASM loaded successfully!');
            
            // Test basic functionality
            console.log('🧪 Testing basic WASM functionality...');
            const testResults = await page.evaluate(async () => {
                return await window.runBasicTests();
            });
            
            console.log('📊 Test results:', JSON.stringify(testResults, null, 2));
            
        } else {
            console.error('❌ WASM loading timed out');
            
            // Check what's available on the window
            const windowState = await page.evaluate(() => ({
                wasmLoaded: window.wasmLoaded,
                wasmModule: !!window.wasmModule,
                hasRunBasicTests: typeof window.runBasicTests === 'function',
                hasRunMBDTestSuite: typeof window.runMBDTestSuite === 'function',
                currentUrl: location.href,
                errors: window.lastError || 'No error captured'
            }));
            
            console.log('🔍 Window state:', JSON.stringify(windowState, null, 2));
        }
        
    } catch (error) {
        console.error('💥 Debug test failed:', error.message);
    } finally {
        console.log('🏁 Closing browser...');
        await browser.close();
    }
}

// Run the debug test
debugWasmLoading().catch(console.error);