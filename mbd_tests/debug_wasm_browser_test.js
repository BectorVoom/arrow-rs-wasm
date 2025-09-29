/**
 * Debug WASM Browser Test
 * Simple diagnostic script to identify WASM loading issues
 */

import { chromium } from 'playwright';

async function debugWASMLoading() {
    console.log('🔍 Starting WASM loading diagnostic...');
    
    const browser = await chromium.launch({
        headless: false, // Show browser for debugging
        args: ['--enable-experimental-web-platform-features']
    });
    
    try {
        const context = await browser.newContext({
            ignoreHTTPSErrors: true
        });
        
        const page = await context.newPage();
        
        // Collect console logs
        const logs = [];
        page.on('console', msg => {
            const message = `[${msg.type()}] ${msg.text()}`;
            console.log(`Browser: ${message}`);
            logs.push(message);
        });
        
        // Collect errors
        page.on('pageerror', error => {
            console.error(`Browser Error: ${error.message}`);
            logs.push(`ERROR: ${error.message}`);
        });
        
        console.log('📖 Navigating to test page...');
        await page.goto('http://localhost:8080/test_runner_wasm.html');
        
        console.log('⏳ Waiting for DOM load...');
        await page.waitForLoadState('domcontentloaded');
        
        console.log('🎯 Checking WASM loading status...');
        
        // Wait for WASM loading with detailed monitoring
        let wasmLoaded = false;
        let attemptCount = 0;
        const maxAttempts = 60; // 60 seconds
        
        while (!wasmLoaded && attemptCount < maxAttempts) {
            try {
                // Check if wasmLoaded flag is set
                wasmLoaded = await page.evaluate(() => window.wasmLoaded === true);
                
                if (!wasmLoaded) {
                    // Check loading status
                    const loadingStatus = await page.evaluate(() => {
                        return {
                            wasmLoaded: window.wasmLoaded,
                            wasmModule: typeof window.wasmModule,
                            statusText: document.getElementById('wasm-status')?.textContent
                        };
                    });
                    
                    console.log(`Attempt ${attemptCount + 1}/${maxAttempts}: ${JSON.stringify(loadingStatus)}`);
                    
                    // Wait 1 second before next check
                    await page.waitForTimeout(1000);
                    attemptCount++;
                } else {
                    console.log('✅ WASM loaded successfully!');
                    break;
                }
                
            } catch (error) {
                console.error(`Error checking WASM status: ${error.message}`);
                break;
            }
        }
        
        if (!wasmLoaded) {
            console.error('❌ WASM loading timed out');
            
            // Get final page state for debugging
            const finalState = await page.evaluate(() => {
                return {
                    wasmLoaded: window.wasmLoaded,
                    wasmModule: typeof window.wasmModule,
                    statusElement: document.getElementById('wasm-status')?.outerHTML,
                    consoleErrors: window.console?.errors || []
                };
            });
            
            console.log('Final page state:', JSON.stringify(finalState, null, 2));
        } else {
            console.log('🧪 Testing basic WASM functionality...');
            
            // Test basic WASM functions
            const testResult = await page.evaluate(async () => {
                try {
                    if (window.runBasicTests) {
                        return await window.runBasicTests();
                    } else {
                        return { error: 'runBasicTests function not available' };
                    }
                } catch (error) {
                    return { error: error.message };
                }
            });
            
            console.log('🎯 Test results:', JSON.stringify(testResult, null, 2));
        }
        
        // Take screenshot for debugging
        await page.screenshot({ 
            path: 'debug_wasm_loading.png',
            fullPage: true 
        });
        
        console.log('📸 Screenshot saved as debug_wasm_loading.png');
        
        return {
            success: wasmLoaded,
            logs: logs,
            attempts: attemptCount
        };
        
    } finally {
        await browser.close();
    }
}

// Run diagnostic
if (import.meta.url === `file://${process.argv[1]}`) {
    try {
        const result = await debugWASMLoading();
        console.log('\n🏁 Diagnostic completed:', result.success ? 'SUCCESS' : 'FAILED');
        process.exit(result.success ? 0 : 1);
    } catch (error) {
        console.error('❌ Diagnostic failed:', error);
        process.exit(1);
    }
}

export default debugWASMLoading;