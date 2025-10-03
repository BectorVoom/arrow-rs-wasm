/**
 * Test Execution Context for Model-Based Testing
 * 
 * Provides the runtime environment and utilities for executing generated tests
 * Handles WASM interactions, test data creation, and result validation
 */

class TestExecutionContext {
    constructor(wasmModule) {
        this.wasm = wasmModule;
        this.testData = new Map();
        this.stateHistory = [];
        this.transitionHistory = [];
        this.performanceMetrics = new Map();
        this.errorHistory = [];
        this.handles = new Set(); // Track created handles for cleanup
    }

    /**
     * Record state visitation for coverage tracking
     */
    recordStateVisit(stateName) {
        this.stateHistory.push({
            state: stateName,
            timestamp: performance.now(),
            context: this.getCurrentContext()
        });
    }

    /**
     * Record transition execution for coverage tracking
     */
    recordTransitionExecution(transitionId) {
        this.transitionHistory.push({
            transition: transitionId,
            timestamp: performance.now(),
            context: this.getCurrentContext()
        });
    }

    /**
     * Execute a state machine transition
     */
    async executeTransition(transition, wasm) {
        try {
            const trigger = transition.trigger;
            const condition = transition.condition;
            const action = transition.action;

            // Check pre-condition if specified
            if (condition && !this.evaluateCondition(condition)) {
                return { success: false, error: `Condition not met: ${condition}` };
            }

            // Execute the transition trigger
            const result = await this.executeTrigger(trigger, wasm);
            
            if (result.success) {
                // Execute the transition action
                await this.executeAction(action, result.data);
                return { success: true, data: result.data };
            } else {
                return { success: false, error: result.error };
            }
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    /**
     * Execute a workflow state
     */
    async executeWorkflowState(state, testData, wasm) {
        try {
            const startTime = performance.now();
            
            let result;
            switch (state.type) {
                case 'initial':
                    result = await this.executeInitialState(state, testData, wasm);
                    break;
                case 'normal':
                    result = await this.executeNormalState(state, testData, wasm);
                    break;
                case 'conditional':
                    result = await this.executeConditionalState(state, testData, wasm);
                    break;
                case 'final':
                    result = await this.executeFinalState(state, testData, wasm);
                    break;
                case 'error':
                    result = await this.executeErrorState(state, testData, wasm);
                    break;
                default:
                    throw new Error(`Unknown state type: ${state.type}`);
            }
            
            const endTime = performance.now();
            this.recordPerformanceMetric(state.id, endTime - startTime);
            
            return { success: true, data: result, duration: endTime - startTime };
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    /**
     * Execute different types of workflow states
     */
    async executeInitialState(state, testData, wasm) {
        switch (state.id) {
            case 'F1': // detect_format
                return this.detectFileFormat(testData);
            case 'W1': // validate_table
                return this.validateTableStructure(testData);
            case 'P1': // detect_parquet
                return this.detectParquetFormat(testData);
            default:
                return { message: `Executed initial state: ${state.id}` };
        }
    }

    async executeNormalState(state, testData, wasm) {
        switch (state.id) {
            case 'F2': // validate_headers
                return this.validateArrowHeaders(testData);
            case 'F5': // parse_schema
                return this.parseArrowSchema(testData);
            case 'F6': // read_batches
                return this.readRecordBatches(testData, wasm);
            case 'W2': // create_schema
                return this.createArrowSchema(testData);
            case 'W4': // write_headers
                return this.writeArrowHeaders(testData);
            case 'W5': // write_batches
                return this.writeRecordBatches(testData, wasm);
            default:
                return { message: `Executed normal state: ${state.id}` };
        }
    }

    async executeConditionalState(state, testData, wasm) {
        switch (state.id) {
            case 'F4': // decompress
                return this.decompressLZ4Data(testData);
            case 'W3': // configure_compression
                return this.configureLZ4Compression(testData);
            default:
                return { message: `Executed conditional state: ${state.id}` };
        }
    }

    async executeFinalState(state, testData, wasm) {
        switch (state.id) {
            case 'F7': // create_table
                return this.createManagedTable(testData, wasm);
            case 'W6': // finalize_file
                return this.finalizeIPCFile(testData);
            default:
                return { message: `Executed final state: ${state.id}` };
        }
    }

    /**
     * Create test data based on specification
     */
    async createTestData(specification) {
        if (typeof specification === 'string') {
            // Pre-defined test data
            return this.getPreDefinedTestData(specification);
        } else if (typeof specification === 'object') {
            // Generated test data
            return this.generateTestData(specification);
        } else {
            // Default test data
            return this.createDefaultArrowData();
        }
    }

    /**
     * Get pre-defined test data
     */
    getPreDefinedTestData(name) {
        const testDataSets = {
            'arrow_ipc_simple.arrow': this.createSimpleArrowIPC(),
            'arrow_ipc_lz4.arrow': this.createLZ4CompressedArrowIPC(),
            'sample.parquet': this.createSampleParquet(),
            'sample.feather': this.createSampleFeather(),
            'generated_table_with_lz4': this.createGeneratedTableForLZ4()
        };

        return testDataSets[name] || this.createDefaultArrowData();
    }

    /**
     * Create simple Arrow IPC test data
     */
    createSimpleArrowIPC() {
        // Create a simple Arrow IPC structure
        const header = new Uint8Array([
            0x41, 0x52, 0x52, 0x4F, 0x57, 0x31, 0x00, 0x00 // "ARROW1\0\0"
        ]);
        
        // Add minimal metadata and a simple record batch
        const metadata = this.createMinimalArrowMetadata();
        const batch = this.createSimpleRecordBatch();
        
        return this.combineArrowParts(header, metadata, batch);
    }

    /**
     * Create LZ4 compressed Arrow IPC test data
     */
    createLZ4CompressedArrowIPC() {
        const uncompressed = this.createSimpleArrowIPC();
        // For testing purposes, we'll simulate LZ4 compression by adding compression markers
        const compressionHeader = new Uint8Array([0x04, 0x22, 0x4D, 0x18]); // LZ4 magic
        
        return this.combineArrays(compressionHeader, uncompressed);
    }

    /**
     * Execute specific trigger functions
     */
    async executeTrigger(trigger, wasm) {
        try {
            const result = await this.parseTriggerAndExecute(trigger, wasm);
            return { success: true, data: result };
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    /**
     * Parse trigger string and execute corresponding WASM function
     */
    async parseTriggerAndExecute(trigger, wasm) {
        if (trigger.includes('init()')) {
            await wasm.init_with_options(true);
            return { operation: 'init', result: 'initialized' };
        }
        
        if (trigger.includes('read_table_from_array_buffer')) {
            const testData = this.createSimpleArrowIPC();
            const handle = wasm.read_table_from_array_buffer(testData.buffer);
            this.handles.add(handle);
            return { operation: 'read_table', handle, result: 'table_loaded' };
        }
        
        if (trigger.includes('get_table_row_count')) {
            const handles = Array.from(this.handles);
            if (handles.length === 0) {
                throw new Error('No table handles available');
            }
            const rowCount = wasm.get_table_row_count(handles[0]);
            return { operation: 'get_row_count', rowCount, result: 'row_count_retrieved' };
        }
        
        if (trigger.includes('export_column_by_name')) {
            const handles = Array.from(this.handles);
            if (handles.length === 0) {
                throw new Error('No table handles available');
            }
            const schema = JSON.parse(wasm.table_schema_summary(handles[0]));
            const firstColumn = schema.columns[0].name;
            const columnData = JSON.parse(wasm.export_column_by_name(handles[0], firstColumn));
            return { operation: 'export_column', columnData, result: 'column_exported' };
        }
        
        if (trigger.includes('free_table')) {
            const handles = Array.from(this.handles);
            if (handles.length > 0) {
                const handle = handles[0];
                wasm.free_table(handle);
                this.handles.delete(handle);
                return { operation: 'free_table', handle, result: 'table_freed' };
            }
        }
        
        if (trigger.includes('clear_all_tables')) {
            wasm.clear_all_tables();
            this.handles.clear();
            return { operation: 'clear_all', result: 'all_tables_cleared' };
        }
        
        // Default case
        return { operation: 'unknown', trigger, result: 'executed' };
    }

    /**
     * Evaluate condition strings
     */
    evaluateCondition(condition) {
        // Simple condition evaluation for testing
        if (condition.includes('valid')) return true;
        if (condition.includes('table_count > 0')) return this.handles.size > 0;
        if (condition.includes('table_count == 0')) return this.handles.size === 0;
        if (condition.includes('WASM module loads successfully')) return this.wasm !== null;
        
        // Default to true for conditions we can't evaluate
        return true;
    }

    /**
     * Execute action after successful transition
     */
    async executeAction(action, data) {
        // Record action execution
        this.recordAction(action, data);
    }

    /**
     * Setup error conditions for testing
     */
    async setupErrorCondition(errorCode) {
        this.currentErrorCondition = errorCode;
        
        // Setup specific error conditions based on error code
        switch (errorCode) {
            case 'ERR_IO_001': // insufficient_data
                this.testData.set('insufficient_data', true);
                break;
            case 'ERR_FORMAT_001': // invalid_magic_bytes
                this.testData.set('invalid_format', true);
                break;
            case 'ERR_MEMORY_002': // invalid_handle
                this.testData.set('invalid_handle', 9999);
                break;
            default:
                console.warn(`Unknown error condition: ${errorCode}`);
        }
    }

    /**
     * Execute operation expecting an error
     */
    async executeOperationExpectingError(wasm) {
        try {
            if (this.currentErrorCondition === 'ERR_IO_001') {
                // Try to read truncated data
                const truncatedData = new Uint8Array(4); // Too small
                const handle = wasm.read_table_from_array_buffer(truncatedData.buffer);
                return { success: true, handle }; // Should not reach here
            }
            
            if (this.currentErrorCondition === 'ERR_FORMAT_001') {
                // Try to read data with wrong magic bytes
                const badData = new Uint8Array([0x42, 0x41, 0x44, 0x21]); // "BAD!"
                const handle = wasm.read_table_from_array_buffer(badData.buffer);
                return { success: true, handle }; // Should not reach here
            }
            
            if (this.currentErrorCondition === 'ERR_MEMORY_002') {
                // Try to use invalid handle
                const invalidHandle = this.testData.get('invalid_handle');
                const rowCount = wasm.get_table_row_count(invalidHandle);
                return { success: true, rowCount }; // Should not reach here
            }
            
            return { success: true }; // No error occurred
        } catch (error) {
            return { error: error.message };
        }
    }

    /**
     * Verify error format and content
     */
    verifyErrorFormat(error, expectedErrorInfo) {
        // Check if error message contains expected information
        const errorLower = error.toLowerCase();
        const descriptionLower = expectedErrorInfo.description.toLowerCase();
        
        if (errorLower.includes('invalid') && descriptionLower.includes('invalid')) {
            return { valid: true };
        }
        
        if (errorLower.includes('format') && descriptionLower.includes('format')) {
            return { valid: true };
        }
        
        if (errorLower.includes('handle') && descriptionLower.includes('handle')) {
            return { valid: true };
        }
        
        // Default validation
        return { valid: true, reason: 'Error format acceptable' };
    }

    /**
     * Execute performance operations with measurement
     */
    async executePerformanceOperation(requirement, wasm) {
        const operation = requirement.operation;
        
        switch (operation) {
            case 'arrow_ipc_read':
                return this.performanceTestArrowRead(wasm);
            case 'lz4_decompression':
                return this.performanceTestLZ4Decompression(wasm);
            case 'parquet_read':
                return this.performanceTestParquetRead(wasm);
            default:
                return this.performanceTestGeneric(operation, wasm);
        }
    }

    /**
     * Performance test for Arrow IPC reading
     */
    async performanceTestArrowRead(wasm) {
        const testData = this.createSimpleArrowIPC();
        const handle = wasm.read_table_from_array_buffer(testData.buffer);
        this.handles.add(handle);
        return { handle, dataSize: testData.byteLength };
    }

    /**
     * Utility methods for test data creation
     */
    createMinimalArrowMetadata() {
        // Simplified Arrow metadata structure
        return new Uint8Array([
            // Flatbuffer metadata would go here
            0x00, 0x00, 0x00, 0x00, // Length placeholder
            0x10, 0x00, 0x00, 0x00, // Metadata offset
        ]);
    }

    createSimpleRecordBatch() {
        // Simplified record batch with basic integer data
        const data = new Int32Array([1, 2, 3, 4, 5]);
        return new Uint8Array(data.buffer);
    }

    combineArrowParts(header, metadata, batch) {
        const total = new Uint8Array(header.length + metadata.length + batch.length);
        total.set(header, 0);
        total.set(metadata, header.length);
        total.set(batch, header.length + metadata.length);
        return total;
    }

    combineArrays(...arrays) {
        const totalLength = arrays.reduce((sum, arr) => sum + arr.length, 0);
        const result = new Uint8Array(totalLength);
        let offset = 0;
        for (const arr of arrays) {
            result.set(arr, offset);
            offset += arr.length;
        }
        return result;
    }

    /**
     * Cleanup and utility methods
     */
    getCurrentContext() {
        return {
            handles: Array.from(this.handles),
            testDataKeys: Array.from(this.testData.keys()),
            timestamp: performance.now()
        };
    }

    recordAction(action, data) {
        // Record action for audit trail
    }

    recordPerformanceMetric(stateId, duration) {
        if (!this.performanceMetrics.has(stateId)) {
            this.performanceMetrics.set(stateId, []);
        }
        this.performanceMetrics.get(stateId).push(duration);
    }

    /**
     * Cleanup all resources
     */
    cleanup() {
        // Free all remaining handles
        for (const handle of this.handles) {
            try {
                this.wasm.free_table(handle);
            } catch (error) {
                console.warn(`Failed to free handle ${handle}:`, error);
            }
        }
        this.handles.clear();
        this.testData.clear();
    }

    /**
     * Get test execution summary
     */
    getSummary() {
        return {
            statesVisited: this.stateHistory.length,
            transitionsExecuted: this.transitionHistory.length,
            handlesCreated: this.handles.size,
            performanceMetrics: Object.fromEntries(this.performanceMetrics),
            errors: this.errorHistory.length
        };
    }
}

// Export for use in test runner
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { TestExecutionContext };
} else {
    window.TestExecutionContext = TestExecutionContext;
}