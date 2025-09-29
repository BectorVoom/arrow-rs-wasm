/**
 * Automated test for the geometry plugin functionality
 * Tests the plugin registration, WKB data creation, and geometry field validation
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

async function testGeometryPlugin() {
    console.log('🚀 Starting geometry plugin automated test...');
    
    const browser = await chromium.launch();
    const page = await browser.newPage();
    
    // Collect console logs
    const logs = [];
    page.on('console', msg => {
        const text = msg.text();
        console.log(`[Browser] ${text}`);
        logs.push(text);
    });
    
    // Collect errors
    const errors = [];
    page.on('pageerror', error => {
        console.error(`[Browser Error] ${error.message}`);
        errors.push(error.message);
    });
    
    try {
        // Navigate to the geometry plugin test page
        console.log('📄 Loading geometry plugin test page...');
        await page.goto('http://127.0.0.1:8081/geometry_plugin_test.html');
        
        // Wait for the test to complete
        console.log('⏳ Waiting for tests to complete...');
        await page.waitForTimeout(5000);
        
        // Check for test results
        const testResults = await page.evaluate(() => {
            const results = document.getElementById('results');
            const progress = document.getElementById('progress');
            
            return {
                resultsHTML: results ? results.innerHTML : 'No results found',
                progressHTML: progress ? progress.innerHTML : 'No progress found',
                hasErrors: document.querySelectorAll('.error').length > 0,
                hasPasses: document.querySelectorAll('.success').length > 0,
                testCount: document.querySelectorAll('.test-result').length
            };
        });
        
        console.log('\n=== Test Results Summary ===');
        console.log(`Tests run: ${testResults.testCount}`);
        console.log(`Has passes: ${testResults.hasPasses}`);
        console.log(`Has errors: ${testResults.hasErrors}`);
        console.log(`Progress: ${testResults.progressHTML.replace(/<[^>]*>/g, '')}`);
        
        // Check if geometry plugin functions are available
        const pluginFunctions = await page.evaluate(() => {
            // Check if the WASM module exports the expected geometry functions
            return typeof window.register_geometry_plugin !== 'undefined';
        });
        
        console.log(`Geometry plugin functions available: ${pluginFunctions}`);
        
        // Take a screenshot for documentation
        await page.screenshot({ 
            path: path.join(__dirname, 'reports', 'geometry_plugin_test_screenshot.png'),
            fullPage: true 
        });
        
        // Generate test report
        const report = {
            timestamp: new Date().toISOString(),
            success: !testResults.hasErrors && testResults.hasPasses,
            testCount: testResults.testCount,
            logs: logs,
            errors: errors,
            pluginFunctionsAvailable: pluginFunctions,
            summary: {
                geometryPluginRegistered: logs.some(log => log.includes('Geometry plugin registered successfully')),
                wkbDataCreated: logs.some(log => log.includes('Created WKB points')),
                geometryFieldCreated: logs.some(log => log.includes('Geometry field created')),
                pluginValidated: logs.some(log => log.includes('Geometry plugin validation passed')),
                demonstrationCompleted: logs.some(log => log.includes('Geometry plugin demonstration completed'))
            }
        };
        
        // Save the report
        const reportPath = path.join(__dirname, 'reports', 'geometry_plugin_test_report.json');
        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
        
        console.log('\n=== Geometry Plugin Test Summary ===');
        console.log(`✅ Plugin Registration: ${report.summary.geometryPluginRegistered}`);
        console.log(`✅ WKB Data Creation: ${report.summary.wkbDataCreated}`);
        console.log(`✅ Geometry Field Creation: ${report.summary.geometryFieldCreated}`);
        console.log(`✅ Plugin Validation: ${report.summary.pluginValidated}`);
        console.log(`✅ Demonstration Complete: ${report.summary.demonstrationCompleted}`);
        
        const allTestsPassed = Object.values(report.summary).every(test => test === true);
        
        if (allTestsPassed && !testResults.hasErrors) {
            console.log('\n🎉 All geometry plugin tests PASSED!');
            console.log('The geometry plugin is working correctly and ready for use.');
        } else {
            console.log('\n❌ Some geometry plugin tests FAILED!');
            console.log('Check the test report for details.');
        }
        
        console.log(`\n📊 Test report saved: ${reportPath}`);
        
        return { success: allTestsPassed && !testResults.hasErrors, report };
        
    } catch (error) {
        console.error('❌ Test execution failed:', error);
        return { success: false, error: error.message };
    } finally {
        await browser.close();
    }
}

// Run the test
if (require.main === module) {
    testGeometryPlugin()
        .then(result => {
            console.log(`\n🏁 Test completed with result: ${result.success ? 'SUCCESS' : 'FAILURE'}`);
            process.exit(result.success ? 0 : 1);
        })
        .catch(error => {
            console.error('❌ Test runner failed:', error);
            process.exit(1);
        });
}

module.exports = { testGeometryPlugin };