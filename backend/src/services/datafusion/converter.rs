// DataFusion ResultConverter
//
// Converts DataFusion RecordBatch results to JSON format compatible with
// the existing QueryResult model.

use datafusion::arrow::array::*;
use datafusion::arrow::datatypes::{DataType, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use serde_json::{json, Value as JsonValue, Map};
use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

use crate::models::query::QueryResult;

/// Converts DataFusion query results to JSON format
///
/// The ResultConverter handles type conversions from Arrow data types
/// to JSON, maintaining compatibility with the existing QueryResult model.
pub struct DataFusionResultConverter;

impl DataFusionResultConverter {
    /// Convert query execution results to QueryResult model
    ///
    /// # Arguments
    /// * `schema` - Arrow schema defining column types
    /// * `batches` - Record batches containing query results
    ///
    /// # Returns
    /// QueryResult compatible with existing API format
    pub fn convert_to_query_result(
        schema: SchemaRef,
        batches: Vec<RecordBatch>,
    ) -> Result<QueryResult> {
        // Extract column names from schema
        let columns: Vec<String> = schema
            .fields()
            .iter()
            .map(|field| field.name().clone())
            .collect();

        // Convert all batches to rows
        let mut all_rows = Vec::new();
        for batch in &batches {
            let rows = Self::batch_to_json_rows(&schema, batch)?;
            all_rows.extend(rows);
        }

        Ok(QueryResult {
            columns,
            rows: all_rows,
            row_count: all_rows.len(),
        })
    }

    /// Convert a single RecordBatch to JSON rows
    ///
    /// # Arguments
    /// * `schema` - Arrow schema
    /// * `batch` - Record batch to convert
    ///
    /// # Returns
    /// Vector of JSON objects, one per row
    fn batch_to_json_rows(schema: &Schema, batch: &RecordBatch) -> Result<Vec<Vec<JsonValue>>> {
        let num_rows = batch.num_rows();
        let num_cols = batch.num_columns();

        let mut rows = Vec::with_capacity(num_rows);

        for row_idx in 0..num_rows {
            let mut row = Vec::with_capacity(num_cols);

            for col_idx in 0..num_cols {
                let column = batch.column(col_idx);
                let field = schema.field(col_idx);

                let value = Self::array_value_to_json(column, row_idx, field.data_type())?;
                row.push(value);
            }

            rows.push(row);
        }

        Ok(rows)
    }

    /// Convert a single array value to JSON
    ///
    /// # Arguments
    /// * `array` - Arrow array containing the data
    /// * `row_idx` - Row index to extract
    /// * `data_type` - Data type of the column
    ///
    /// # Returns
    /// JSON representation of the value
    fn array_value_to_json(
        array: &ArrayRef,
        row_idx: usize,
        data_type: &DataType,
    ) -> Result<JsonValue> {
        // Handle NULL values
        if array.is_null(row_idx) {
            return Ok(JsonValue::Null);
        }

        let value = match data_type {
            // Boolean
            DataType::Boolean => {
                let array = array.as_any().downcast_ref::<BooleanArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to BooleanArray"))?;
                json!(array.value(row_idx))
            }

            // Integer types
            DataType::Int8 => {
                let array = array.as_any().downcast_ref::<Int8Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Int8Array"))?;
                json!(array.value(row_idx))
            }
            DataType::Int16 => {
                let array = array.as_any().downcast_ref::<Int16Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Int16Array"))?;
                json!(array.value(row_idx))
            }
            DataType::Int32 => {
                let array = array.as_any().downcast_ref::<Int32Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Int32Array"))?;
                json!(array.value(row_idx))
            }
            DataType::Int64 => {
                let array = array.as_any().downcast_ref::<Int64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Int64Array"))?;
                json!(array.value(row_idx))
            }

            // Unsigned integer types
            DataType::UInt8 => {
                let array = array.as_any().downcast_ref::<UInt8Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to UInt8Array"))?;
                json!(array.value(row_idx))
            }
            DataType::UInt16 => {
                let array = array.as_any().downcast_ref::<UInt16Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to UInt16Array"))?;
                json!(array.value(row_idx))
            }
            DataType::UInt32 => {
                let array = array.as_any().downcast_ref::<UInt32Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to UInt32Array"))?;
                json!(array.value(row_idx))
            }
            DataType::UInt64 => {
                let array = array.as_any().downcast_ref::<UInt64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to UInt64Array"))?;
                json!(array.value(row_idx))
            }

            // Floating point types
            DataType::Float32 => {
                let array = array.as_any().downcast_ref::<Float32Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Float32Array"))?;
                json!(array.value(row_idx))
            }
            DataType::Float64 => {
                let array = array.as_any().downcast_ref::<Float64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Float64Array"))?;
                json!(array.value(row_idx))
            }

            // Decimal types
            DataType::Decimal128(_, scale) => {
                let array = array.as_any().downcast_ref::<Decimal128Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Decimal128Array"))?;
                let value = array.value(row_idx);
                let scale = *scale as u32;
                let divisor = 10_i128.pow(scale);
                let decimal_value = value as f64 / divisor as f64;
                json!(decimal_value)
            }

            // String types
            DataType::Utf8 => {
                let array = array.as_any().downcast_ref::<StringArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to StringArray"))?;
                json!(array.value(row_idx))
            }
            DataType::LargeUtf8 => {
                let array = array.as_any().downcast_ref::<LargeStringArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to LargeStringArray"))?;
                json!(array.value(row_idx))
            }

            // Binary types
            DataType::Binary => {
                let array = array.as_any().downcast_ref::<BinaryArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to BinaryArray"))?;
                let bytes = array.value(row_idx);
                // Convert to hex string for JSON representation
                let hex_string = bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                json!(hex_string)
            }
            DataType::LargeBinary => {
                let array = array.as_any().downcast_ref::<LargeBinaryArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to LargeBinaryArray"))?;
                let bytes = array.value(row_idx);
                let hex_string = bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                json!(hex_string)
            }

            // Date types
            DataType::Date32 => {
                let array = array.as_any().downcast_ref::<Date32Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Date32Array"))?;
                let days = array.value(row_idx);
                // Date32 is days since Unix epoch
                let date = NaiveDate::from_num_days_from_ce_opt(days + 719_163) // Adjust for epoch
                    .ok_or_else(|| anyhow!("Invalid date value"))?;
                json!(date.format("%Y-%m-%d").to_string())
            }
            DataType::Date64 => {
                let array = array.as_any().downcast_ref::<Date64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Date64Array"))?;
                let millis = array.value(row_idx);
                let datetime = DateTime::from_timestamp_millis(millis)
                    .ok_or_else(|| anyhow!("Invalid timestamp value"))?;
                json!(datetime.format("%Y-%m-%d").to_string())
            }

            // Timestamp types
            DataType::Timestamp(unit, tz) => {
                let timestamp = match unit {
                    datafusion::arrow::datatypes::TimeUnit::Second => {
                        let array = array.as_any().downcast_ref::<TimestampSecondArray>()
                            .ok_or_else(|| anyhow!("Failed to downcast to TimestampSecondArray"))?;
                        DateTime::from_timestamp(array.value(row_idx), 0)
                    }
                    datafusion::arrow::datatypes::TimeUnit::Millisecond => {
                        let array = array.as_any().downcast_ref::<TimestampMillisecondArray>()
                            .ok_or_else(|| anyhow!("Failed to downcast to TimestampMillisecondArray"))?;
                        DateTime::from_timestamp_millis(array.value(row_idx))
                    }
                    datafusion::arrow::datatypes::TimeUnit::Microsecond => {
                        let array = array.as_any().downcast_ref::<TimestampMicrosecondArray>()
                            .ok_or_else(|| anyhow!("Failed to downcast to TimestampMicrosecondArray"))?;
                        DateTime::from_timestamp_micros(array.value(row_idx))
                    }
                    datafusion::arrow::datatypes::TimeUnit::Nanosecond => {
                        let array = array.as_any().downcast_ref::<TimestampNanosecondArray>()
                            .ok_or_else(|| anyhow!("Failed to downcast to TimestampNanosecondArray"))?;
                        DateTime::from_timestamp_nanos(array.value(row_idx))
                    }
                };

                let dt = timestamp.ok_or_else(|| anyhow!("Invalid timestamp value"))?;
                json!(dt.to_rfc3339())
            }

            // Time types
            DataType::Time64(unit) => {
                match unit {
                    datafusion::arrow::datatypes::TimeUnit::Microsecond => {
                        let array = array.as_any().downcast_ref::<Time64MicrosecondArray>()
                            .ok_or_else(|| anyhow!("Failed to downcast to Time64MicrosecondArray"))?;
                        let micros = array.value(row_idx);
                        // Convert to HH:MM:SS.ffffff format
                        let seconds = micros / 1_000_000;
                        let remaining_micros = micros % 1_000_000;
                        let hours = seconds / 3600;
                        let minutes = (seconds % 3600) / 60;
                        let secs = seconds % 60;
                        json!(format!("{:02}:{:02}:{:02}.{:06}", hours, minutes, secs, remaining_micros))
                    }
                    datafusion::arrow::datatypes::TimeUnit::Nanosecond => {
                        let array = array.as_any().downcast_ref::<Time64NanosecondArray>()
                            .ok_or_else(|| anyhow!("Failed to downcast to Time64NanosecondArray"))?;
                        let nanos = array.value(row_idx);
                        let seconds = nanos / 1_000_000_000;
                        let remaining_nanos = nanos % 1_000_000_000;
                        let hours = seconds / 3600;
                        let minutes = (seconds % 3600) / 60;
                        let secs = seconds % 60;
                        json!(format!("{:02}:{:02}:{:02}.{:09}", hours, minutes, secs, remaining_nanos))
                    }
                    _ => json!(null),
                }
            }

            // List types (arrays)
            DataType::List(_) => {
                json!("LIST_NOT_YET_SUPPORTED")
            }

            // Struct types
            DataType::Struct(_) => {
                json!("STRUCT_NOT_YET_SUPPORTED")
            }

            // Default for unsupported types
            _ => {
                tracing::warn!("Unsupported Arrow data type: {:?}", data_type);
                json!(format!("UNSUPPORTED_TYPE_{:?}", data_type))
            }
        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use datafusion::arrow::datatypes::{Field, Schema};

    #[test]
    fn test_convert_simple_batch() {
        // Create a simple schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
        ]));

        // Create data
        let id_array = Int32Array::from(vec![1, 2, 3]);
        let name_array = StringArray::from(vec![Some("Alice"), Some("Bob"), None]);

        // Create record batch
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(id_array), Arc::new(name_array)],
        ).unwrap();

        // Convert to QueryResult
        let result = DataFusionResultConverter::convert_to_query_result(schema, vec![batch]);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.columns, vec!["id", "name"]);
        assert_eq!(result.row_count, 3);
        assert_eq!(result.rows.len(), 3);

        // Check first row
        assert_eq!(result.rows[0][0], json!(1));
        assert_eq!(result.rows[0][1], json!("Alice"));

        // Check third row (with NULL)
        assert_eq!(result.rows[2][0], json!(3));
        assert_eq!(result.rows[2][1], JsonValue::Null);
    }

    #[test]
    fn test_convert_numeric_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("int_val", DataType::Int64, false),
            Field::new("float_val", DataType::Float64, false),
            Field::new("bool_val", DataType::Boolean, false),
        ]));

        let int_array = Int64Array::from(vec![100, 200]);
        let float_array = Float64Array::from(vec![1.5, 2.5]);
        let bool_array = BooleanArray::from(vec![true, false]);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(int_array),
                Arc::new(float_array),
                Arc::new(bool_array),
            ],
        ).unwrap();

        let result = DataFusionResultConverter::convert_to_query_result(schema, vec![batch]);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.row_count, 2);

        // Check values
        assert_eq!(result.rows[0][0], json!(100));
        assert_eq!(result.rows[0][1], json!(1.5));
        assert_eq!(result.rows[0][2], json!(true));

        assert_eq!(result.rows[1][0], json!(200));
        assert_eq!(result.rows[1][1], json!(2.5));
        assert_eq!(result.rows[1][2], json!(false));
    }

    #[test]
    fn test_empty_result() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
        ]));

        let result = DataFusionResultConverter::convert_to_query_result(schema, vec![]);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.row_count, 0);
        assert_eq!(result.rows.len(), 0);
    }
}
