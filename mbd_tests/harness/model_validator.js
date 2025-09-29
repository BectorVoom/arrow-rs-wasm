#!/usr/bin/env node

/**
 * Model-Based Development - Model Validator
 * 
 * This script validates that all model JSON files conform to the expected schemas
 * and are semantically consistent for test generation.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Define expected schemas for different model types
const MODEL_SCHEMAS = {
  state_machine: {
    required_fields: ['model_id', 'model_type', 'version', 'description', 'states', 'transitions', 'metadata'],
    state_required_fields: ['id', 'name', 'type', 'properties', 'invariants'],
    transition_required_fields: ['id', 'from', 'to', 'trigger', 'guard', 'action', 'requirements'],
    valid_state_types: ['initial', 'normal', 'final', 'error']
  },
  statechart: {
    required_fields: ['model_id', 'model_type', 'version', 'description', 'states', 'transitions', 'metadata'],
    state_required_fields: ['id', 'name', 'type', 'properties', 'invariants'],
    transition_required_fields: ['id', 'from', 'to', 'trigger', 'guard', 'action', 'requirements'],
    valid_state_types: ['initial', 'normal', 'final', 'error', 'composite']
  },
  component: {
    required_fields: ['model_id', 'model_type', 'version', 'description', 'metadata'],
    // Component models have flexible structure based on their specific purpose
  },
  error_model: {
    required_fields: ['model_id', 'model_type', 'version', 'description', 'error_categories', 'metadata'],
    error_category_required_fields: ['id', 'name', 'description', 'error_states'],
    error_state_required_fields: ['error_code', 'severity', 'description', 'causes', 'detection_method', 'user_visible_error', 'recovery_actions', 'test_scenarios']
  },
  error_recovery: {
    required_fields: ['model_id', 'model_type', 'version', 'description', 'recovery_scenarios', 'metadata'],
    recovery_scenario_required_fields: ['scenario_id', 'name', 'description', 'error_triggers', 'recovery_paths'],
    recovery_path_required_fields: ['path_id', 'name', 'description', 'preconditions', 'steps', 'success_criteria', 'failure_handling']
  }
};

class ModelValidator {
  constructor() {
    this.errors = [];
    this.warnings = [];
    this.modelsDir = path.join(__dirname, '..', 'models');
  }

  /**
   * Main validation entry point
   */
  async validateAllModels() {
    console.log('🔍 Starting model validation...\n');

    try {
      // Validate behavioral models
      await this.validateModelCategory('behavioral');
      
      // Validate structural models  
      await this.validateModelCategory('structural');
      
      // Validate error models
      await this.validateModelCategory('error');

      // Report results
      this.reportResults();
      
      return this.errors.length === 0;
    } catch (error) {
      console.error('❌ Validation failed with error:', error.message);
      return false;
    }
  }

  /**
   * Validate all models in a specific category
   */
  async validateModelCategory(category) {
    const categoryDir = path.join(this.modelsDir, category);
    
    if (!fs.existsSync(categoryDir)) {
      this.addError(`Category directory not found: ${categoryDir}`);
      return;
    }

    console.log(`📁 Validating ${category} models...`);

    const files = fs.readdirSync(categoryDir)
      .filter(file => file.endsWith('.json'))
      .sort();

    for (const file of files) {
      const filePath = path.join(categoryDir, file);
      await this.validateModelFile(filePath, category);
    }
  }

  /**
   * Validate a single model file
   */
  async validateModelFile(filePath, category) {
    const fileName = path.basename(filePath);
    console.log(`  📄 Validating ${fileName}...`);

    try {
      // Read and parse JSON
      const content = fs.readFileSync(filePath, 'utf8');
      let model;
      
      try {
        model = JSON.parse(content);
      } catch (parseError) {
        this.addError(`${fileName}: Invalid JSON - ${parseError.message}`);
        return;
      }

      // Schema validation
      this.validateSchema(model, fileName);
      
      // Semantic validation based on model type
      if (model.model_type) {
        this.validateSemantics(model, fileName);
      }

      // Cross-reference validation (if applicable)
      this.validateCrossReferences(model, fileName);

      console.log(`    ✅ ${fileName} validated successfully`);

    } catch (error) {
      this.addError(`${fileName}: Validation error - ${error.message}`);
    }
  }

  /**
   * Validate model against schema
   */
  validateSchema(model, fileName) {
    const modelType = model.model_type;
    const schema = MODEL_SCHEMAS[modelType];

    if (!schema) {
      this.addWarning(`${fileName}: Unknown model type '${modelType}'`);
      return;
    }

    // Check required top-level fields
    for (const field of schema.required_fields) {
      if (!(field in model)) {
        this.addError(`${fileName}: Missing required field '${field}'`);
      }
    }

    // Validate specific structures based on model type
    if (modelType === 'state_machine' || modelType === 'statechart') {
      this.validateStateMachineSchema(model, fileName, schema);
    } else if (modelType === 'error_model') {
      this.validateErrorModelSchema(model, fileName, schema);
    } else if (modelType === 'error_recovery') {
      this.validateErrorRecoverySchema(model, fileName, schema);
    }
  }

  /**
   * Validate state machine schema
   */
  validateStateMachineSchema(model, fileName, schema) {
    // Validate states
    if (model.states && Array.isArray(model.states)) {
      for (const [index, state] of model.states.entries()) {
        for (const field of schema.state_required_fields) {
          if (!(field in state)) {
            this.addError(`${fileName}: State ${index} missing required field '${field}'`);
          }
        }
        
        if (state.type && !schema.valid_state_types.includes(state.type)) {
          this.addError(`${fileName}: State '${state.id}' has invalid type '${state.type}'`);
        }
      }
    } else {
      this.addError(`${fileName}: 'states' must be an array`);
    }

    // Validate transitions
    if (model.transitions && Array.isArray(model.transitions)) {
      for (const [index, transition] of model.transitions.entries()) {
        for (const field of schema.transition_required_fields) {
          if (!(field in transition)) {
            this.addError(`${fileName}: Transition ${index} missing required field '${field}'`);
          }
        }
      }
    } else {
      this.addError(`${fileName}: 'transitions' must be an array`);
    }
  }

  /**
   * Validate error model schema
   */
  validateErrorModelSchema(model, fileName, schema) {
    if (model.error_categories && Array.isArray(model.error_categories)) {
      for (const category of model.error_categories) {
        for (const field of schema.error_category_required_fields) {
          if (!(field in category)) {
            this.addError(`${fileName}: Error category '${category.id || 'unknown'}' missing field '${field}'`);
          }
        }

        if (category.error_states && Array.isArray(category.error_states)) {
          for (const errorState of category.error_states) {
            for (const field of schema.error_state_required_fields) {
              if (!(field in errorState)) {
                this.addError(`${fileName}: Error state '${errorState.error_code || 'unknown'}' missing field '${field}'`);
              }
            }
          }
        }
      }
    }
  }

  /**
   * Validate error recovery schema
   */
  validateErrorRecoverySchema(model, fileName, schema) {
    if (model.recovery_scenarios && Array.isArray(model.recovery_scenarios)) {
      for (const scenario of model.recovery_scenarios) {
        for (const field of schema.recovery_scenario_required_fields) {
          if (!(field in scenario)) {
            this.addError(`${fileName}: Recovery scenario '${scenario.scenario_id || 'unknown'}' missing field '${field}'`);
          }
        }

        if (scenario.recovery_paths && Array.isArray(scenario.recovery_paths)) {
          for (const path of scenario.recovery_paths) {
            for (const field of schema.recovery_path_required_fields) {
              if (!(field in path)) {
                this.addError(`${fileName}: Recovery path '${path.path_id || 'unknown'}' missing field '${field}'`);
              }
            }
          }
        }
      }
    }
  }

  /**
   * Validate semantic consistency
   */
  validateSemantics(model, fileName) {
    if (model.model_type === 'state_machine' || model.model_type === 'statechart') {
      this.validateStateMachineSemantics(model, fileName);
    }
  }

  /**
   * Validate state machine semantic consistency
   */
  validateStateMachineSemantics(model, fileName) {
    if (!model.states || !model.transitions) return;

    const stateIds = new Set(model.states.map(s => s.id));
    const initialStates = model.states.filter(s => s.type === 'initial');

    // Check for exactly one initial state
    if (initialStates.length === 0) {
      this.addError(`${fileName}: State machine must have exactly one initial state`);
    } else if (initialStates.length > 1) {
      this.addError(`${fileName}: State machine has multiple initial states: ${initialStates.map(s => s.id).join(', ')}`);
    }

    // Check transition references
    for (const transition of model.transitions) {
      if (!stateIds.has(transition.from)) {
        this.addError(`${fileName}: Transition '${transition.id}' references unknown 'from' state '${transition.from}'`);
      }
      if (!stateIds.has(transition.to)) {
        this.addError(`${fileName}: Transition '${transition.id}' references unknown 'to' state '${transition.to}'`);
      }
    }

    // Check for unreachable states (warning only)
    const reachableStates = new Set();
    const addReachable = (stateId) => {
      if (!reachableStates.has(stateId)) {
        reachableStates.add(stateId);
        const outgoingTransitions = model.transitions.filter(t => t.from === stateId);
        for (const transition of outgoingTransitions) {
          addReachable(transition.to);
        }
      }
    };

    if (initialStates.length === 1) {
      addReachable(initialStates[0].id);
    }

    for (const stateId of stateIds) {
      if (!reachableStates.has(stateId)) {
        this.addWarning(`${fileName}: State '${stateId}' appears to be unreachable`);
      }
    }
  }

  /**
   * Validate cross-references between models
   */
  validateCrossReferences(model, fileName) {
    // Check requirement references are consistent
    if (model.metadata && model.metadata.requirements) {
      const reqPattern = /REQ-\d+/g;
      for (const req of model.metadata.requirements) {
        if (!reqPattern.test(req)) {
          this.addWarning(`${fileName}: Requirement '${req}' doesn't follow REQ-XXX format`);
        }
      }
    }

    // Check transitions reference requirements
    if (model.transitions) {
      for (const transition of model.transitions) {
        if (transition.requirements && Array.isArray(transition.requirements)) {
          for (const req of transition.requirements) {
            if (!/REQ-\d+/.test(req)) {
              this.addWarning(`${fileName}: Transition '${transition.id}' requirement '${req}' doesn't follow REQ-XXX format`);
            }
          }
        }
      }
    }
  }

  /**
   * Add validation error
   */
  addError(message) {
    this.errors.push(message);
    console.log(`    ❌ ERROR: ${message}`);
  }

  /**
   * Add validation warning
   */
  addWarning(message) {
    this.warnings.push(message);
    console.log(`    ⚠️  WARNING: ${message}`);
  }

  /**
   * Report validation results
   */
  reportResults() {
    console.log('\n📊 Validation Results:');
    console.log(`✅ Total errors: ${this.errors.length}`);
    console.log(`⚠️  Total warnings: ${this.warnings.length}`);

    if (this.errors.length === 0) {
      console.log('\n🎉 All models validated successfully!');
    } else {
      console.log('\n❌ Validation failed. Please fix the errors above.');
    }

    if (this.warnings.length > 0) {
      console.log('\n⚠️  Warnings (recommended to fix):');
      for (const warning of this.warnings) {
        console.log(`  - ${warning}`);
      }
    }
  }
}

// Main execution
if (import.meta.url === `file://${process.argv[1]}`) {
  const validator = new ModelValidator();
  const success = await validator.validateAllModels();
  process.exit(success ? 0 : 1);
}

export { ModelValidator };