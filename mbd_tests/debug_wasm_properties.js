/**
 * Debug WASM module properties
 * Check what's actually available on the imported WASM module
 */

import { chromium } from 'playwright';

async function debugWASMProperties() {
    console.log('🔍 Debugging WASM module properties...');
    
    const browser = await chromium.launch({
        headless: false,
        args: ['--enable-experimental-web-platform-features']
    });
    
    try {
        const context = await browser.newContext({
            ignoreHTTPSErrors: true
        });
        
        const page = await context.newPage();
        
        // Collect console logs
        page.on('console', msg => {
            console.log(`Browser: [${msg.type()}] ${msg.text()}`);
        });
        
        page.on('pageerror', error => {
            console.error(`Browser Error: ${error.message}`);
        });
        
        console.log('📖 Navigating to test server root...');
        await page.goto('http://localhost:8080/');
        
        console.log('🔧 Testing WASM module import directly...');
        
        // Test WASM import directly in browser console
        const wasmAnalysis = await page.evaluate(async () => {
            try {
                // Import the WASM module
                const wasmModule = await import('/pkg/arrow_wasm.js');
                
                const beforeInit = {
                    type: typeof wasmModule,
                    keys: Object.keys(wasmModule),
                    hasDefault: 'default' in wasmModule,
                    hasInit: 'init' in wasmModule,
                    defaultType: typeof wasmModule.default,
                    initType: typeof wasmModule.init
                };
                
                // Try to initialize
                let afterInit = null;
                let initError = null;
                
                try {
                    await wasmModule.default();
                    
                    afterInit = {
                        type: typeof wasmModule,
                        keys: Object.keys(wasmModule),
                        hasDefault: 'default' in wasmModule,
                        hasInit: 'init' in wasmModule,
                        defaultType: typeof wasmModule.default,
                        initType: typeof wasmModule.init
                    };
                } catch (error) {
                    initError = error.message;
                }
                
                return {
                    beforeInit,
                    afterInit,
                    initError
                };
                
            } catch (error) {
                return {
                    error: error.message,
                    stack: error.stack
                };
            }
        });
        
        console.log('\n🎯 WASM Module Analysis:');
        console.log(JSON.stringify(wasmAnalysis, null, 2));
        
        return wasmAnalysis;
        
    } finally {
        await browser.close();
    }
}

// Run diagnostic
if (import.meta.url === `file://${process.argv[1]}`) {
    try {
        const result = await debugWASMProperties();
        console.log('\n🏁 Property diagnostic completed');
        process.exit(0);
    } catch (error) {
        console.error('❌ Property diagnostic failed:', error);
        process.exit(1);
    }
}

export default debugWASMProperties;