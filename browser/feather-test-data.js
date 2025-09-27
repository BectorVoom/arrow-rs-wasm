/**
 * Test data generator for Feather compression testing
 * Generates various types of test datasets to validate compression and round-trip functionality
 */

export class FeatherTestDataGenerator {
    /**
     * Generate a small mixed-type dataset for basic testing
     */
    static generateSmallMixedData() {
        return [
            { id: 1, name: "Alice", salary: 75000.50, active: true, department: "Engineering" },
            { id: 2, name: "Bob", salary: 82000.75, active: true, department: "Marketing" },
            { id: 3, name: "Charlie", salary: 68000.00, active: false, department: "Engineering" },
            { id: 4, name: "Diana", salary: 91000.25, active: true, department: "Sales" },
            { id: 5, name: "Eve", salary: 77500.80, active: true, department: "Engineering" }
        ];
    }

    /**
     * Generate a larger dataset with repetitive data (good for compression testing)
     */
    static generateRepetitiveData(size = 1000) {
        const departments = ["Engineering", "Marketing", "Sales", "HR", "Finance"];
        const statuses = ["Active", "Inactive", "Pending", "Terminated"];
        const locations = ["New York", "San Francisco", "Chicago", "Austin", "Seattle"];
        
        const data = [];
        for (let i = 1; i <= size; i++) {
            data.push({
                employee_id: i,
                name: `Employee_${i.toString().padStart(4, '0')}`,
                department: departments[i % departments.length],
                status: statuses[i % statuses.length],
                location: locations[i % locations.length],
                salary: Math.floor(50000 + (i % 100) * 1000),
                years_experience: Math.floor(i % 20),
                is_manager: i % 10 === 0,
                hire_date: `2020-${String((i % 12) + 1).padStart(2, '0')}-${String((i % 28) + 1).padStart(2, '0')}`
            });
        }
        return data;
    }

    /**
     * Generate a dataset with many null values
     */
    static generateSparseData(size = 500) {
        const data = [];
        for (let i = 1; i <= size; i++) {
            data.push({
                id: i,
                optional_field_1: i % 5 === 0 ? `value_${i}` : null,
                optional_field_2: i % 7 === 0 ? i * 10.5 : null,
                optional_field_3: i % 3 === 0 ? (i % 2 === 0) : null,
                required_field: `required_${i}`
            });
        }
        return data;
    }

    /**
     * Generate a wide dataset (many columns)
     */
    static generateWideData(rows = 100, columns = 50) {
        const data = [];
        for (let i = 1; i <= rows; i++) {
            const row = { id: i };
            for (let j = 1; j <= columns; j++) {
                const colType = j % 4;
                switch (colType) {
                    case 0: // Integer
                        row[`int_col_${j}`] = i + j;
                        break;
                    case 1: // Float
                        row[`float_col_${j}`] = (i + j) * 1.5;
                        break;
                    case 2: // String
                        row[`str_col_${j}`] = `str_${i}_${j}`;
                        break;
                    case 3: // Boolean
                        row[`bool_col_${j}`] = (i + j) % 2 === 0;
                        break;
                }
            }
            data.push(row);
        }
        return data;
    }

    /**
     * Generate highly compressible data (lots of repetition)
     */
    static generateHighlyCompressibleData(size = 2000) {
        const data = [];
        const repeatValue = "This is a repeated string value that should compress very well";
        const repeatNumber = 42;
        
        for (let i = 1; i <= size; i++) {
            data.push({
                id: i,
                repeated_string: repeatValue,
                repeated_number: repeatNumber,
                repeated_boolean: true,
                sequential_number: Math.floor(i / 100), // Groups of 100 with same value
                category: `Category_${Math.floor(i / 50)}` // Groups of 50 with same category
            });
        }
        return data;
    }

    /**
     * Generate poorly compressible data (random values)
     */
    static generateRandomData(size = 1000) {
        const data = [];
        for (let i = 1; i <= size; i++) {
            data.push({
                id: i,
                random_string: Math.random().toString(36).substring(2, 15),
                random_number: Math.random() * 1000000,
                random_boolean: Math.random() > 0.5,
                uuid: this.generateUUID(),
                timestamp: new Date(Date.now() - Math.random() * 1000000000).toISOString()
            });
        }
        return data;
    }

    /**
     * Generate edge case data
     */
    static generateEdgeCaseData() {
        return [
            // Empty strings
            { id: 1, name: "", value: 0, flag: false },
            // Very long string
            { id: 2, name: "A".repeat(1000), value: Number.MAX_SAFE_INTEGER, flag: true },
            // Special characters
            { id: 3, name: "Special: !@#$%^&*()[]{}|;':\",./<>?", value: -999999, flag: null },
            // Unicode characters
            { id: 4, name: "Unicode: 🚀🌟💎🔥⚡", value: 3.14159, flag: false },
            // Very small numbers
            { id: 5, name: "Small", value: 0.000001, flag: true }
        ];
    }

    /**
     * Generate a single row dataset
     */
    static generateSingleRowData() {
        return [{ id: 1, name: "Single", value: 42, active: true }];
    }

    /**
     * Generate an empty dataset (just schema, no rows)
     */
    static generateEmptyData() {
        return [];
    }

    /**
     * Get all available test datasets
     */
    static getAllTestDatasets() {
        return {
            "Small Mixed Data": this.generateSmallMixedData(),
            "Repetitive Data (1K rows)": this.generateRepetitiveData(1000),
            "Repetitive Data (5K rows)": this.generateRepetitiveData(5000),
            "Sparse Data (nulls)": this.generateSparseData(500),
            "Wide Data (100x50)": this.generateWideData(100, 50),
            "Highly Compressible": this.generateHighlyCompressibleData(2000),
            "Random Data": this.generateRandomData(1000),
            "Edge Cases": this.generateEdgeCaseData(),
            "Single Row": this.generateSingleRowData()
        };
    }

    /**
     * Generate UUID (simple version)
     */
    static generateUUID() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.random() * 16 | 0;
            const v = c === 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    }

    /**
     * Get dataset statistics for compression analysis
     */
    static getDatasetStats(data) {
        if (!data || data.length === 0) {
            return { rows: 0, columns: 0, estimatedSize: 0 };
        }

        const rows = data.length;
        const columns = Object.keys(data[0]).length;
        
        // Rough size estimation (JSON stringified)
        const jsonString = JSON.stringify(data);
        const estimatedSize = new Blob([jsonString]).size;

        return {
            rows,
            columns,
            estimatedSize,
            estimatedSizeKB: Math.round(estimatedSize / 1024 * 100) / 100
        };
    }
}

// For backwards compatibility if imported without ES modules
if (typeof window !== 'undefined') {
    window.FeatherTestDataGenerator = FeatherTestDataGenerator;
}