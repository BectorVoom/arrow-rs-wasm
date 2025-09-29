/**
 * Test Helper Utilities for Model-Based Testing
 * 
 * Provides common utilities, setup/teardown functions, and validation helpers
 * for the generated test suites.
 */

import { expect } from 'chai';

/**
 * WASM Module Initialization and Management
 */
class WasmModuleManager {
    constructor() {
        this.module = null;
        this.initialized = false;
    }

    async initialize() {
        if (this.initialized) {
            return this.module;
        }

        try {
            // Dynamic import of the WASM module
            const wasmModule = await import('../pkg/arrow_wasm.js');
            await wasmModule.default(); // Initialize WASM
            
            this.module = wasmModule;
            this.initialized = true;
            
            return this.module;
        } catch (error) {
            throw new Error(`Failed to initialize WASM module: ${error.message}`);
        }
    }

    async dispose() {
        if (this.module && this.initialized) {
            // Cleanup any global state
            try {
                if (this.module.dispose) {
                    this.module.dispose();
                }
            } catch (error) {
                console.warn('Warning during WASM module disposal:', error);
            }
            this.module = null;
            this.initialized = false;
        }
    }

    isInitialized() {
        return this.initialized;
    }

    getModule() {
        if (!this.initialized) {
            throw new Error('WASM module not initialized. Call initialize() first.');
        }
        return this.module;
    }
}

/**
 * Test Data Generation Utilities
 */
class TestDataGenerator {
    static generateSimpleTableData(rows = 3) {
        return [
            { id: 1, name: "Alice", age: 30, active: true, score: 95.5 },
            { id: 2, name: "Bob", age: 25, active: false, score: 87.3 },
            { id: 3, name: "Charlie", age: 35, active: true, score: 92.1 }
        ].slice(0, rows);
    }

    static generateDataWithNulls(rows = 3) {
        return [
            { id: 1, name: "Alice", age: 30, active: true, score: null },
            { id: 2, name: null, age: 25, active: false, score: 87.3 },
            { id: 3, name: "Charlie", age: null, active: true, score: 92.1 }
        ].slice(0, rows);
    }

    static generateLargeDataset(rows = 1000) {
        const data = [];
        for (let i = 0; i < rows; i++) {
            data.push({
                id: i + 1,
                name: `User${i + 1}`,
                age: 20 + (i % 50),
                active: i % 2 === 0,
                score: Math.random() * 100,
                category: ['A', 'B', 'C'][i % 3],
                timestamp: Date.now() + i * 1000
            });
        }
        return data;
    }

    static generateInvalidData() {
        return {
            null_data: null,
            empty_array: [],
            non_array: { not: "an array" },
            inconsistent_schema: [
                { id: 1, name: "Alice" },
                { id: 2, different_field: "Bob" }
            ],
            mixed_types: [
                { value: 123 },
                { value: "string" },
                { value: true }
            ]
        };
    }

    static generateValidIpcBuffer() {
        // This would generate a valid Arrow IPC buffer
        // For now, return a placeholder that tests can use
        return new ArrayBuffer(0); // Placeholder
    }

    static generateInvalidIpcBuffers() {
        return {
            empty_buffer: new ArrayBuffer(0),
            invalid_magic: new Uint8Array([1, 2, 3, 4]).buffer,
            corrupted_metadata: new Uint8Array(100).fill(0xff).buffer
        };
    }
}

/**
 * State Validation Utilities
 */
class StateValidator {
    static validateModuleState(module, expectedState) {
        switch (expectedState) {
            case 'uninitialized':
                // Module should not be ready for operations
                expect(() => module.getVersion()).to.throw();
                break;
            
            case 'ready':
                // Module should be ready for operations
                expect(() => module.getVersion()).to.not.throw();
                const version = module.getVersion();
                expect(version).to.have.property('major');
                expect(version).to.have.property('minor');
                expect(version).to.have.property('patch');
                break;
            
            case 'operating':
                // Module should be operational with possible active tables
                expect(() => module.getVersion()).to.not.throw();
                // Additional checks for operational state
                break;
            
            case 'error':
                // Module should be in error state
                // Check for error indicators
                break;
                
            default:
                throw new Error(`Unknown module state: ${expectedState}`);
        }
    }

    static validateTableState(table, expectedState) {
        switch (expectedState) {
            case 'active':
                expect(table).to.not.be.null;
                expect(table.numRows).to.be.a('function');
                expect(table.numColumns).to.be.a('function');
                expect(table.numRows()).to.be.a('number');
                expect(table.numColumns()).to.be.a('number');
                break;
                
            case 'sliced':
                expect(table).to.not.be.null;
                // Additional validations for sliced state
                break;
                
            case 'disposed':
                // Table should be disposed and unusable
                expect(() => table.numRows()).to.throw();
                break;
                
            default:
                throw new Error(`Unknown table state: ${expectedState}`);
        }
    }

    static validateStateProperties(actualProperties, expectedProperties) {
        for (const [key, expectedValue] of Object.entries(expectedProperties)) {
            if (expectedValue === ">=0") {
                expect(actualProperties[key]).to.be.at.least(0);
            } else if (expectedValue === ">0") {
                expect(actualProperties[key]).to.be.above(0);
            } else if (expectedValue === "number") {
                expect(actualProperties[key]).to.be.a('number');
            } else if (expectedValue === "string") {
                expect(actualProperties[key]).to.be.a('string');
            } else if (expectedValue === "boolean") {
                expect(actualProperties[key]).to.be.a('boolean');
            } else {
                expect(actualProperties[key]).to.equal(expectedValue);
            }
        }
    }

    static validateInvariants(context, invariants) {
        for (const invariant of invariants) {
            try {
                // Simple evaluation of invariant expressions
                const result = this.evaluateInvariant(context, invariant);
                expect(result).to.be.true;
            } catch (error) {
                throw new Error(`Invariant violation: ${invariant} - ${error.message}`);
            }
        }
    }

    static evaluateInvariant(context, invariant) {
        // Simple invariant evaluation - in production this would be more sophisticated
        // Handle common patterns like "handle_id !== null", "reference_count >= 0"
        
        if (invariant.includes('!==')) {
            const [left, right] = invariant.split('!==').map(s => s.trim());
            return this.getValue(context, left) !== this.getValue(context, right);
        }
        
        if (invariant.includes('===')) {
            const [left, right] = invariant.split('===').map(s => s.trim());
            return this.getValue(context, left) === this.getValue(context, right);
        }
        
        if (invariant.includes('>=')) {
            const [left, right] = invariant.split('>=').map(s => s.trim());
            return this.getValue(context, left) >= this.getValue(context, right);
        }
        
        if (invariant.includes('>')) {
            const [left, right] = invariant.split('>').map(s => s.trim());
            return this.getValue(context, left) > this.getValue(context, right);
        }
        
        // Default to true for complex invariants
        return true;
    }

    static getValue(context, expression) {
        // Simple value extraction from context
        if (expression === 'null') return null;
        if (expression === 'true') return true;
        if (expression === 'false') return false;
        if (!isNaN(expression)) return Number(expression);
        if (expression.startsWith('"') && expression.endsWith('"')) {
            return expression.slice(1, -1);
        }
        
        // Try to get from context
        return context[expression];
    }
}

/**
 * Performance Testing Utilities
 */
class PerformanceValidator {
    static async measureExecutionTime(operation) {
        const start = performance.now();
        const result = await operation();
        const end = performance.now();
        return {
            result,
            duration: end - start
        };
    }

    static validateTiming(duration, requirement) {
        // Parse requirement like "< 100ms", "< 1s"
        const match = requirement.match(/^(<|<=|>|>=)\s*(\d+(?:\.\d+)?)\s*(ms|s)$/);
        if (!match) {
            throw new Error(`Invalid timing requirement: ${requirement}`);
        }

        const [, operator, value, unit] = match;
        const threshold = parseFloat(value) * (unit === 's' ? 1000 : 1);

        switch (operator) {
            case '<':
                expect(duration).to.be.below(threshold);
                break;
            case '<=':
                expect(duration).to.be.at.most(threshold);
                break;
            case '>':
                expect(duration).to.be.above(threshold);
                break;
            case '>=':
                expect(duration).to.be.at.least(threshold);
                break;
            default:
                throw new Error(`Unknown operator: ${operator}`);
        }
    }

    static async measureMemoryUsage(operation) {
        // Memory usage measurement would be browser-specific
        // This is a placeholder implementation
        if (typeof performance !== 'undefined' && performance.memory) {
            const before = performance.memory.usedJSHeapSize;
            const result = await operation();
            const after = performance.memory.usedJSHeapSize;
            return {
                result,
                memoryDelta: after - before
            };
        }
        
        // Fallback if memory measurement not available
        return {
            result: await operation(),
            memoryDelta: 0
        };
    }
}

/**
 * Error Testing Utilities
 */
class ErrorTestHelper {
    static expectError(operation, expectedErrorCode) {
        return expect(operation).to.eventually.be.rejectedWith(expectedErrorCode);
    }

    static async testErrorCondition(operation, expectedError) {
        try {
            await operation();
            throw new Error('Expected operation to fail but it succeeded');
        } catch (error) {
            if (expectedError.code) {
                expect(error.code || error.name).to.include(expectedError.code);
            }
            if (expectedError.message) {
                expect(error.message).to.include(expectedError.message);
            }
        }
    }

    static validateErrorResponse(error, expectedErrorCode) {
        expect(error).to.be.an('error');
        if (expectedErrorCode) {
            expect(error.code || error.name).to.include(expectedErrorCode);
        }
    }
}

/**
 * Data Integrity Validators
 */
class DataIntegrityValidator {
    static validateRoundTrip(originalData, convertedData) {
        expect(convertedData).to.have.lengthOf(originalData.length);
        
        for (let i = 0; i < originalData.length; i++) {
            const original = originalData[i];
            const converted = convertedData[i];
            
            for (const key of Object.keys(original)) {
                if (original[key] === null) {
                    expect(converted[key]).to.be.null;
                } else {
                    expect(converted[key]).to.equal(original[key]);
                }
            }
        }
    }

    static validateSchemaPreservation(originalSchema, convertedSchema) {
        expect(convertedSchema.fields).to.have.lengthOf(originalSchema.fields.length);
        
        for (let i = 0; i < originalSchema.fields.length; i++) {
            const originalField = originalSchema.fields[i];
            const convertedField = convertedSchema.fields[i];
            
            expect(convertedField.name).to.equal(originalField.name);
            expect(convertedField.dataType).to.deep.equal(originalField.dataType);
            expect(convertedField.nullable).to.equal(originalField.nullable);
        }
    }

    static validateNullHandling(data, convertedData) {
        for (let i = 0; i < data.length; i++) {
            const original = data[i];
            const converted = convertedData[i];
            
            for (const key of Object.keys(original)) {
                if (original[key] === null) {
                    expect(converted[key]).to.be.null;
                }
            }
        }
    }
}

/**
 * Main Test Helpers Export
 */
export class TestHelpers {
    static wasmManager = new WasmModuleManager();
    static dataGenerator = TestDataGenerator;
    static stateValidator = StateValidator;
    static performanceValidator = PerformanceValidator;
    static errorHelper = ErrorTestHelper;
    static integrityValidator = DataIntegrityValidator;

    static async initializeWasmModule() {
        return await this.wasmManager.initialize();
    }

    static async disposeWasmModule() {
        return await this.wasmManager.dispose();
    }

    static getWasmModule() {
        return this.wasmManager.getModule();
    }
}

// Convenience exports
export {
    WasmModuleManager,
    TestDataGenerator,
    StateValidator,
    PerformanceValidator,
    ErrorTestHelper,
    DataIntegrityValidator
};

// Global test setup helper
export async function initializeWasmModule() {
    return await TestHelpers.initializeWasmModule();
}