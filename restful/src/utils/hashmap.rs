use std::collections::HashMap;
use serde_json::{Value, Map};
use common::utils::serde_json_map_to_hashmap;

pub fn transform(input: HashMap<String, Value>) -> HashMap<String, Value> {
    let mut root_value = Value::Object(Map::new());
    for (key, value) in input {
        let path = parse_key(&key);
        insert_value(&mut root_value, &path, value);
    }
    serde_json_map_to_hashmap(root_value.as_object().unwrap())
}

fn parse_key(key: &str) -> Vec<String> {
    if key.contains('/') {
        key.split('/').map(|s| s.to_string()).collect()
    } else {
        vec![key.to_string()]
    }
}

fn insert_value(current: &mut Value, path: &[String], value: Value) {
    if let Value::Object(obj) = current {
        if path.is_empty() {
            return;
        }
        let component = &path[0];
        if path.len() == 1 {
            obj.insert(component.clone(), value);
        } else {
            if component.ends_with("[]") {
                let array_name = component;
                let array = obj.entry(array_name.clone()).or_insert_with(|| {
                    Value::Array(vec![Value::Object(Map::new())])
                });
                if let Value::Array(arr) = array {
                    if arr.is_empty() {
                        arr.push(Value::Object(Map::new()));
                    }
                    if let Some(first) = arr.get_mut(0) {
                        insert_value(first, &path[1..], value);
                    }
                }
            } else {
                let field_name = component;
                let child = obj.entry(field_name.clone()).or_insert_with(|| Value::Object(Map::new()));
                insert_value(child, &path[1..], value);
            }
        }
    } else {
        panic!("Expected object");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transform() {
        let mut input = HashMap::new();
        input.insert("[]/cocode.proj".to_string(), json!([]));
        input.insert("Proj[]/cocode.Proj".to_string(), json!([]));
        input.insert("account/cocode.address".to_string(), json!([]));
        input.insert("codecode.account".to_string(), json!({}));
        input.insert("a[]/b/c".to_string(), json!(1));
        input.insert("a[]/b/d".to_string(), json!(2));

        let output = transform(input);

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}