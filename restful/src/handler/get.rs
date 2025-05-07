
use std::collections::HashMap;
use crate::db::datasource::DBConn;
use crate::db::executor::query_executor::DEFAULT_MAX_COUNT;
use crate::handler::build_rpc_value;
use crate::utils::hashmap::transform;
use crate::db::context::query::{QueryContext, QueryNode, RATIO_PRIMARY};

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
        let query_node = self.query_node.clone();

        // 遍历每个层级的节点
        for (_, nodes) in query_node {
            // 创建一个可变的克隆来进行排序
            let mut sorted_nodes = nodes.clone();
            // 节点排序, 权重高的先处理
            sorted_nodes.sort_by(|a, b| b.borrow().weight.cmp(&a.borrow().weight));
            // 查询每个节点的数据
            for node in sorted_nodes {
                let mut node_borrow = node.borrow_mut();
                // 处理主节点
                if node_borrow.weight >= RATIO_PRIMARY {
                    self.query_primary_node(&mut *node_borrow, db).await;
                } else {
                    self.query_relate_node(&mut *node_borrow, db).await;
                }
            }
        }

        // 构建结果 - 主从节点处理
        let code = self.code;
        let msg = self.err.clone();
        build_rpc_value(code as u32, msg, transform(HashMap::new()))
    }

    async fn query_primary_node(&mut self, node: &mut QueryNode, db: &DBConn) {
        self.query_node_data(node, db).await;
        // 整理被关联的数据
        let node_results = &node.results;
        if node.is_list {
            for result in node_results {
                for (k, v) in result {
                    let full_path = format!("{}/{}", node.path, k);
                    if let Some(entry) = self.relate_field_values.get_mut(&full_path) {
                        match entry {
                            serde_json::Value::Null => { *entry = serde_json::Value::Array(vec![v.clone()]); }
                            serde_json::Value::Array(arr) => { arr.push(v.clone()); }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            if node_results.is_empty() { return; }
            let result = &node_results[0];
            for (k, v) in result {
                let full_path = format!("{}/{}", node.path, k);
                if let Some(existing_value_slot) = self.relate_field_values.get_mut(&full_path) {
                    *existing_value_slot = v.clone();
                }
            }
        }
    }

    async fn query_relate_node(&mut self, node: &mut QueryNode, db: &DBConn) {
        // 依赖的主节点属性整理为 in 条件
        let node_relate_kv = node.relate_kv.clone();
        for (field, primary_node_field_path) in &node_relate_kv {
            match self.relate_field_values.get(primary_node_field_path) {
                Some(value) => {
                    if value.is_null() { return (); }
                    // 克隆值，因为后续会使用它
                    let cloned_value = value.clone();
                    if let serde_json::Value::Array(_) = &cloned_value {
                        node.sql_executor.page_size(serde_json::json!(0), serde_json::json!(DEFAULT_MAX_COUNT));
                    }
                    // 处理 in 条件
                    node.sql_executor.parse_condition(&field, &cloned_value);
                }
                None => { continue; }
            }
        }

        // 查询节点数据
        self.query_node_data(node, db).await;

        // 从节点数据转为map结构, 用于主节点获取从节点数据
        let results = node.results.clone();
        for (field, _) in &node_relate_kv {
            let mut field_map: HashMap<String, Vec<HashMap<String, serde_json::Value>>> = HashMap::new();
            for result in &results {
                let field_value = result.get(field).unwrap();
                let field_path = format!("{}/{}", field, field_value);
                field_map.entry(field_path).or_insert_with(Vec::new).push(result.clone());
            }
            self.relate_node_field_values.insert((&node.path).to_string(), field_map);
        }
    }

    async fn query_node_data(&mut self, node: &mut QueryNode, db: &DBConn) {
        let node_name = &node.name.to_lowercase();
        let node_path = &node.path;
        let node_attrs = &node.attributes;
        let _ = node.sql_executor.parse_table(node_name);
        for (key, value) in node_attrs {
            let _ = node.sql_executor.parse_condition(key, value);
        }
        if node.is_list {
            let parent_path = self.get_parent_node_path(node_path);
            if let Some(parent_node_attrs) = self.parent_node.get(&parent_path).cloned() {
                let page = parent_node_attrs.get("page").cloned().unwrap_or_else(|| serde_json::json!(0));
                let count = parent_node_attrs.get("count").cloned().unwrap_or_else(|| serde_json::json!(DEFAULT_MAX_COUNT));
                node.sql_executor.page_size(page, count);
            } else {
                node.sql_executor.page_size(serde_json::json!(0), serde_json::json!(DEFAULT_MAX_COUNT));
            }
        }
        match node.sql_executor.exec(db).await {
            Ok(results) => {
                node.results = results;
            },
            Err(e) => { // 保存错误信息到上下文
                self.err = Some(e.to_string());
                self.code = 400;
            }
        }
    }


    //
    // fn relocate_query_node(&mut self) {
    //     let query_node = &self.query_node;
    //     for (node_path, shared_node) in query_node {
    //         // 非列表节点，需要查看父节点是否list
    //         if !shared_node.borrow().name.contains("[]") && node_path.contains('/') {
    //             let parent_node_path = self.get_path_parent(&shared_node.borrow().path);
    //             if let Some(parent_node) = self.parent_node.get(parent_node_path.as_str()) {
    //                 if !parent_node.borrow().is_list { return; }
    //                 let parent_borrow = parent_node.borrow();
    //                 let mut shared_node_borrow = shared_node.borrow_mut();
    //                 shared_node_borrow.is_list = shared_node_borrow.is_primary || !shared_node_borrow.relate_kv.contains_key("id");
    //                 shared_node_borrow.sql_executor.page_size(parent_borrow.page.clone(), parent_borrow.count.clone());
    //             }
    //         }
    //         // 反向处理主节点关联信息
    //         let relate_kv = &shared_node.borrow().relate_kv;
    //         for (field, primary_full_path) in relate_kv {
    //             let primary_node_path = self.get_path_parent(&primary_full_path);
    //             let primary_node_field = &primary_full_path.split("/").last().unwrap();
    //             let relate_node_path = format!("{}/{}", node_path, field);
    //             self.query_node.get(primary_node_path.as_str()).map(|primary_node| {
    //                 primary_node.borrow_mut().relate_kv.insert((&primary_node_field).to_string(), relate_node_path);
    //             });
    //         }
    //
    //     }
    // }



    /// 构建最终响应结果
    ///
    /// 处理主节点查询结果，并根据节点类型(列表/单个)构建不同的响应结构
    /// 对于列表类型节点，会构建包含所有结果的数组
    /// 对于单个节点，会处理关联的从节点数据
    ///
    /// # 参数
    /// * `primary_node` - 主节点列表，包含Rc<RefCell<QueryNode>>类型的节点
    ///
    /// # 返回值
    /// 返回处理后的扁平化响应映射表，键为节点路径，值为JSON数据
    // fn result(&self, primary_node: Vec<Rc<RefCell<QueryNode>>>) -> HashMap<String, serde_json::Value> {
    //     let mut flat_response_map = HashMap::with_capacity(primary_node.len());
    //
    //     // 遍历所有主节点
    //     for node in primary_node {
    //         let node_ref = node.borrow();
    //         let node_name = node_ref.name.clone();
    //         let node_path = node_ref.path.clone();
    //         let relate_kv = node_ref.relate_kv.clone();
    //         let results = node_ref.results.clone();
    //         let is_list = node_ref.is_list;
    //
    //         // 处理列表类型节点
    //         if is_list {
    //             let primary_node_result_list = results.into_iter()
    //                 .map(|result| self.build_primary_value(&node_name, result, &relate_kv))
    //                 .collect::<Vec<_>>();
    //             // 获取父级命名空间并插入结果列表
    //             let namespace = self.get_path_parent(&node_path);
    //             flat_response_map.insert(namespace, serde_json::json!(primary_node_result_list));
    //         }
    //         // 处理单个节点
    //         else {
    //             // 获取第一个结果或空映射表
    //             let result = results.first().cloned().unwrap_or_default();
    //             flat_response_map.insert(node_path.clone(), serde_json::json!(result));
    //
    //             // 处理关联的从节点数据
    //             for (relate_field, relate_node_field_path) in &relate_kv {
    //                 if let Some(relate_field_data) = self.get_relate_data(result.clone(), relate_field, relate_node_field_path) {
    //                     let node_data_path = self.get_path_parent(relate_node_field_path);
    //                     flat_response_map.insert(node_data_path, relate_field_data);
    //                 }
    //             }
    //         }
    //     }
    //     flat_response_map
    // }
    //
    // fn build_primary_value(&self, name: &str, value: HashMap<String, serde_json::Value>, relate_kv: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
    //     let mut result_item_value = HashMap::<String, serde_json::Value>::new();
    //     // 主节点数据
    //     result_item_value.insert(name.to_string(), serde_json::to_value(value.clone()).unwrap());
    //
    //     // 从节点数据
    //     for (relate_field, relate_node_field_path) in relate_kv {
    //         let relate_field_data = self.get_relate_data(value.clone(), relate_field, relate_node_field_path);
    //         if relate_field_data.is_some() {
    //             let node_data_path = relate_node_field_path.split('/').nth_back(1).unwrap();
    //             result_item_value.insert(node_data_path.to_string(), relate_field_data.unwrap());
    //         }
    //     }
    //     result_item_value
    // }
    //
    // fn get_relate_data(&self, primary_value: HashMap<String, serde_json::Value>, relate_field: &str, relate_node_field_path: &str) -> Option<serde_json::Value> {
    //     let relate_node_path = self.get_path_parent(&relate_node_field_path);
    //     let primary_node_field = &relate_node_field_path.split("/").last()?;
    //     let primary_node_field_value = &primary_value.get(relate_field)?;
    //     let relate_field_key = format!("{}/{}", primary_node_field, primary_node_field_value);
    //     let relate_field_data_opt = self.relate_node_field_values.get(&relate_node_path)?.get(&relate_field_key);
    //     match relate_field_data_opt {
    //         None => {
    //             log::debug!("relate.data: {}.{} is empty", &relate_node_path, relate_field_key);
    //             None
    //         }
    //         Some(relate_field_data) => {
    //             if self.query_node.get(&relate_node_path).unwrap().borrow().is_list {
    //                 Some(serde_json::to_value(relate_field_data.clone()).unwrap())
    //             } else {
    //                 Some(serde_json::to_value(relate_field_data[0].clone()).unwrap())
    //             }
    //         }
    //     }
    // }

    fn get_parent_node_path(&self, node_path: &str) -> String {
        node_path.rsplit_once('/').map(|(parent, _)| parent.to_string()).unwrap_or_default()
    }

}