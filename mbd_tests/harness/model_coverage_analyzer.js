#!/usr/bin/env node

/**
 * Model Coverage Analysis Engine
 * 
 * Analyzes model coverage to ensure ≥90% of mandatory model transitions
 * are exercised by the test suite. This is a critical component for MBD
 * acceptance criteria validation.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class ModelCoverageAnalyzer {
    constructor() {
        this.modelsDir = path.join(__dirname, '..', 'models');
        this.reportsDir = path.join(__dirname, '..', 'reports');
        this.coverageThreshold = 90; // 90% minimum coverage for mandatory transitions
        this.results = {
            totalModels: 0,
            modelsCovered: 0,
            totalStates: 0,
            statesCovered: 0,
            totalTransitions: 0,
            transitionsCovered: 0,
            mandatoryTransitions: 0,
            mandatoryTransitionsCovered: 0,
            coverageByModel: {},
            gaps: [],
            passed: false
        };
    }

    /**
     * Analyze coverage across all models
     */
    async analyzeCoverage() {
        console.log('🔍 Starting model coverage analysis...');
        
        try {
            // Load all model files
            const models = await this.loadAllModels();
            console.log(`📊 Found ${models.length} model files`);
            
            // Load test execution records
            const testResults = await this.loadTestResults();
            console.log(`📈 Loaded test results for ${Object.keys(testResults).length} model elements`);
            
            // Analyze coverage for each model
            for (const model of models) {
                await this.analyzeModelCoverage(model, testResults);
            }
            
            // Calculate overall coverage metrics
            this.calculateOverallCoverage();
            
            // Check if coverage meets acceptance criteria
            this.validateAcceptanceCriteria();
            
            // Generate coverage report
            await this.generateCoverageReport();
            
            console.log(`✅ Model coverage analysis complete`);
            return this.results;
            
        } catch (error) {
            console.error('❌ Model coverage analysis failed:', error);
            throw error;
        }
    }

    /**
     * Load all model JSON files recursively
     */
    async loadAllModels() {
        const models = [];
        const modelFiles = await this.findModelFiles(this.modelsDir);
        
        for (const filePath of modelFiles) {
            try {
                const content = fs.readFileSync(filePath, 'utf8');
                const model = JSON.parse(content);
                model._filePath = filePath;
                model._fileName = path.basename(filePath, '.json');
                models.push(model);
            } catch (error) {
                console.warn(`⚠️  Failed to load model ${filePath}:`, error.message);
            }
        }
        
        return models;
    }

    /**
     * Recursively find all JSON model files
     */
    async findModelFiles(dir) {
        const files = [];
        const entries = fs.readdirSync(dir, { withFileTypes: true });
        
        for (const entry of entries) {
            const fullPath = path.join(dir, entry.name);
            if (entry.isDirectory()) {
                files.push(...await this.findModelFiles(fullPath));
            } else if (entry.isFile() && entry.name.endsWith('.json')) {
                files.push(fullPath);
            }
        }
        
        return files;
    }

    /**
     * Load test execution results from traceability matrix
     */
    async loadTestResults() {
        const traceabilityPath = path.join(__dirname, '..', 'traceability_matrix.csv');
        const testResults = {};
        
        if (!fs.existsSync(traceabilityPath)) {
            console.warn('⚠️  Traceability matrix not found, using empty test results');
            return testResults;
        }
        
        try {
            const csvContent = fs.readFileSync(traceabilityPath, 'utf8');
            const lines = csvContent.split('\n').slice(1); // Skip header
            
            for (const line of lines) {
                if (line.trim()) {
                    const [reqId, modelId, modelElement, testId, testType, timestamp] = 
                        line.split(',').map(field => field.replace(/"/g, ''));
                    
                    const key = `${modelId}:${modelElement}`;
                    if (!testResults[key]) {
                        testResults[key] = [];
                    }
                    
                    testResults[key].push({
                        testId,
                        testType,
                        timestamp,
                        requirements: reqId.split(';')
                    });
                }
            }
        } catch (error) {
            console.warn('⚠️  Failed to load test results:', error.message);
        }
        
        return testResults;
    }

    /**
     * Analyze coverage for a single model
     */
    async analyzeModelCoverage(model, testResults) {
        const modelId = model.model_id || model._fileName;
        const coverage = {
            modelId,
            modelType: model.model_type,
            totalStates: 0,
            statesCovered: 0,
            totalTransitions: 0,
            transitionsCovered: 0,
            mandatoryTransitions: 0,
            mandatoryTransitionsCovered: 0,
            coveragePercentage: 0,
            mandatoryCoveragePercentage: 0,
            gaps: [],
            requirements: model.requirements || []
        };

        // Analyze state coverage
        if (model.states) {
            coverage.totalStates = model.states.length;
            for (const state of model.states) {
                const key = `${modelId}:${state.id}`;
                if (testResults[key] && testResults[key].length > 0) {
                    coverage.statesCovered++;
                } else {
                    coverage.gaps.push({
                        type: 'state',
                        id: state.id,
                        name: state.name,
                        mandatory: this.isMandatory(state)
                    });
                }
            }
        }

        // Analyze transition coverage
        if (model.transitions) {
            coverage.totalTransitions = model.transitions.length;
            for (const transition of model.transitions) {
                const key = `${modelId}:${transition.id}`;
                const isMandatory = this.isMandatory(transition);
                
                if (isMandatory) {
                    coverage.mandatoryTransitions++;
                }
                
                if (testResults[key] && testResults[key].length > 0) {
                    coverage.transitionsCovered++;
                    if (isMandatory) {
                        coverage.mandatoryTransitionsCovered++;
                    }
                } else {
                    coverage.gaps.push({
                        type: 'transition',
                        id: transition.id,
                        name: transition.trigger || transition.id,
                        from: transition.from,
                        to: transition.to,
                        mandatory: isMandatory,
                        requirements: transition.requirements || []
                    });
                }
            }
        }

        // Calculate percentages
        coverage.coveragePercentage = coverage.totalTransitions > 0 ? 
            (coverage.transitionsCovered / coverage.totalTransitions) * 100 : 100;
        
        coverage.mandatoryCoveragePercentage = coverage.mandatoryTransitions > 0 ? 
            (coverage.mandatoryTransitionsCovered / coverage.mandatoryTransitions) * 100 : 100;

        this.results.coverageByModel[modelId] = coverage;
        console.log(`📊 ${modelId}: ${coverage.mandatoryCoveragePercentage.toFixed(1)}% mandatory coverage`);
    }

    /**
     * Determine if a model element is mandatory
     */
    isMandatory(element) {
        // Consider elements mandatory if they have requirements or are marked as critical
        return (element.requirements && element.requirements.length > 0) ||
               (element.type === 'initial' || element.type === 'final') ||
               (element.guard && element.guard !== 'true') ||
               (element.properties && element.properties.mandatory === true);
    }

    /**
     * Calculate overall coverage metrics
     */
    calculateOverallCoverage() {
        this.results.totalModels = Object.keys(this.results.coverageByModel).length;
        
        for (const coverage of Object.values(this.results.coverageByModel)) {
            this.results.totalStates += coverage.totalStates;
            this.results.statesCovered += coverage.statesCovered;
            this.results.totalTransitions += coverage.totalTransitions;
            this.results.transitionsCovered += coverage.transitionsCovered;
            this.results.mandatoryTransitions += coverage.mandatoryTransitions;
            this.results.mandatoryTransitionsCovered += coverage.mandatoryTransitionsCovered;
            
            if (coverage.mandatoryCoveragePercentage >= this.coverageThreshold) {
                this.results.modelsCovered++;
            }
            
            // Collect gaps from all models
            this.results.gaps.push(...coverage.gaps.filter(gap => gap.mandatory));
        }
    }

    /**
     * Validate MBD acceptance criteria
     */
    validateAcceptanceCriteria() {
        const mandatoryCoverage = this.results.mandatoryTransitions > 0 ?
            (this.results.mandatoryTransitionsCovered / this.results.mandatoryTransitions) * 100 : 100;
        
        this.results.mandatoryCoveragePercentage = mandatoryCoverage;
        this.results.passed = mandatoryCoverage >= this.coverageThreshold;
        
        console.log(`📈 Overall mandatory transition coverage: ${mandatoryCoverage.toFixed(1)}%`);
        console.log(`🎯 Coverage threshold: ${this.coverageThreshold}%`);
        console.log(`${this.results.passed ? '✅' : '❌'} Acceptance criteria: ${this.results.passed ? 'PASSED' : 'FAILED'}`);
    }

    /**
     * Generate detailed coverage report
     */
    async generateCoverageReport() {
        const reportPath = path.join(this.reportsDir, 'model_coverage_report.json');
        const htmlReportPath = path.join(this.reportsDir, 'model_coverage_report.html');
        
        // Ensure reports directory exists
        if (!fs.existsSync(this.reportsDir)) {
            fs.mkdirSync(this.reportsDir, { recursive: true });
        }
        
        // Generate JSON report
        const jsonReport = {
            ...this.results,
            generatedAt: new Date().toISOString(),
            coverageThreshold: this.coverageThreshold
        };
        
        fs.writeFileSync(reportPath, JSON.stringify(jsonReport, null, 2));
        console.log(`📄 JSON coverage report: ${reportPath}`);
        
        // Generate HTML report
        const htmlReport = this.generateHtmlReport(jsonReport);
        fs.writeFileSync(htmlReportPath, htmlReport);
        console.log(`📄 HTML coverage report: ${htmlReportPath}`);
    }

    /**
     * Generate HTML coverage report
     */
    generateHtmlReport(results) {
        const mandatoryCoverage = results.mandatoryCoveragePercentage || 0;
        const statusColor = results.passed ? '#28a745' : '#dc3545';
        const statusText = results.passed ? 'PASSED' : 'FAILED';
        
        return `<!DOCTYPE html>
<html>
<head>
    <title>Model Coverage Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #f8f9fa; padding: 20px; border-radius: 5px; }
        .status { color: ${statusColor}; font-weight: bold; font-size: 1.2em; }
        .metric { display: inline-block; margin: 10px 20px; text-align: center; }
        .metric-value { font-size: 2em; font-weight: bold; }
        .metric-label { color: #666; }
        .model-table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        .model-table th, .model-table td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        .model-table th { background: #f8f9fa; }
        .coverage-bar { width: 100px; height: 20px; background: #e9ecef; border-radius: 10px; overflow: hidden; }
        .coverage-fill { height: 100%; background: linear-gradient(90deg, #dc3545 0%, #ffc107 50%, #28a745 100%); }
        .gaps { background: #fff3cd; padding: 15px; border-radius: 5px; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Model Coverage Analysis Report</h1>
        <p>Generated: ${results.generatedAt}</p>
        <p class="status">Status: ${statusText}</p>
    </div>
    
    <div class="metrics">
        <div class="metric">
            <div class="metric-value">${mandatoryCoverage.toFixed(1)}%</div>
            <div class="metric-label">Mandatory Coverage</div>
        </div>
        <div class="metric">
            <div class="metric-value">${results.mandatoryTransitionsCovered}</div>
            <div class="metric-label">Mandatory Covered</div>
        </div>
        <div class="metric">
            <div class="metric-value">${results.mandatoryTransitions}</div>
            <div class="metric-label">Mandatory Total</div>
        </div>
        <div class="metric">
            <div class="metric-value">${results.totalModels}</div>
            <div class="metric-label">Models Analyzed</div>
        </div>
    </div>
    
    <h2>Coverage by Model</h2>
    <table class="model-table">
        <tr>
            <th>Model</th>
            <th>Type</th>
            <th>Mandatory Coverage</th>
            <th>States</th>
            <th>Transitions</th>
            <th>Gaps</th>
        </tr>
        ${Object.values(results.coverageByModel).map(model => `
        <tr>
            <td>${model.modelId}</td>
            <td>${model.modelType}</td>
            <td>
                <div class="coverage-bar">
                    <div class="coverage-fill" style="width: ${model.mandatoryCoveragePercentage}%"></div>
                </div>
                ${model.mandatoryCoveragePercentage.toFixed(1)}%
            </td>
            <td>${model.statesCovered}/${model.totalStates}</td>
            <td>${model.transitionsCovered}/${model.totalTransitions}</td>
            <td>${model.gaps.filter(g => g.mandatory).length}</td>
        </tr>
        `).join('')}
    </table>
    
    ${results.gaps.length > 0 ? `
    <div class="gaps">
        <h2>Coverage Gaps (Mandatory)</h2>
        <ul>
            ${results.gaps.map(gap => `
            <li><strong>${gap.type}:</strong> ${gap.id} ${gap.name ? `(${gap.name})` : ''} 
                ${gap.from && gap.to ? `[${gap.from} → ${gap.to}]` : ''}</li>
            `).join('')}
        </ul>
    </div>
    ` : ''}
    
</body>
</html>`;
    }
}

// CLI Interface
if (import.meta.url === `file://${process.argv[1]}`) {
    const analyzer = new ModelCoverageAnalyzer();
    
    analyzer.analyzeCoverage()
        .then(results => {
            console.log(`\n📊 Coverage Analysis Summary:`);
            console.log(`   Mandatory Transition Coverage: ${results.mandatoryCoveragePercentage?.toFixed(1) || 0}%`);
            console.log(`   Acceptance Criteria: ${results.passed ? 'PASSED' : 'FAILED'}`);
            console.log(`   Coverage Gaps: ${results.gaps.length}`);
            
            process.exit(results.passed ? 0 : 1);
        })
        .catch(error => {
            console.error('❌ Analysis failed:', error);
            process.exit(1);
        });
}

export default ModelCoverageAnalyzer;