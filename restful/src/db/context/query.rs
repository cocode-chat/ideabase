use std::collections::{BTreeMap, HashMap, VecDeque};
use fnv::FnvHashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::db::executor::query_executor::QueryExecutor;

/// 主节点权重常量
pub const RATIO_PRIMARY: i32 = 10000;
/// 被依赖一次的权重系数
const RATIO_RELATED: i32 = 10;


#[derive(Debug)]
pub struct QueryContext {
    // 状态码
    pub code: i32,
    // 错误信息
    pub err: Option<String>,

    // 关联字段的值(主节点字段路径 -> 主节点字段值)
    pub relate_field_values: HashMap<String, serde_json::Value>,

    // 关联字段映射表(从节点父路径 -> 字段对应的值), 用于主节点获取从节点数据
    pub relate_node_field_values: HashMap<String, HashMap<String, Vec<HashMap<String, serde_json::Value>>>>,

    // 父节点
    pub parent_node: FnvHashMap<String, FnvHashMap<String, serde_json::Value>>,
    // 层级: 节点列表 (改为BTreeMap以支持按键排序)
    pub query_node: BTreeMap<i32, Vec<Rc<RefCell<QueryNode>>>>,
}

#[derive(Debug)]
pub struct QueryNode {
    // 节点名称
    pub name: String,
    // 当前节点的完整路径
    pub path: String,
    // 标记是否是列表查询
    pub is_list: bool,
    /// 属性映射
    pub attributes: HashMap<String, serde_json::Value>,
    /// 依赖字段映射表: 属性名 -> 被依赖节点路径
    pub relate_kv: HashMap<String, String>,

    // 权重
    pub weight: i32,

    // 查询结果集
    pub results: Vec<HashMap<String, serde_json::Value>>,
    // SQL执行器，负责生成和执行SQL
    pub sql_executor: QueryExecutor,
}


impl QueryContext {
    /// 从 JSON 值构建 QueryContext
    pub fn from_json(root: HashMap<String, serde_json::Value>) -> Self {
        let mut parent_node = FnvHashMap::default();
        let mut query_node: BTreeMap<i32, Vec<Rc<RefCell<QueryNode>>>> = BTreeMap::default();
        let mut queue: VecDeque<(String, String, serde_json::Value, i32)> = VecDeque::new();

        for (key, val) in root {
            if key.ends_with("[]") {
                parent_node.insert(key.clone(), collect_scalar_attrs(&val));
                if let Some(map) = val.as_object() {
                    for (k, v) in map {
                        if v.is_object() {
                            queue.push_back((key.clone(), k.clone(), v.clone(), 2));
                        }
                    }
                }
            } else {
                queue.push_back((String::new(), key.clone(), val.clone(), 1));
            }
        }

        while let Some((parent, name, node_val, depth)) = queue.pop_front() {
            let full_path = if parent.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", parent, name)
            };

            if name.ends_with("[]") {
                parent_node.insert(full_path.clone(), collect_scalar_attrs(&node_val));
                if let Some(map) = node_val.as_object() {
                    for (k, v) in map {
                        if v.is_object() {
                            queue.push_back((full_path.clone(), k.clone(), v.clone(), depth + 1));
                        }
                    }
                }
            } else {
                let mut attributes = HashMap::new();
                let mut relate_kv = HashMap::new();
                if let Some(map) = node_val.as_object() {
                    for (ak, av) in map {
                        if is_scalar_field(av) {
                            attributes.insert(ak.clone(), av.clone());
                            if let serde_json::Value::String(s) = av {
                                if let Some(pos) = s.rfind('/') {
                                    let parent_path = s[..pos].to_string();
                                    relate_kv.insert(ak.clone(), parent_path);
                                }
                            }
                        }
                    }
                }
                let is_list = parent.ends_with("[]");
                let node = QueryNode {
                    name: name.clone(),
                    path: full_path.clone(),
                    weight: 0,
                    is_list,
                    attributes,
                    relate_kv,
                    sql_executor: QueryExecutor::new(),
                    results: Vec::new(),
                };

                query_node.entry(depth).or_default().push(Rc::new(RefCell::new(node)));
            }
        }

        let mut ctx = QueryContext { code: 0, err: None, relate_field_values: Default::default(), relate_node_field_values: Default::default(), parent_node, query_node };
        ctx.compute_weights();
        ctx
    }

    /// 计算每个节点的权重
    fn compute_weights(&mut self) {
        // 收集所有节点引用
        let all_nodes: Vec<Rc<RefCell<QueryNode>>> = self.query_node.values().flatten().cloned().collect();

        // 统计被依赖次数
        let mut counts: HashMap<String, u32> = HashMap::new();
        for rc in &all_nodes {
            let relate = rc.borrow().relate_kv.clone();
            for parent_path in relate.values() {
                *counts.entry(parent_path.clone()).or_insert(0) += 1;
            }
        }

        // 设置权重
        for rc in &all_nodes {
            // 临时借用字段
            let (path, has_dep) = {
                let b = rc.borrow();
                (b.path.clone(), !b.relate_kv.is_empty())
            };
            let count = counts.get(&path).copied().unwrap_or(0);
            let addition = if count > 0 { RATIO_RELATED.pow(count) } else { 0 };
            let weight = if !has_dep {
                RATIO_PRIMARY + addition
            } else {
                addition
            };
            rc.borrow_mut().weight = weight;
        }
    }
}

/// 判断是否为标量
fn is_scalar_field(v: &serde_json::Value) -> bool {
    v.is_number() || v.is_string() || v.is_boolean()
}

/// 从 JSON 对象中收集标量属性
fn collect_scalar_attrs(v: &serde_json::Value) -> FnvHashMap<String, serde_json::Value> {
    let mut scalar_attrs = FnvHashMap::default();
    if let Some(o) = v.as_object() {
        for (k, val) in o {
            if is_scalar_field(val) {
                scalar_attrs.insert(k.clone(), val.clone());
            }
        }
    }
    scalar_attrs
}

#[cfg(test)]
mod tests {
    use common::json::json_to_json_value;
    use common::utils::serde_json_map_to_hashmap;
    use crate::db::context::query::QueryContext;

    #[test]
    fn test_main() {
        let json_str = r#"
    {
      "[]":{
        "page":0,
        "count":2,
        "timeline.Moment":{ "content$":"%a%" },
        "timeline.User":{ "id@":"[]/timeline.Moment/user_id","@column":"id,username,avatar" },
        "Comment[]":{
          "count":2,
          "timeline.Comment":{"moment_id@":"[]/timeline.Moment/id"},
          "timeline.User":{ "id@":"[]/Comment[]/timeline.Comment/user_id","@column":"id,username,avatar" }
        }
      },
      "timeline.Moment":{ "id":28710 },
      "timeline.User":{ "id@":"timeline.Moment/user_id" }
    }
    "#;

        let v = json_to_json_value(json_str);
        let ctx = QueryContext::from_json(serde_json_map_to_hashmap(v.as_object().unwrap()));
        for (k, v) in &ctx.query_node {
            println!("{}: {:?}", k, v);
        }
    }
}
