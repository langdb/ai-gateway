use quick_js::JsValue;
use serde_json::{Map, Value};

/// Convert a JsValue to a serde_json::Value
pub fn js_value_to_json(value: &JsValue) -> Value {
    match value {
        JsValue::Undefined => Value::Null,
        JsValue::Null => Value::Null,
        JsValue::Bool(b) => Value::Bool(*b),
        JsValue::Int(i) => Value::Number((*i).into()),
        JsValue::Float(f) => {
            // Handle NaN and Infinity which are not valid JSON
            if f.is_nan() {
                Value::Null
            } else if f.is_infinite() {
                if *f > 0.0 {
                    // Positive infinity
                    Value::String("Infinity".to_string())
                } else {
                    // Negative infinity
                    Value::String("-Infinity".to_string())
                }
            } else {
                // Regular finite number
                match serde_json::Number::from_f64(*f) {
                    Some(n) => Value::Number(n),
                    None => Value::Null,
                }
            }
        }
        JsValue::String(s) => Value::String(s.clone()),
        JsValue::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(js_value_to_json).collect();
            Value::Array(values)
        }
        JsValue::Object(obj) => {
            let mut map = Map::new();
            for (key, value) in obj {
                map.insert(key.clone(), js_value_to_json(value));
            }
            Value::Object(map)
        }
        _ => Value::Null, // Handle __NonExhaustive variant
    }
}

/// Convert a JsValue to a serde_json::Value and consume the JsValue
pub fn js_value_into_json(value: JsValue) -> Value {
    match value {
        JsValue::Undefined => Value::Null,
        JsValue::Null => Value::Null,
        JsValue::Bool(b) => Value::Bool(b),
        JsValue::Int(i) => Value::Number(i.into()),
        JsValue::Float(f) => {
            // Handle NaN and Infinity which are not valid JSON
            if f.is_nan() {
                Value::Null
            } else if f.is_infinite() {
                if f > 0.0 {
                    // Positive infinity
                    Value::String("Infinity".to_string())
                } else {
                    // Negative infinity
                    Value::String("-Infinity".to_string())
                }
            } else {
                // Regular finite number
                match serde_json::Number::from_f64(f) {
                    Some(n) => Value::Number(n),
                    None => Value::Null,
                }
            }
        }
        JsValue::String(s) => Value::String(s),
        JsValue::Array(arr) => {
            let values: Vec<Value> = arr.into_iter().map(js_value_into_json).collect();
            Value::Array(values)
        }
        JsValue::Object(obj) => {
            let mut map = Map::new();
            for (key, value) in obj {
                map.insert(key, js_value_into_json(value));
            }
            Value::Object(map)
        }
        _ => Value::Null, // Handle __NonExhaustive variant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_js_value_to_json_primitives() {
        // Test undefined
        assert_eq!(js_value_to_json(&JsValue::Undefined), Value::Null);

        // Test null
        assert_eq!(js_value_to_json(&JsValue::Null), Value::Null);

        // Test boolean
        assert_eq!(js_value_to_json(&JsValue::Bool(true)), Value::Bool(true));
        assert_eq!(js_value_to_json(&JsValue::Bool(false)), Value::Bool(false));

        // Test integer
        assert_eq!(
            js_value_to_json(&JsValue::Int(42)),
            Value::Number(42.into())
        );
        assert_eq!(
            js_value_to_json(&JsValue::Int(-42)),
            Value::Number((-42).into())
        );

        // Test float
        assert_eq!(
            js_value_to_json(&JsValue::Float(std::f64::consts::PI)),
            Value::Number(serde_json::Number::from_f64(std::f64::consts::PI).unwrap())
        );

        // Test string
        assert_eq!(
            js_value_to_json(&JsValue::String("hello".to_string())),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_js_value_to_json_complex() {
        // Test array
        let js_array = JsValue::Array(vec![
            JsValue::Int(1),
            JsValue::String("two".to_string()),
            JsValue::Bool(true),
        ]);

        let expected_array = Value::Array(vec![
            Value::Number(1.into()),
            Value::String("two".to_string()),
            Value::Bool(true),
        ]);

        assert_eq!(js_value_to_json(&js_array), expected_array);

        // Test object
        let mut js_obj = HashMap::new();
        js_obj.insert("name".to_string(), JsValue::String("John".to_string()));
        js_obj.insert("age".to_string(), JsValue::Int(30));
        js_obj.insert("active".to_string(), JsValue::Bool(true));

        let js_object = JsValue::Object(js_obj);

        let mut expected_map = Map::new();
        expected_map.insert("name".to_string(), Value::String("John".to_string()));
        expected_map.insert("age".to_string(), Value::Number(30.into()));
        expected_map.insert("active".to_string(), Value::Bool(true));

        let expected_object = Value::Object(expected_map);

        assert_eq!(js_value_to_json(&js_object), expected_object);
    }

    #[test]
    fn test_js_value_to_json_special_floats() {
        // Test NaN
        assert_eq!(js_value_to_json(&JsValue::Float(f64::NAN)), Value::Null);

        // Test positive infinity
        assert_eq!(
            js_value_to_json(&JsValue::Float(f64::INFINITY)),
            Value::String("Infinity".to_string())
        );

        // Test negative infinity
        assert_eq!(
            js_value_to_json(&JsValue::Float(f64::NEG_INFINITY)),
            Value::String("-Infinity".to_string())
        );
    }
}
