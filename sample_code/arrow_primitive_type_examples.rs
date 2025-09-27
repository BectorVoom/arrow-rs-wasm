//! Minimal working examples for ArrowPrimitiveType trait usage
//! Demonstrates type system integration and generic programming

use arrow_array::{Array, PrimitiveArray};
use arrow_array::types::*;

/// Examples demonstrating ArrowPrimitiveType trait usage
pub mod trait_usage {
    use super::*;

    /// Example showing basic trait constraint usage
    pub fn example_generic_function<T: ArrowPrimitiveType>() 
    where
        T::Native: std::fmt::Debug + Default + Copy,
    {
        let default_val = T::default_value();
        let data_type = T::DATA_TYPE;
        println!("Type: {:?}, Default: {:?}", data_type, default_val);
    }

    /// Example showing type-specific array creation
    pub fn example_create_typed_arrays() {
        // Integer array
        let int_array: PrimitiveArray<Int32Type> = PrimitiveArray::from_iter_values(1..=5);
        println!("Int32 array: {:?}", int_array.data_type());
        
        // Float array
        let float_array: PrimitiveArray<Float64Type> = PrimitiveArray::from_iter_values([1.1, 2.2, 3.3]);
        println!("Float64 array: {:?}", float_array.data_type());
        
        // Date array
        let date_array: PrimitiveArray<Date32Type> = PrimitiveArray::from_iter_values([19000, 19001, 19002]);
        println!("Date32 array: {:?}", date_array.data_type());
    }

    /// Example showing runtime type checking
    pub fn example_runtime_type_info() {
        // Check data types at runtime
        println!("Int8 type: {:?}", Int8Type::DATA_TYPE);
        println!("Int16 type: {:?}", Int16Type::DATA_TYPE);
        println!("Int32 type: {:?}", Int32Type::DATA_TYPE);
        println!("Int64 type: {:?}", Int64Type::DATA_TYPE);
        
        println!("Float32 type: {:?}", Float32Type::DATA_TYPE);
        println!("Float64 type: {:?}", Float64Type::DATA_TYPE);
    }

    /// Example showing default values for different types
    pub fn example_default_values() {
        println!("Default values:");
        println!("  Int32: {}", Int32Type::default_value());
        println!("  Float64: {}", Float64Type::default_value());
        println!("  Date32: {}", Date32Type::default_value());
        println!("  UInt64: {}", UInt64Type::default_value());
    }
}

/// Examples for specific primitive type implementations
pub mod type_implementations {
    use super::*;

    /// Integer types examples
    pub fn example_integer_types() {
        // Signed integers
        let i8_array: PrimitiveArray<Int8Type> = PrimitiveArray::from_iter_values([1i8, 2, 3]);
        let i16_array: PrimitiveArray<Int16Type> = PrimitiveArray::from_iter_values([100i16, 200, 300]);
        let i32_array: PrimitiveArray<Int32Type> = PrimitiveArray::from_iter_values([1000i32, 2000, 3000]);
        let i64_array: PrimitiveArray<Int64Type> = PrimitiveArray::from_iter_values([100000i64, 200000, 300000]);
        
        println!("Integer arrays created with {} {} {} {} elements", 
                 i8_array.len(), i16_array.len(), i32_array.len(), i64_array.len());

        // Unsigned integers
        let u8_array: PrimitiveArray<UInt8Type> = PrimitiveArray::from_iter_values([1u8, 2, 3]);
        let u16_array: PrimitiveArray<UInt16Type> = PrimitiveArray::from_iter_values([100u16, 200, 300]);
        let u32_array: PrimitiveArray<UInt32Type> = PrimitiveArray::from_iter_values([1000u32, 2000, 3000]);
        let u64_array: PrimitiveArray<UInt64Type> = PrimitiveArray::from_iter_values([100000u64, 200000, 300000]);
        
        println!("Unsigned integer arrays created with {} {} {} {} elements", 
                 u8_array.len(), u16_array.len(), u32_array.len(), u64_array.len());
    }

    /// Floating point types examples
    pub fn example_float_types() {
        let f32_array: PrimitiveArray<Float32Type> = PrimitiveArray::from_iter_values([1.1f32, 2.2, 3.3]);
        let f64_array: PrimitiveArray<Float64Type> = PrimitiveArray::from_iter_values([1.111f64, 2.222, 3.333]);
        
        println!("Float arrays created with {} and {} elements", f32_array.len(), f64_array.len());
        println!("F32 precision: single, F64 precision: double");
    }

    /// Temporal types examples
    pub fn example_temporal_types() {
        // Date types (days since epoch)
        let date32_array: PrimitiveArray<Date32Type> = PrimitiveArray::from_iter_values([19000, 19001, 19002]);
        let date64_array: PrimitiveArray<Date64Type> = PrimitiveArray::from_iter_values([1640995200000i64, 1641081600000, 1641168000000]);
        
        println!("Date arrays: Date32 ({} elements), Date64 ({} elements)", 
                 date32_array.len(), date64_array.len());

        // Time types (time since midnight)
        let time32_sec: PrimitiveArray<Time32SecondType> = PrimitiveArray::from_iter_values([3600, 7200, 10800]);
        let time64_nano: PrimitiveArray<Time64NanosecondType> = PrimitiveArray::from_iter_values([3600000000000i64, 7200000000000, 10800000000000]);
        
        println!("Time arrays: Time32Sec ({} elements), Time64Nano ({} elements)", 
                 time32_sec.len(), time64_nano.len());
    }
}

/// Generic programming examples using ArrowPrimitiveType
pub mod generic_programming {
    use super::*;

    /// Generic function that works with any primitive array
    pub fn process_primitive_array<T: ArrowPrimitiveType>(array: &PrimitiveArray<T>) 
    where
        T::Native: std::fmt::Debug,
    {
        println!("Processing array:");
        println!("  Type: {:?}", array.data_type());
        println!("  Length: {}", array.len());
        println!("  Null count: {}", array.null_count());
        
        if !array.is_empty() {
            println!("  First value: {:?}", array.value(0));
        }
    }

    /// Example showing type-agnostic array processing
    pub fn example_generic_processing() {
        let int_array = PrimitiveArray::<Int32Type>::from_iter_values(1..=3);
        let float_array = PrimitiveArray::<Float64Type>::from_iter_values([1.1, 2.2, 3.3]);
        
        process_primitive_array(&int_array);
        process_primitive_array(&float_array);
    }

    /// Generic function for array statistics
    pub fn array_stats<T: ArrowPrimitiveType>(array: &PrimitiveArray<T>) -> (usize, usize)
    where
        T::Native: PartialOrd + Copy,
    {
        let len = array.len();
        let null_count = array.null_count();
        (len, null_count)
    }

    /// Example showing statistical operations
    pub fn example_array_statistics() {
        let array_with_nulls = {
            use arrow_buffer::{ScalarBuffer, NullBuffer};
            let values = ScalarBuffer::from(vec![1, 2, 3, 4, 5]);
            let nulls = NullBuffer::from(vec![true, false, true, true, false]);
            PrimitiveArray::<Int32Type>::new(values, Some(nulls))
        };
        
        let (len, null_count) = array_stats(&array_with_nulls);
        println!("Array statistics: {} total, {} nulls, {} valid", len, null_count, len - null_count);
    }
}

/// Utility function to run all examples
pub fn run_all_examples() {
    println!("=== ArrowPrimitiveType Trait Usage ===");
    trait_usage::example_generic_function::<Int32Type>();
    trait_usage::example_generic_function::<Float64Type>();
    trait_usage::example_create_typed_arrays();
    trait_usage::example_runtime_type_info();
    trait_usage::example_default_values();
    
    println!("\n=== Type Implementations ===");
    type_implementations::example_integer_types();
    type_implementations::example_float_types();
    type_implementations::example_temporal_types();
    
    println!("\n=== Generic Programming ===");
    generic_programming::example_generic_processing();
    generic_programming::example_array_statistics();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_usage() {
        trait_usage::example_create_typed_arrays();
        trait_usage::example_runtime_type_info();
        trait_usage::example_default_values();
    }

    #[test]
    fn test_type_implementations() {
        type_implementations::example_integer_types();
        type_implementations::example_float_types();
        type_implementations::example_temporal_types();
    }

    #[test]
    fn test_generic_programming() {
        generic_programming::example_generic_processing();
        generic_programming::example_array_statistics();
    }
}