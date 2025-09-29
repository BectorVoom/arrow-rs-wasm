
use std::sync::Arc;
use arrow_array::{ArrayRef, Int32Array, StringArray, RecordBatch};
use arrow_schema::{DataType, Field, Schema};
use arrow_ipc::writer::FileWriter;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create schema
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("value", DataType::Int32, true),
    ]);

    // Create arrays
    let id_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
    let name_array = StringArray::from(vec!["Alice", "Bob", "Charlie", "Diana", "Eve"]);
    let value_array = Int32Array::from(vec![Some(100), None, Some(300), Some(400), None]);

    // Create record batch
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(id_array) as ArrayRef,
            Arc::new(name_array) as ArrayRef,
            Arc::new(value_array) as ArrayRef,
        ],
    )?;

    // Write to file
    let file = File::create("test_data.arrow")?;
    let mut writer = FileWriter::try_new(file, &batch.schema())?;
    writer.write(&batch)?;
    writer.finish()?;

    println!("✅ Test Arrow file created: test_data.arrow");
    Ok(())
}
