//! Minimal working examples for supporting structures
//! Covers ScalarBuffer, NullBuffer, and DataType usage

use arrow_array::{Array, Int32Array};
use arrow_buffer::{ScalarBuffer, NullBuffer};
use arrow_schema::DataType;

/// ScalarBuffer examples
pub mod scalar_buffer {
    use super::*;

    /// Example creating ScalarBuffer from Vec
    pub fn example_from_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let buffer: ScalarBuffer<i32> = data.into();
        println!("ScalarBuffer created with {} elements", buffer.len());
    }

    /// Example accessing ScalarBuffer data
    pub fn example_buffer_access() {
        let buffer = ScalarBuffer::from(vec![10, 20, 30]);
        println!("Buffer length: {}", buffer.len());
        println!("Buffer is empty: {}", buffer.is_empty());
        
        // Access via indexing
        for i in 0..buffer.len() {
            println!("  Element {}: {}", i, buffer[i]);
        }
    }

    /// Example using ScalarBuffer with PrimitiveArray
    pub fn example_with_primitive_array() {
        let values = ScalarBuffer::from(vec![100, 200, 300, 400]);
        let array = Int32Array::new(values, None);
        
        println!("Created array from ScalarBuffer:");
        println!("  Array length: {}", array.len());
        println!("  Values: {:?}", array.values());
    }

    /// Example showing zero-copy construction
    pub fn example_zero_copy() {
        let original_vec = vec![1, 2, 3, 4, 5];
        println!("Original vector length: {}", original_vec.len());
        
        // Convert to ScalarBuffer (may be zero-copy)
        let buffer = ScalarBuffer::from(original_vec);
        println!("ScalarBuffer length: {}", buffer.len());
        
        // Use in array construction
        let array = Int32Array::new(buffer, None);
        println!("Array length: {}", array.len());
    }
}

/// NullBuffer examples
pub mod null_buffer {
    use super::*;

    /// Example creating NullBuffer from boolean vector
    pub fn example_from_bool_vec() {
        let null_mask = vec![true, false, true, false, true]; // true = valid, false = null
        let null_buffer = NullBuffer::from(null_mask);
        println!("NullBuffer created with {} bits", null_buffer.len());
    }

    /// Example using NullBuffer with PrimitiveArray
    pub fn example_with_array() {
        let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
        let nulls = NullBuffer::from(vec![true, false, true, false, true]);
        
        let array = Int32Array::new(values, Some(nulls));
        
        println!("Array with nulls:");
        for i in 0..array.len() {
            if array.is_null(i) {
                println!("  Index {}: NULL", i);
            } else {
                println!("  Index {}: {}", i, array.value(i));
            }
        }
    }

    /// Example showing null count functionality
    pub fn example_null_count() {
        let null_mask = vec![true, true, false, false, true]; // 3 valid, 2 null
        let null_buffer = NullBuffer::from(null_mask);
        
        println!("Total length: {}", null_buffer.len());
        println!("Null count: {}", null_buffer.null_count());
    }

    /// Example creating array without nulls vs with nulls
    pub fn example_with_without_nulls() {
        let values = ScalarBuffer::from(vec![1, 2, 3]);
        
        // Array without nulls
        let array_no_nulls = Int32Array::new(values.clone(), None);
        println!("Array without nulls - null count: {}", array_no_nulls.null_count());
        
        // Array with nulls
        let nulls = NullBuffer::from(vec![true, false, true]);
        let array_with_nulls = Int32Array::new(values, Some(nulls));
        println!("Array with nulls - null count: {}", array_with_nulls.null_count());
    }
}

/// DataType examples
pub mod data_type {
    use super::*;

    /// Example showing DataType variants for primitives
    pub fn example_primitive_data_types() {
        println!("Primitive DataType variants:");
        println!("  Int8: {:?}", DataType::Int8);
        println!("  Int16: {:?}", DataType::Int16);
        println!("  Int32: {:?}", DataType::Int32);
        println!("  Int64: {:?}", DataType::Int64);
        println!("  Float32: {:?}", DataType::Float32);
        println!("  Float64: {:?}", DataType::Float64);
    }

    /// Example showing temporal DataTypes
    pub fn example_temporal_data_types() {
        use arrow_schema::{TimeUnit, IntervalUnit};
        
        println!("Temporal DataType variants:");
        println!("  Date32: {:?}", DataType::Date32);
        println!("  Date64: {:?}", DataType::Date64);
        println!("  Time32(Second): {:?}", DataType::Time32(TimeUnit::Second));
        println!("  Time64(Nanosecond): {:?}", DataType::Time64(TimeUnit::Nanosecond));
        println!("  Timestamp(Millisecond): {:?}", DataType::Timestamp(TimeUnit::Millisecond, None));
        println!("  Duration(Microsecond): {:?}", DataType::Duration(TimeUnit::Microsecond));
        println!("  Interval(YearMonth): {:?}", DataType::Interval(IntervalUnit::YearMonth));
    }

    /// Example getting DataType from array
    pub fn example_array_data_type() {
        let array = Int32Array::from_iter_values(1..=5);
        let data_type = array.data_type();
        
        println!("Array data type: {:?}", data_type);
        
        // Check if it's a specific type
        match data_type {
            DataType::Int32 => println!("This is an Int32 array"),
            DataType::Float64 => println!("This is a Float64 array"),
            _ => println!("This is some other type"),
        }
    }

    /// Example showing type checking
    pub fn example_type_checking() {
        let int_array = Int32Array::from_iter_values(1..=3);
        
        // Type checking examples
        println!("Type checks:");
        println!("  Is Int32: {}", matches!(int_array.data_type(), DataType::Int32));
        println!("  Is Float64: {}", matches!(int_array.data_type(), DataType::Float64));
        println!("  Is primitive: {}", int_array.data_type().is_primitive());
    }
}

/// Integration examples showing how structures work together
pub mod integration {
    use super::*;

    /// Example showing complete array construction workflow
    pub fn example_complete_construction() {
        println!("Complete array construction workflow:");
        
        // Step 1: Create values buffer
        let values = ScalarBuffer::from(vec![10, 20, 30, 40, 50]);
        println!("1. Created ScalarBuffer with {} values", values.len());
        
        // Step 2: Create null buffer
        let nulls = NullBuffer::from(vec![true, false, true, false, true]);
        println!("2. Created NullBuffer with {} valid values", nulls.len() - nulls.null_count());
        
        // Step 3: Construct array
        let array = Int32Array::new(values, Some(nulls));
        println!("3. Constructed PrimitiveArray");
        
        // Step 4: Verify data type
        println!("4. Array data type: {:?}", array.data_type());
        
        // Step 5: Access data
        println!("5. Array contents:");
        for i in 0..array.len() {
            if array.is_null(i) {
                println!("   [{}]: NULL", i);
            } else {
                println!("   [{}]: {}", i, array.value(i));
            }
        }
    }

    /// Example showing buffer reuse
    pub fn example_buffer_reuse() {
        let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
        
        // Create multiple arrays with the same values buffer
        let array1 = Int32Array::new(values.clone(), None);
        let array2 = Int32Array::new(values.clone(), 
            Some(NullBuffer::from(vec![true, true, false, true, true])));
        
        println!("Reused buffer in {} arrays", 2);
        println!("Array 1 null count: {}", array1.null_count());
        println!("Array 2 null count: {}", array2.null_count());
    }

    /// Example showing memory efficiency
    pub fn example_memory_efficiency() {
        // Large array to demonstrate efficiency
        let size = 1000;
        let values: Vec<i32> = (0..size).collect();
        
        println!("Creating array with {} elements", size);
        
        // Convert to ScalarBuffer (potentially zero-copy)
        let buffer = ScalarBuffer::from(values);
        let array = Int32Array::new(buffer, None);
        
        println!("Array created successfully");
        println!("  Length: {}", array.len());
        println!("  Data type: {:?}", array.data_type());
        println!("  Memory layout: columnar");
    }
}

/// Utility function to run all examples
pub fn run_all_examples() {
    println!("=== ScalarBuffer Examples ===");
    scalar_buffer::example_from_vec();
    scalar_buffer::example_buffer_access();
    scalar_buffer::example_with_primitive_array();
    scalar_buffer::example_zero_copy();
    
    println!("\n=== NullBuffer Examples ===");
    null_buffer::example_from_bool_vec();
    null_buffer::example_with_array();
    null_buffer::example_null_count();
    null_buffer::example_with_without_nulls();
    
    println!("\n=== DataType Examples ===");
    data_type::example_primitive_data_types();
    data_type::example_temporal_data_types();
    data_type::example_array_data_type();
    data_type::example_type_checking();
    
    println!("\n=== Integration Examples ===");
    integration::example_complete_construction();
    integration::example_buffer_reuse();
    integration::example_memory_efficiency();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_buffer() {
        scalar_buffer::example_from_vec();
        scalar_buffer::example_buffer_access();
        scalar_buffer::example_with_primitive_array();
        scalar_buffer::example_zero_copy();
    }

    #[test]
    fn test_null_buffer() {
        null_buffer::example_from_bool_vec();
        null_buffer::example_with_array();
        null_buffer::example_null_count();
        null_buffer::example_with_without_nulls();
    }

    #[test]
    fn test_data_type() {
        data_type::example_primitive_data_types();
        data_type::example_temporal_data_types();
        data_type::example_array_data_type();
        data_type::example_type_checking();
    }

    #[test]
    fn test_integration() {
        integration::example_complete_construction();
        integration::example_buffer_reuse();
        integration::example_memory_efficiency();
    }
}