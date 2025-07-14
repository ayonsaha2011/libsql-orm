//! Unit tests for libsql-orm
//!
//! These tests verify core functionality including boolean type conversion
//! and value operations.

#[cfg(test)]
mod value_tests {
    use crate::Value;

    #[test]
    fn test_value_conversions() {
        // Test boolean value conversions
        assert_eq!(Value::from(true), Value::Boolean(true));
        assert_eq!(Value::from(false), Value::Boolean(false));

        // Test integer conversions
        assert_eq!(Value::from(42i64), Value::Integer(42));

        // Test string conversions
        assert_eq!(Value::from("hello"), Value::Text("hello".to_string()));
        assert_eq!(
            Value::from("hello".to_string()),
            Value::Text("hello".to_string())
        );

        // Test float conversions
        assert_eq!(
            Value::from(std::f64::consts::PI),
            Value::Real(std::f64::consts::PI)
        );

        // Test null
        assert_eq!(Value::Null, Value::Null);
    }

    #[test]
    fn test_value_from_json() {
        use serde_json;

        // Test conversion from JSON values
        let json_bool = serde_json::Value::Bool(true);
        let value: Value = json_bool.into();
        assert_eq!(value, Value::Boolean(true));

        let json_number = serde_json::Value::Number(serde_json::Number::from(42));
        let value: Value = json_number.into();
        assert_eq!(value, Value::Integer(42));

        let json_string = serde_json::Value::String("test".to_string());
        let value: Value = json_string.into();
        assert_eq!(value, Value::Text("test".to_string()));

        let json_null = serde_json::Value::Null;
        let value: Value = json_null.into();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_boolean_type_detection() {
        // This test would be for the boolean conversion logic in the macro
        // We can't easily test the macro-generated code here, but we can verify
        // that boolean values work as expected

        let bool_val = Value::Boolean(true);
        match bool_val {
            Value::Boolean(true) => {}
            _ => panic!("Boolean value should match"),
        }

        let bool_val = Value::Boolean(false);
        match bool_val {
            Value::Boolean(false) => {}
            _ => panic!("Boolean value should match"),
        }
    }

    #[test]
    fn test_optional_conversions() {
        // Test optional value conversions
        let some_string: Option<String> = Some("test".to_string());
        let value: Value = some_string.into();
        assert_eq!(value, Value::Text("test".to_string()));

        let none_string: Option<String> = None;
        let value: Value = none_string.into();
        assert_eq!(value, Value::Null);

        let some_int: Option<i64> = Some(42);
        let value: Value = some_int.into();
        assert_eq!(value, Value::Integer(42));

        let none_int: Option<i64> = None;
        let value: Value = none_int.into();
        assert_eq!(value, Value::Null);

        let some_bool: Option<bool> = Some(true);
        let value: Value = some_bool.into();
        assert_eq!(value, Value::Boolean(true));

        let none_bool: Option<bool> = None;
        let value: Value = none_bool.into();
        assert_eq!(value, Value::Null);
    }
}
