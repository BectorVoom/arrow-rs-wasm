//! Minimal working examples for PrimitiveArray methods
//! All examples are intentionally simple and educational

use arrow_array::{Array, Int32Array};
use arrow_buffer::{ScalarBuffer, NullBuffer};

/// Construction Methods Examples
pub mod construction {
    use super::*;

    /// Example for new() method
    pub fn example_new() {
        let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
        let array = Int32Array::new(values, None);
        println!("Created array with {} elements", array.len());
    }

    /// Example for new_null() method
    pub fn example_new_null() {
        let null_array = Int32Array::new_null(5);
        assert_eq!(null_array.len(), 5);
        assert!(null_array.is_null(0));
        println!("Created null array with {} elements", null_array.len());
    }

    /// Example for try_new() method
    pub fn example_try_new() {
        let values = ScalarBuffer::from(vec![1, 2, 3]);
        let nulls = NullBuffer::from(vec![true, false, true]);
        
        match Int32Array::try_new(values, Some(nulls)) {
            Ok(array) => println!("Created array with {} elements", array.len()),
            Err(e) => eprintln!("Error creating array: {}", e),
        }
    }

    /// Example for from_iter_values() method
    pub fn example_from_iter_values() {
        let array = Int32Array::from_iter_values(0..5);
        println!("Created array from iterator: {:?}", array.values());
    }

    /// Example for from_value() method
    pub fn example_from_value() {
        let array = Int32Array::from_value(42, 3);
        assert_eq!(array.len(), 3);
        assert_eq!(array.value(0), 42);
        println!("Created array with repeated value: {:?}", array.values());
    }
}

/// Access Methods Examples
pub mod access {
    use super::*;

    /// Example for len() method
    pub fn example_len() {
        let array = Int32Array::from_iter_values(0..10);
        println!("Array length: {}", array.len());
    }

    /// Example for is_empty() method
    pub fn example_is_empty() {
        let empty_array = Int32Array::from_iter_values(std::iter::empty());
        let non_empty_array = Int32Array::from_iter_values(0..3);
        
        println!("Empty array: {}", empty_array.is_empty());
        println!("Non-empty array: {}", non_empty_array.is_empty());
    }

    /// Example for values() method
    pub fn example_values() {
        let array = Int32Array::from_iter_values(1..=5);
        let buffer = array.values();
        println!("Underlying buffer has {} values", buffer.len());
    }

    /// Example for value() method
    pub fn example_value() {
        let array = Int32Array::from_iter_values(10..15);
        println!("Value at index 0: {}", array.value(0));
        println!("Value at index 2: {}", array.value(2));
    }

    /// Example for iter() method
    pub fn example_iter() {
        let array = Int32Array::from_iter_values(1..=3);
        println!("Array values:");
        for (i, value) in array.iter().enumerate() {
            println!("  Index {}: {:?}", i, value);
        }
    }
}

/// Transformation Methods Examples
pub mod transformation {
    use super::*;

    /// Example for slice() method
    pub fn example_slice() {
        let array = Int32Array::from_iter_values(0..10);
        let sliced = array.slice(2, 3);
        
        println!("Original array length: {}", array.len());
        println!("Sliced array length: {}", sliced.len());
        println!("First value in slice: {}", sliced.value(0));
    }

    /// Example for reinterpret_cast() method
    pub fn example_reinterpret_cast() {
        // Create a Date32 array (uses i32 internally)
        use arrow_array::types::Date32Type;
        let date_array: arrow_array::PrimitiveArray<Date32Type> = 
            arrow_array::PrimitiveArray::from_iter_values([19000, 19001, 19002]);
        
        // Reinterpret as Int32 array (same i32 native type)
        let int_array: Int32Array = date_array.reinterpret_cast();
        println!("Date {} reinterpreted as integer: {}", 19000, int_array.value(0));
    }

    /// Example for unary() method
    pub fn example_unary() {
        let array = Int32Array::from_iter_values(1..=5);
        let doubled: Int32Array = array.unary(|x| x * 2);
        
        println!("Original: {:?}", array.values());
        println!("Doubled: {:?}", doubled.values());
    }
}

/// Utility function to run all examples
pub fn run_all_examples() {
    println!("=== Construction Examples ===");
    construction::example_new();
    construction::example_new_null();
    construction::example_try_new();
    construction::example_from_iter_values();
    construction::example_from_value();
    
    println!("\n=== Access Examples ===");
    access::example_len();
    access::example_is_empty();
    access::example_values();
    access::example_value();
    access::example_iter();
    
    println!("\n=== Transformation Examples ===");
    transformation::example_slice();
    transformation::example_reinterpret_cast();
    transformation::example_unary();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_examples() {
        construction::example_new();
        construction::example_new_null();
        construction::example_try_new();
        construction::example_from_iter_values();
        construction::example_from_value();
    }

    #[test]
    fn test_access_examples() {
        access::example_len();
        access::example_is_empty();
        access::example_values();
        access::example_value();
        access::example_iter();
    }

    #[test]
    fn test_transformation_examples() {
        transformation::example_slice();
        transformation::example_reinterpret_cast();
        transformation::example_unary();
    }
}