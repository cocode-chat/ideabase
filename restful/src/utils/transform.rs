use std::collections::HashMap;
use serde_json::{Value, Map};

/// 将一个平铺了 “a/b/c” 风格键的 JSON 对象，转换为嵌套版本
pub fn transform_salve_value(input_map: HashMap<String, Value>) -> HashMap<String, Value> {
    let mut result_map = HashMap::new();
    for (raw_key, value) in input_map.into_iter() {
        // 拆分路径
        let segments: Vec<&str> = raw_key.split('/').collect();
        if segments.len() == 1 {
            result_map.insert(raw_key, value);
            continue;
        }
        let first = segments[0];
        let rest = &segments[1..];
        // 如果第一级是数组路径，并且值本身是数组，则对每个元素分别构造
        if first.ends_with("[]") {
            // 确保目标数组存在
            let entry = result_map
                .entry(first.to_string())
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Value::Array(arr) = entry {
                // 对 value 中每个元素单独处理
                if let Value::Array(orig_elems) = value {
                    for elem in orig_elems {
                        // 从叶子开始向外包装
                        let mut node = elem;
                        for &seg in rest.iter().rev() {
                            if seg.ends_with("[]") {
                                // 内层数组保持元素分离，不自动再包数组
                                let mut m = Map::new();
                                m.insert(seg.to_string(), Value::Array(vec![node]));
                                node = Value::Object(m);
                            } else {
                                let mut m = Map::new();
                                m.insert(seg.to_string(), node);
                                node = Value::Object(m);
                            }
                        }
                        arr.push(node);
                    }
                } else {
                    // 如果不是数组，按单值处理
                    let mut node = value.clone();
                    for &seg in rest.iter().rev() {
                        if seg.ends_with("[]") {
                            let mut m = Map::new();
                            m.insert(seg.to_string(), Value::Array(vec![node]));
                            node = Value::Object(m);
                        } else {
                            let mut m = Map::new();
                            m.insert(seg.to_string(), node);
                            node = Value::Object(m);
                        }
                    }
                    arr.push(node);
                }
            }
        } else {
            // 普通键或非顶层数组，按原逻辑
            let mut node = value;
            for &seg in rest.iter().rev() {
                if seg.ends_with("[]") {
                    let arr = Value::Array(vec![node]);
                    let mut m = Map::new();
                    m.insert(seg.to_string(), arr);
                    node = Value::Object(m);
                } else {
                    let mut m = Map::new();
                    m.insert(seg.to_string(), node);
                    node = Value::Object(m);
                }
            }
            result_map.insert(first.to_string(), node);
        }
    }
    result_map
}


#[cfg(test)]
mod tests {
    use crate::utils::transform::transform_salve_value;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_1() {
        // 示例 1
        let src1 = r#"
        {
            "Comment[]/timeline.Comment": [
                { "id": 18711, "content": "呼呼呼呼" }
            ]
        }
        "#;
        let src2 = r#"{
        "Comment[]/timeline.Comment": [
                { "id": 18711, "content": "呼呼呼呼" },
                { "id": 18712, "content": "哈哈哈哈" }
            ]
        }"#;
        let src3 = r#"{
            "Comment[]/timeline.Comment": { "id": 18711, "content": "呼呼呼呼" }
        }"#;
        let src4 = r#"{"Comment[]/User[]/timeline.User": { "id": 18711, "name": "zk" }}"#;

        // 遍历执行所有测试用例
        let test_cases = vec![src1, src2, src3, src4];
        
        for (i, src) in test_cases.iter().enumerate() {
            println!("测试用例 {}", i + 1);
            let v: Value = serde_json::from_str(src).unwrap();
            let input_map: HashMap<String, Value> = v.as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            let out = transform_salve_value(input_map);
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
            println!("----------------------------");
        }
    }
}
