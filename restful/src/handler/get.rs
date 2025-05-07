use std::collections::HashMap;
use fnv::FnvHashMap;
use crate::db::datasource::DBConn;
use crate::db::executor::query_executor::DEFAULT_MAX_COUNT;
use crate::handler::build_rpc_value;
use crate::utils::hashmap::transform;
use crate::db::context::query::{get_parent_node_path, QueryContext, QueryNode, RATIO_PRIMARY};

/// 处理GET请求的异步方法
///
/// # 参数
/// * `body_map` - 包含请求参数的HashMap，键为String类型，值为serde_json::Value类型
///
/// # 返回值
/// 返回serde_json::Value类型的JSON响应数据
pub async fn handle_get(db: &DBConn, body_map: HashMap<String, serde_json::Value>) -> serde_json::Value {
    let mut ctx = QueryContext::from_json(body_map);
    ctx.response(db).await
}

impl QueryContext {
    async fn response(&mut self, db: &DBConn) ->  serde_json::Value {
        // 克隆 query_node 以避免借用冲突
        let query_node = self.layer_query_node.clone();

        // 遍历每个层级的节点
        for (depth, nodes) in query_node {
            // 创建一个可变的克隆来进行排序
            let mut sorted_nodes = nodes.clone();
            // 节点排序, 权重高的先处理
            sorted_nodes.sort_by(|a, b| b.borrow().weight.cmp(&a.borrow().weight));
            // 查询每个节点的数据
            for node in sorted_nodes {
                let mut node_rc = node.borrow_mut();
                println!("{} weight: {} node: {}", depth, node_rc.weight, node_rc.path);
                // 处理主节点
                if node_rc.weight >= RATIO_PRIMARY {
                    self.query_primary_node(&mut *node_rc, db).await;
                } else {
                    self.query_relate_node(&mut *node_rc, db).await;
                }
            }
        }

        // 遍历数据节点
        let mut flat_response_map = HashMap::new();
        for (node_path, results) in &self.primary_node_data {
            let node_ref = self.query_node.get(node_path).unwrap().borrow();
            let node_name = node_ref.name.clone();
            let node_path = node_ref.path.clone();
            // 创建一个空的HashMap作为备选值
            let related_kv = self.primary_relate_kv.get(&node_path).cloned().unwrap_or_default();
            let is_list = node_ref.is_list;

            if is_list { // 处理列表类型节点
                let primary_node_result_list = results.into_iter()
                    .map(|result| self.build_primary_value(&node_name, result, &related_kv))
                    .collect::<Vec<_>>();
                // 获取父级命名空间并插入结果列表
                let namespace = get_parent_node_path(&node_path);
                flat_response_map.insert(namespace, serde_json::json!(primary_node_result_list));
            } else { // 处理单个节点
                // 获取第一个结果或空映射表
                let result = results.first().cloned().unwrap_or_default();
                flat_response_map.insert(node_path.clone(), serde_json::json!(result));
                // 处理关联的从节点数据
                for (relate_field, relate_node_field_path) in related_kv {
                    let related_field_value = result.get(&relate_field).unwrap();
                    if let Some(relate_field_data) = self.get_relate_data(related_field_value, &relate_node_field_path) {
                        let node_data_path = get_parent_node_path(&relate_node_field_path);
                        flat_response_map.insert(node_data_path, relate_field_data);
                    }
                }
            }
        }

        // 构建结果 - 主从节点处理
        let code = self.code;
        let msg = self.err.clone();
        build_rpc_value(code as u32, msg, transform(flat_response_map))
    }

    fn build_primary_value(&self, name: &str, record: &HashMap<String, serde_json::Value>, related_kv: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
        let mut result_item_value = HashMap::<String, serde_json::Value>::new();
        // 主节点数据
        result_item_value.insert(name.to_string(), serde_json::to_value(record.clone()).unwrap());

        // 从节点数据
        for (related_field, relate_node_field_path) in related_kv {
            let related_field_value = record.get(related_field).unwrap();
            let relate_field_data = self.get_relate_data(related_field_value, relate_node_field_path);
            if relate_field_data.is_some() {
                let node_data_path = relate_node_field_path.split('/').nth_back(1).unwrap();
                result_item_value.insert(node_data_path.to_string(), relate_field_data.unwrap());
            }
        }
        result_item_value
    }

    fn get_relate_data(&self, related_field_value: &serde_json::Value, relate_node_field_path: &str) -> Option<serde_json::Value> {
        let relate_node_path = get_parent_node_path(&relate_node_field_path);
        let relate_node_field = &relate_node_field_path.split("/").last()?;
        let relate_field_key = format!("{}/{}", relate_node_field, related_field_value);
        let relate_field_data_opt = self.slave_node_relate_data.get(&relate_node_path)?.get(&relate_field_key);
        match relate_field_data_opt {
            None => {
                log::debug!("relate.data: {}.{} is empty", &relate_node_path, relate_field_key);
                None
            }
            Some(relate_field_data) => {
                if self.query_node.get(&relate_node_path).unwrap().borrow().is_list {
                    Some(serde_json::to_value(relate_field_data.clone()).unwrap())
                } else {
                    Some(serde_json::to_value(relate_field_data[0].clone()).unwrap())
                }
            }
        }
    }

    async fn query_primary_node(&mut self, node: &mut QueryNode, db: &DBConn) {
        // 查询主节点数据
        let result_opt = self.query_node_data(node, db).await;
        match result_opt {
            Some(node_results) => {
                // 保存查询结果到节点数据映射表
                self.primary_node_data.insert(node.path.clone(), node_results.clone());
                
                // 处理列表类型节点的结果
                if node.is_list {
                    // 遍历列表中的每个结果项
                    for result in node_results {
                        for (k, v) in result {
                            // 构建字段的完整路径
                            let full_path = format!("{}/{}", node.path, k);
                            // 更新关联字段值映射表
                            if let Some(entry) = self.primary_node_related_field_values.get_mut(&full_path) {
                                match entry {
                                    // 如果当前值为空，创建新数组
                                    serde_json::Value::Null => { *entry = serde_json::Value::Array(vec![v.clone()]); }
                                    // 如果已经是数组，追加新值
                                    serde_json::Value::Array(arr) => { arr.push(v.clone()); }
                                    _ => {}
                                }
                            }
                        }
                    }
                } else {
                    // 处理单个节点的结果（非列表）
                    if node_results.is_empty() { return; }
                    // 获取第一个结果项
                    let result = &node_results[0];
                    // 更新关联字段值映射表
                    for (k, v) in result {
                        let full_path = format!("{}/{}", node.path, k);
                        if let Some(existing_value_slot) = self.primary_node_related_field_values.get_mut(&full_path) {
                            *existing_value_slot = v.clone();
                        }
                    }
                }
            }
            None => { return; }
        }
    }

    async fn query_relate_node(&mut self, node: &mut QueryNode, db: &DBConn) {
        // 依赖的主节点属性整理为 in 条件
        let node_path = node.path.clone();
        let node_relate_kv = self.slave_relate_kv.get(&node_path).cloned().unwrap_or_default();
        for (field_name, primary_node_field_path) in &node_relate_kv {
            match self.primary_node_related_field_values.get(primary_node_field_path) {
                Some(value) => {
                    if value.is_null() { return (); }
                    // 克隆值，因为后续会使用它
                    let cloned_value = value.clone();
                    if let serde_json::Value::Array(array) = &cloned_value {
                        node.sql_executor.page_size(serde_json::json!(0), serde_json::json!(array.len()));
                    }
                    // 处理 in 条件
                    node.sql_executor.parse_condition(&field_name, &cloned_value);
                }
                None => { continue; }
            }
        }

        // 查询节点数据
        let result_opt = self.query_node_data(node, db).await;
        match result_opt {
            Some(node_results) => {
                for (field, _) in &node_relate_kv {
                    let mut field_map = FnvHashMap::<String, Vec<HashMap<String, serde_json::Value>>>::default();
                    for result in &node_results {
                        let field_key = if field.ends_with("@") { field[..(field.len()-1)].to_string() } else { field.to_string() };
                        let field_value = result.get(&field_key).unwrap();
                        let field_path = format!("{}/{}", &field_key, field_value);
                        field_map.entry(field_path).or_insert_with(Vec::new).push(result.clone());
                    }
                    self.slave_node_relate_data.insert((&node.path).to_string(), field_map);
                }
            }
            None => { return; }
        }
    }

    async fn query_node_data(&mut self, node: &mut QueryNode, db: &DBConn) -> Option<Vec<HashMap<String, serde_json::Value>>> {
        // 准备SQL查询的基本参数
        let node_name = &node.name.to_lowercase();
        let node_path = &node.path;
        let node_attrs = &node.attributes;
        // 设置查询的表名
        let _ = node.sql_executor.parse_table(node_name);
        // 解析节点属性中的查询条件
        for (key, value) in node_attrs {
            let _ = node.sql_executor.parse_condition(key, value);
        }
        
        // 处理列表查询的分页逻辑
        if node.is_list {
            let parent_path = get_parent_node_path(node_path);
            // 尝试从父节点获取分页参数
            if let Some(parent_node_attrs) = self.namespace_node.get(&parent_path).cloned() {
                // 获取页码和每页数量，如果不存在则使用默认值
                let page = parent_node_attrs.get("page").cloned().unwrap_or_else(|| serde_json::json!(0));
                let count = parent_node_attrs.get("count").cloned().unwrap_or_else(|| serde_json::json!(DEFAULT_MAX_COUNT));
                node.sql_executor.page_size(page, count);
            } else {
                // 父节点不存在时使用默认分页参数
                node.sql_executor.page_size(serde_json::json!(0), serde_json::json!(DEFAULT_MAX_COUNT));
            }
        }
        
        // 执行SQL查询并处理结果
        match node.sql_executor.exec(db).await {
            Ok(results) => { Some(results) },
            Err(e) => { // 保存错误信息到上下文
                self.err = Some(e.to_string());
                self.code = 400;
                None
            }
        }
    }
}