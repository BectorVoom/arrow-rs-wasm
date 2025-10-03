/**
 * Cross-Browser Test Automation Runner
 * 
 * Orchestrates model-based tests across multiple browser environments
 * using Playwright for comprehensive cross-browser compatibility testing
 */

import { chromium, firefox, webkit } from 'playwright';
import express from 'express';
import path from 'path';
import fs from 'fs/promises';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PROJECT_ROOT = path.resolve(__dirname, '..');

class CrossBrowserTestRunner {
    constructor() {
        this.results = new Map();
        this.browsers = ['chromium', 'firefox', 'webkit'];
        this.testServer = null;
        this.serverPort = 8080;
        this.testResults = {
            summary: {
                totalTests: 0,
                passedTests: 0,
                failedTests: 0,
                skippedTests: 0,
                browsers: {},
                duration: 0
            },
            detailed: {}
        };
    }

    /**
     * Start test server for serving test files
     */
    async startTestServer() {
        const app = express();
        
        // Enable CORS for all origins
        app.use((req, res, next) => {
            res.header('Access-Control-Allow-Origin', '*');
            res.header('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
            res.header('Access-Control-Allow-Headers', 'Origin, X-Requested-With, Content-Type, Accept');
            if (req.method === 'OPTIONS') {
                res.sendStatus(200);
            } else {
                next();
            }
        });

        // Serve static files from project root
        app.use(express.static(PROJECT_ROOT));
        
        // Serve WASM files with correct MIME type
        app.use('/pkg', express.static(path.join(PROJECT_ROOT, 'pkg'), {
            setHeaders: (res, path) => {
                if (path.endsWith('.wasm')) {
                    res.setHeader('Content-Type', 'application/wasm');
                }
            }
        }));

        return new Promise((resolve, reject) => {
            this.testServer = app.listen(this.serverPort, () => {
                console.log(`Test server started on http://localhost:${this.serverPort}`);
                resolve();
            }).on('error', reject);
        });
    }

    /**
     * Stop test server
     */
    async stopTestServer() {
        if (this.testServer) {
            this.testServer.close();
            console.log('Test server stopped');
        }
    }

    /**
     * Run tests across all configured browsers
     */
    async runAllBrowsers() {
        const startTime = Date.now();
        
        await this.startTestServer();
        
        try {
            for (const browserName of this.browsers) {
                console.log(`\n=== Running tests in ${browserName} ===`);
                const browserResults = await this.runBrowserTests(browserName);
                this.results.set(browserName, browserResults);
                this.testResults.detailed[browserName] = browserResults;
                this.testResults.summary.browsers[browserName] = {
                    passed: browserResults.passed,
                    failed: browserResults.failed,
                    skipped: browserResults.skipped,
                    total: browserResults.total,
                    duration: browserResults.duration
                };
            }
        } finally {
            await this.stopTestServer();
        }

        this.testResults.summary.duration = Date.now() - startTime;
        this.calculateOverallSummary();
        
        return this.testResults;
    }

    /**
     * Run tests in a specific browser
     */
    async runBrowserTests(browserName) {
        const browser = await this.launchBrowser(browserName);
        const startTime = Date.now();
        
        try {
            const context = await browser.newContext({
                // Enable permissions for local file access
                permissions: ['clipboard-read', 'clipboard-write']
            });
            
            const page = await context.newPage();
            
            // Enable console logging from browser
            page.on('console', msg => {
                if (msg.type() === 'error') {
                    console.error(`[${browserName}] Console Error:`, msg.text());
                }
            });
            
            // Navigate to model-based test runner
            const testUrl = `http://localhost:${this.serverPort}/model_based_test_runner.html`;
            await page.goto(testUrl);
            
            // Wait for WASM to load
            await page.waitForFunction(() => window.wasmModule !== undefined, { timeout: 30000 });
            
            // Run the model-based tests
            const results = await this.executeModelBasedTests(page, browserName);
            
            await context.close();
            return {
                ...results,
                duration: Date.now() - startTime,
                browser: browserName
            };
            
        } catch (error) {
            console.error(`Error running tests in ${browserName}:`, error);
            return {
                passed: 0,
                failed: 1,
                skipped: 0,
                total: 1,
                duration: Date.now() - startTime,
                browser: browserName,
                error: error.message,
                tests: [{
                    name: 'Browser Launch',
                    status: 'failed',
                    error: error.message
                }]
            };
        } finally {
            await browser.close();
        }
    }

    /**
     * Execute model-based tests on a page
     */
    async executeModelBasedTests(page, browserName) {
        console.log(`Executing model-based tests in ${browserName}...`);
        
        // Start the test suite
        await page.evaluate(() => {
            if (window.startModelBasedTests) {
                window.startModelBasedTests();
            }
        });
        
        // Wait for tests to complete (max 5 minutes)
        await page.waitForFunction(() => {
            return window.testResults && window.testResults.completed === true;
        }, { timeout: 300000 });
        
        // Extract test results
        const results = await page.evaluate(() => {
            return window.testResults;
        });
        
        // Extract coverage metrics
        const coverage = await page.evaluate(() => {
            return window.coverageMetrics || {};
        });
        
        // Extract performance metrics
        const performance = await page.evaluate(() => {
            return window.performanceMetrics || {};
        });
        
        return {
            passed: results.passed || 0,
            failed: results.failed || 0,
            skipped: results.skipped || 0,
            total: results.total || 0,
            tests: results.tests || [],
            coverage: coverage,
            performance: performance,
            modelCoverage: results.modelCoverage || {},
            traceability: results.traceability || {}
        };
    }

    /**
     * Launch browser based on name
     */
    async launchBrowser(browserName) {
        const options = {
            headless: process.env.HEADLESS !== 'false',
            args: ['--disable-web-security', '--allow-running-insecure-content']
        };
        
        switch (browserName) {
            case 'chromium':
                return await chromium.launch(options);
            case 'firefox':
                return await firefox.launch(options);
            case 'webkit':
                return await webkit.launch(options);
            default:
                throw new Error(`Unknown browser: ${browserName}`);
        }
    }

    /**
     * Calculate overall summary statistics
     */
    calculateOverallSummary() {
        let totalPassed = 0;
        let totalFailed = 0;
        let totalSkipped = 0;
        let totalTests = 0;
        
        for (const [browser, results] of this.results) {
            totalPassed += results.passed || 0;
            totalFailed += results.failed || 0;
            totalSkipped += results.skipped || 0;
            totalTests += results.total || 0;
        }
        
        this.testResults.summary.totalTests = totalTests;
        this.testResults.summary.passedTests = totalPassed;
        this.testResults.summary.failedTests = totalFailed;
        this.testResults.summary.skippedTests = totalSkipped;
    }

    /**
     * Generate comprehensive test report
     */
    async generateReport() {
        const reportPath = path.join(PROJECT_ROOT, 'reports', 'cross-browser-test-report.json');
        await fs.mkdir(path.dirname(reportPath), { recursive: true });
        await fs.writeFile(reportPath, JSON.stringify(this.testResults, null, 2));
        
        // Generate markdown report
        const markdownReport = this.generateMarkdownReport();
        const markdownPath = path.join(PROJECT_ROOT, 'reports', 'cross-browser-test-report.md');
        await fs.writeFile(markdownPath, markdownReport);
        
        console.log(`\nReports generated:`);
        console.log(`- JSON: ${reportPath}`);
        console.log(`- Markdown: ${markdownPath}`);
        
        return {
            jsonPath: reportPath,
            markdownPath: markdownPath,
            results: this.testResults
        };
    }

    /**
     * Generate markdown test report
     */
    generateMarkdownReport() {
        const { summary, detailed } = this.testResults;
        const timestamp = new Date().toISOString();
        
        let report = `# Cross-Browser Test Report\n\n`;
        report += `**Generated:** ${timestamp}\n`;
        report += `**Total Duration:** ${(summary.duration / 1000).toFixed(2)}s\n\n`;
        
        // Overall Summary
        report += `## Overall Summary\n\n`;
        report += `| Metric | Count | Percentage |\n`;
        report += `|--------|-------|------------|\n`;
        report += `| Total Tests | ${summary.totalTests} | 100% |\n`;
        report += `| Passed | ${summary.passedTests} | ${((summary.passedTests / summary.totalTests) * 100).toFixed(1)}% |\n`;
        report += `| Failed | ${summary.failedTests} | ${((summary.failedTests / summary.totalTests) * 100).toFixed(1)}% |\n`;
        report += `| Skipped | ${summary.skippedTests} | ${((summary.skippedTests / summary.totalTests) * 100).toFixed(1)}% |\n\n`;
        
        // Browser-specific results
        report += `## Browser Results\n\n`;
        for (const [browser, browserSummary] of Object.entries(summary.browsers)) {
            report += `### ${browser.charAt(0).toUpperCase() + browser.slice(1)}\n\n`;
            report += `- **Total:** ${browserSummary.total}\n`;
            report += `- **Passed:** ${browserSummary.passed}\n`;
            report += `- **Failed:** ${browserSummary.failed}\n`;
            report += `- **Duration:** ${(browserSummary.duration / 1000).toFixed(2)}s\n`;
            
            const browserDetails = detailed[browser];
            if (browserDetails?.coverage) {
                report += `- **Model Coverage:** ${(browserDetails.coverage.percentage || 0).toFixed(1)}%\n`;
            }
            report += `\n`;
        }
        
        // Compatibility Matrix
        report += `## Browser Compatibility Matrix\n\n`;
        report += `| Browser | Status | Tests Passed | Model Coverage | Performance |\n`;
        report += `|---------|--------|--------------|----------------|--------------|\n`;
        
        for (const [browser, browserSummary] of Object.entries(summary.browsers)) {
            const status = browserSummary.failed === 0 ? '✅ Pass' : '❌ Fail';
            const passRate = ((browserSummary.passed / browserSummary.total) * 100).toFixed(1);
            const coverage = detailed[browser]?.coverage?.percentage?.toFixed(1) || 'N/A';
            const avgPerf = detailed[browser]?.performance?.averageResponseTime?.toFixed(0) || 'N/A';
            
            report += `| ${browser} | ${status} | ${passRate}% | ${coverage}% | ${avgPerf}ms |\n`;
        }
        
        return report;
    }

    /**
     * Print summary to console
     */
    printSummary() {
        const { summary } = this.testResults;
        
        console.log('\n' + '='.repeat(60));
        console.log('CROSS-BROWSER TEST SUMMARY');
        console.log('='.repeat(60));
        console.log(`Total Tests: ${summary.totalTests}`);
        console.log(`Passed: ${summary.passedTests} (${((summary.passedTests / summary.totalTests) * 100).toFixed(1)}%)`);
        console.log(`Failed: ${summary.failedTests} (${((summary.failedTests / summary.totalTests) * 100).toFixed(1)}%)`);
        console.log(`Duration: ${(summary.duration / 1000).toFixed(2)}s`);
        
        console.log('\nBrowser Results:');
        for (const [browser, results] of Object.entries(summary.browsers)) {
            const status = results.failed === 0 ? '✅' : '❌';
            console.log(`  ${status} ${browser}: ${results.passed}/${results.total} passed`);
        }
        console.log('='.repeat(60));
    }
}

// CLI execution
if (import.meta.url === `file://${process.argv[1]}`) {
    const runner = new CrossBrowserTestRunner();
    
    try {
        console.log('Starting cross-browser test execution...');
        const results = await runner.runAllBrowsers();
        await runner.generateReport();
        runner.printSummary();
        
        // Exit with appropriate code
        process.exit(results.summary.failedTests === 0 ? 0 : 1);
    } catch (error) {
        console.error('Cross-browser test execution failed:', error);
        process.exit(1);
    }
}

export { CrossBrowserTestRunner };