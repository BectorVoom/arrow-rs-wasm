/**
 * Model-Based Test Generation Framework
 * 
 * This script automatically generates executable test cases from behavioral models,
 * maintaining full traceability between requirements, models, and tests.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

class ModelTestGenerator {
    constructor() {
        this.modelsPath = path.join(__dirname, '../models');
        this.generatedPath = path.join(__dirname, '../generated');
        this.traceabilityMatrix = new Map();
        this.generatedTests = [];
    }

    /**
     * Generate all tests from models
     */
    async generateAllTests() {
        console.log('🔄 Starting model-based test generation...');
        
        // Load all models
        const models = await this.loadAllModels();
        console.log(`📋 Loaded ${models.length} models`);
        
        // Generate tests for each model
        for (const model of models) {
            await this.generateTestsFromModel(model);
        }
        
        // Generate traceability matrix
        await this.generateTraceabilityMatrix();
        
        // Generate test suite runner
        await this.generateTestSuiteRunner();
        
        console.log(`✅ Generated ${this.generatedTests.length} test cases`);
    }

    /**
     * Load all models from the models directory
     */
    async loadAllModels() {
        const models = [];
        
        // Load behavioral models
        const behavioralPath = path.join(this.modelsPath, 'behavioral');
        models.push(...await this.loadModelsFromDir(behavioralPath, 'behavioral'));
        
        // Load structural models  
        const structuralPath = path.join(this.modelsPath, 'structural');
        models.push(...await this.loadModelsFromDir(structuralPath, 'structural'));
        
        // Load error models
        const errorPath = path.join(this.modelsPath, 'error');
        models.push(...await this.loadModelsFromDir(errorPath, 'error'));
        
        return models;
    }

    /**
     * Load models from a specific directory
     */
    async loadModelsFromDir(dir, category) {
        const models = [];
        
        if (!fs.existsSync(dir)) {
            return models;
        }
        
        const entries = fs.readdirSync(dir, { withFileTypes: true });
        
        for (const entry of entries) {
            const fullPath = path.join(dir, entry.name);
            
            if (entry.isDirectory()) {
                // Recursively load from subdirectories
                models.push(...await this.loadModelsFromDir(fullPath, category));
            } else if (entry.name.endsWith('.json')) {
                try {
                    const modelContent = fs.readFileSync(fullPath, 'utf8');
                    const model = JSON.parse(modelContent);
                    model._category = category;
                    model._filepath = fullPath;
                    models.push(model);
                } catch (error) {
                    console.warn(`⚠️  Failed to load model ${fullPath}: ${error.message}`);
                }
            }
        }
        
        return models;
    }

    /**
     * Generate tests from a single model
     */
    async generateTestsFromModel(model) {
        console.log(`🧪 Generating tests for model: ${model.model_id}`);
        
        switch (model.model_type) {
            case 'state_machine':
                await this.generateStateMachineTests(model);
                break;
            case 'statechart':
                await this.generateStatechartTests(model);
                break;
            case 'component':
                await this.generateComponentTests(model);
                break;
            case 'data_flow':
                await this.generateDataFlowTests(model);
                break;
            default:
                console.warn(`⚠️  Unknown model type: ${model.model_type}`);
        }
    }

    /**
     * Generate tests for state machine models
     */
    async generateStateMachineTests(model) {
        const testSuite = {
            model_id: model.model_id,
            model_type: 'state_machine',
            generated_at: new Date().toISOString(),
            test_cases: []
        };

        // Generate state validation tests
        const states = Array.isArray(model.states) ? model.states : Object.values(model.states || {});
        for (const state of states) {
            const stateTest = this.generateStateValidationTest(model, state);
            testSuite.test_cases.push(stateTest);
            this.addToTraceability(model, stateTest, 'state_validation');
        }

        // Generate transition tests
        const transitions = Array.isArray(model.transitions) ? model.transitions : Object.values(model.transitions || {});
        for (const transition of transitions) {
            const transitionTest = this.generateTransitionTest(model, transition, states);
            testSuite.test_cases.push(transitionTest);
            this.addToTraceability(model, transitionTest, 'transition_test');
        }

        // Generate error path tests
        const errorTests = this.generateErrorPathTests(model);
        testSuite.test_cases.push(...errorTests);

        // Generate invariant tests
        const invariantTests = this.generateInvariantTests(model);
        testSuite.test_cases.push(...invariantTests);

        await this.writeTestSuite(model.model_id, testSuite);
    }

    /**
     * Generate state validation test
     */
    generateStateValidationTest(model, state) {
        const testId = `${model.model_id}_state_${state.id}_validation`;
        
        return {
            test_id: testId,
            test_name: `Validate ${state.name} state properties and invariants`,
            test_type: 'state_validation',
            model_element: state.id,
            requirements: model.requirements || [],
            test_steps: [
                {
                    step: 1,
                    action: `Navigate system to ${state.name} state`,
                    expected: `System reaches ${state.name} state`
                },
                {
                    step: 2,
                    action: 'Validate state properties',
                    expected: `Properties match: ${JSON.stringify(state.properties)}`,
                    validation_code: this.generatePropertyValidation(state.properties)
                },
                {
                    step: 3,
                    action: 'Check state invariants',
                    expected: 'All invariants hold true',
                    validation_code: this.generateInvariantValidation(state.invariants)
                }
            ],
            browser_test_code: this.generateBrowserTestCode(testId, model, state, 'state'),
            priority: state.type === 'initial' ? 'high' : 'medium'
        };
    }

    /**
     * Generate transition test
     */
    generateTransitionTest(model, transition, states) {
        const testId = `${model.model_id}_transition_${transition.id}`;
        
        const fromState = states.find(s => s.id === transition.from);
        const toState = states.find(s => s.id === transition.to);
        
        return {
            test_id: testId,
            test_name: `Test transition: ${fromState?.name} → ${toState?.name}`,
            test_type: 'transition_test',
            model_element: transition.id,
            requirements: transition.requirements || [],
            test_steps: [
                {
                    step: 1,
                    action: `Ensure system is in ${fromState?.name} state`,
                    expected: `System state is ${transition.from}`
                },
                {
                    step: 2,
                    action: `Validate guard condition: ${transition.guard}`,
                    expected: 'Guard condition evaluates to true',
                    validation_code: this.generateGuardValidation(transition.guard)
                },
                {
                    step: 3,
                    action: `Trigger event: ${transition.trigger}`,
                    expected: `Event ${transition.trigger} is processed`
                },
                {
                    step: 4,
                    action: `Execute action: ${transition.action}`,
                    expected: `Action ${transition.action} completes successfully`
                },
                {
                    step: 5,
                    action: 'Verify state transition',
                    expected: `System transitions to ${toState?.name} state`
                },
                {
                    step: 6,
                    action: 'Validate target state properties',
                    expected: 'Target state properties are correct',
                    validation_code: this.generatePropertyValidation(toState?.properties)
                }
            ],
            test_data: transition.test_data || {},
            browser_test_code: this.generateBrowserTestCode(testId, model, transition, 'transition'),
            priority: transition.requirements?.length > 0 ? 'high' : 'medium'
        };
    }

    /**
     * Generate browser test code for execution
     */
    generateBrowserTestCode(testId, model, element, type) {
        return `
// Generated test for ${testId}
// Model: ${model.model_id}, Element: ${element.id || element.name}

describe('${testId}', () => {
    let wasmModule = null;
    
    beforeEach(async () => {
        // Initialize WASM module
        wasmModule = await initializeWasmModule();
    });
    
    afterEach(async () => {
        // Cleanup
        if (wasmModule) {
            await wasmModule.dispose();
        }
    });
    
    it('should ${type === 'state' ? 'validate state properties' : 'execute transition successfully'}', async () => {
        ${this.generateTestImplementation(model, element, type)}
    });
    
    ${this.generateErrorCaseTests(model, element)}
    
    ${this.generatePerformanceTests(model, element)}
});
`;
    }

    /**
     * Generate test implementation code
     */
    generateTestImplementation(model, element, type) {
        if (type === 'state') {
            return this.generateStateTestImplementation(model, element);
        } else if (type === 'transition') {
            return this.generateTransitionTestImplementation(model, element);
        }
        return '// Test implementation to be completed';
    }

    /**
     * Generate state test implementation
     */
    generateStateTestImplementation(model, state) {
        return `
        // Navigate to ${state.name} state
        ${this.generateStateNavigationCode(model, state)}
        
        // Validate state properties
        ${this.generateStatePropertyValidation(state)}
        
        // Check invariants
        ${this.generateStateInvariantChecks(state)}
        `;
    }

    /**
     * Generate transition test implementation
     */
    generateTransitionTestImplementation(model, transition) {
        return `
        // Setup initial state: ${transition.from}
        ${this.generateStateSetupCode(model, transition.from)}
        
        // Verify guard condition
        ${this.generateGuardConditionCheck(transition)}
        
        // Trigger transition
        ${this.generateTransitionTriggerCode(transition)}
        
        // Verify target state
        ${this.generateTargetStateValidation(model, transition.to)}
        `;
    }

    /**
     * Generate various code snippets for test implementation
     */
    generateStateNavigationCode(model, state) {
        // Model-specific navigation logic based on model type
        if (model.model_id === 'module_lifecycle') {
            return this.generateModuleLifecycleNavigation(state);
        } else if (model.model_id === 'table_lifecycle') {
            return this.generateTableLifecycleNavigation(state);
        }
        return '// Navigation code to be implemented';
    }

    generateModuleLifecycleNavigation(state) {
        switch (state.id) {
            case 'uninitialized':
                return '// Module should be in uninitialized state by default';
            case 'loading':
                return 'await wasmModule.startLoading();';
            case 'initializing':
                return 'await wasmModule.initialize();';
            case 'ready':
                return 'await wasmModule.initialize(); await wasmModule.waitForReady();';
            default:
                return `// Navigate to ${state.id} state`;
        }
    }

    generateTableLifecycleNavigation(state) {
        switch (state.id) {
            case 'nonexistent':
                return '// Table should not exist initially';
            case 'creating_from_json':
                return 'wasmModule.tableFromJSON(testData); // Don\'t await to catch in creating state';
            case 'active':
                return 'const table = await wasmModule.tableFromJSON(testData);';
            default:
                return `// Navigate to ${state.id} state`;
        }
    }

    /**
     * Generate error case tests
     */
    generateErrorCaseTests(model, element) {
        return `
    describe('Error Cases', () => {
        ${this.generateErrorConditionTests(element)}
    });`;
    }

    /**
     * Generate performance tests
     */
    generatePerformanceTests(model, element) {
        return `
    describe('Performance', () => {
        it('should meet timing requirements', async () => {
            ${this.generatePerformanceTestCode(element)}
        });
    });`;
    }

    /**
     * Add test to traceability matrix
     */
    addToTraceability(model, test, testType) {
        const traceabilityId = `${model.model_id}_${test.test_id}`;
        
        this.traceabilityMatrix.set(traceabilityId, {
            requirement_ids: model.requirements || [],
            model_id: model.model_id,
            model_element: test.model_element,
            test_id: test.test_id,
            test_type: testType,
            generated_at: new Date().toISOString()
        });
    }

    /**
     * Write test suite to file
     */
    async writeTestSuite(modelId, testSuite) {
        const outputPath = path.join(this.generatedPath, `${modelId}_tests.json`);
        
        // Ensure directory exists
        fs.mkdirSync(path.dirname(outputPath), { recursive: true });
        
        // Write test suite
        fs.writeFileSync(outputPath, JSON.stringify(testSuite, null, 2));
        
        // Generate executable test file
        const executablePath = path.join(this.generatedPath, `${modelId}_tests.spec.js`);
        const executableContent = this.generateExecutableTestFile(testSuite);
        fs.writeFileSync(executablePath, executableContent);
        
        this.generatedTests.push({
            model_id: modelId,
            test_count: testSuite.test_cases.length,
            output_files: [outputPath, executablePath]
        });
        
        console.log(`  📝 Generated ${testSuite.test_cases.length} tests for ${modelId}`);
    }

    /**
     * Generate executable test file content
     */
    generateExecutableTestFile(testSuite) {
        return `
// Generated test suite for ${testSuite.model_id}
// Generated at: ${testSuite.generated_at}

import { expect } from 'chai';
import { initializeWasmModule, TestHelpers } from '../harness/test_helpers.js';

${testSuite.test_cases.map(test => test.browser_test_code).join('\n\n')}

// Test suite metadata
export const testSuiteInfo = ${JSON.stringify({
    model_id: testSuite.model_id,
    test_count: testSuite.test_cases.length,
    generated_at: testSuite.generated_at
}, null, 2)};
`;
    }

    /**
     * Generate traceability matrix
     */
    async generateTraceabilityMatrix() {
        const matrix = Array.from(this.traceabilityMatrix.values());
        const outputPath = path.join(__dirname, '../traceability_matrix.csv');
        
        // Generate CSV content
        const csvHeader = 'Requirement ID,Model ID,Model Element,Test ID,Test Type,Generated At\n';
        const csvRows = matrix.map(entry => {
            const reqIds = Array.isArray(entry.requirement_ids) ? entry.requirement_ids.join(';') : entry.requirement_ids;
            return `"${reqIds}","${entry.model_id}","${entry.model_element}","${entry.test_id}","${entry.test_type}","${entry.generated_at}"`;
        }).join('\n');
        
        fs.writeFileSync(outputPath, csvHeader + csvRows);
        
        // Also generate JSON version
        const jsonPath = path.join(__dirname, '../traceability_matrix.json');
        fs.writeFileSync(jsonPath, JSON.stringify(matrix, null, 2));
        
        console.log(`📋 Generated traceability matrix with ${matrix.length} entries`);
    }

    /**
     * Generate test suite runner
     */
    async generateTestSuiteRunner() {
        const runnerContent = `
// Generated test suite runner
// Executes all model-derived tests

import { execSync } from 'child_process';
import path from 'path';

const testSuites = ${JSON.stringify(this.generatedTests, null, 2)};

async function runAllTests() {
    console.log('🚀 Running model-derived test suite...');
    
    let totalPassed = 0;
    let totalFailed = 0;
    
    for (const suite of testSuites) {
        console.log(\`\\n📋 Running tests for \${suite.model_id} (\${suite.test_count} tests)...\`);
        
        try {
            const testFile = suite.output_files.find(f => f.endsWith('.spec.js'));
            const result = execSync(\`npx mocha "\${testFile}"\`, { 
                cwd: process.cwd(),
                encoding: 'utf8' 
            });
            
            console.log(\`✅ \${suite.model_id}: All tests passed\`);
            totalPassed += suite.test_count;
            
        } catch (error) {
            console.error(\`❌ \${suite.model_id}: Tests failed\`);
            console.error(error.stdout || error.message);
            totalFailed += suite.test_count;
        }
    }
    
    console.log(\`\\n📊 Test Summary:\`);
    console.log(\`  ✅ Passed: \${totalPassed}\`);
    console.log(\`  ❌ Failed: \${totalFailed}\`);
    console.log(\`  📈 Success Rate: \${(totalPassed / (totalPassed + totalFailed) * 100).toFixed(1)}%\`);
    
    return totalFailed === 0;
}

if (import.meta.url === \`file://\${process.argv[1]}\`) {
    runAllTests().then(success => {
        process.exit(success ? 0 : 1);
    });
}

export { runAllTests, testSuites };
`;
        
        const runnerPath = path.join(this.generatedPath, 'run_all_tests.js');
        fs.writeFileSync(runnerPath, runnerContent);
        
        console.log('🏃 Generated test suite runner');
    }

    // Utility methods for code generation (stubs for now)
    generatePropertyValidation(properties) { return '// Property validation code'; }
    generateInvariantValidation(invariants) { return '// Invariant validation code'; }
    generateGuardValidation(guard) { return '// Guard validation code'; }
    generateErrorPathTests(model) { return []; }
    generateInvariantTests(model) { return []; }
    generateStatechartTests(model) { return this.generateStateMachineTests(model); }
    generateComponentTests(model) { /* Component-specific tests */ }
    generateDataFlowTests(model) { /* Data flow specific tests */ }
    generateStatePropertyValidation(state) { return '// State property validation'; }
    generateStateInvariantChecks(state) { return '// State invariant checks'; }
    generateStateSetupCode(model, stateId) { return '// State setup code'; }
    generateGuardConditionCheck(transition) { return '// Guard condition check'; }
    generateTransitionTriggerCode(transition) { return '// Transition trigger code'; }
    generateTargetStateValidation(model, stateId) { return '// Target state validation'; }
    generateErrorConditionTests(element) { return '// Error condition tests'; }
    generatePerformanceTestCode(element) { return '// Performance test code'; }
}

// CLI interface
if (import.meta.url === `file://${process.argv[1]}`) {
    const generator = new ModelTestGenerator();
    generator.generateAllTests().catch(console.error);
}

export { ModelTestGenerator };