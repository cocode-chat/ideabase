use std::rc::Rc;
use std::cell::RefCell;
use fnv::FnvHashMap;
use std::collections::HashMap;

use common::utils::serde_json_map_to_hashmap;

use crate::db::datasource::DBConn;
use crate::db::executor::query_executor::{QueryExecutor, DEFAULT_MAX_COUNT};
use crate::handler::build_rpc_value;
use crate::utils::hashmap::transform;

/// 处理GET请求的异步方法
///
/// # 参数
/// * `body_map` - 包含请求参数的HashMap，键为String类型，值为serde_json::Value类型
///
/// # 返回值
/// 返回serde_json::Value类型的JSON响应数据
pub async fn handle_get(db: &DBConn, body_map: HashMap<String, serde_json::Value>) -> serde_json::Value {
    let mut ctx = QueryContext {
        code: 200,
        err: None,
        query_map: body_map,
        parent_node: FnvHashMap::default(),
        query_node: FnvHashMap::default(),
        relate_field_values: HashMap::new(),
        relate_node_field_values: HashMap::new(),
    };
    ctx.response(db).await
}

#[derive(Debug)]
pub struct QueryContext {
    // 状态码
    code: i32,
    // 错误信息
    err: Option<String>,
    //原始请求的JSON数据(map结构)
    query_map: HashMap<String, serde_json::Value>,

    // 父级节点
    parent_node: FnvHashMap<String, Rc<RefCell<QueryNode>>>,
    // 主节点
    query_node: FnvHashMap<String,  Rc<RefCell<QueryNode>>>,

    // 关联字段的值(主节点字段路径 -> 主节点字段值)
    relate_field_values: HashMap<String, serde_json::Value>,

    // 关联字段映射表(从节点父路径 -> 字段对应的值), 用于主节点获取从节点数据
    relate_node_field_values: HashMap<String, HashMap<String, Vec<HashMap<String, serde_json::Value>>>>,
}

#[derive(Debug)]
pub struct QueryNode {
    // 节点名称
    name: String,
    // 当前节点的完整路径
    path: String,
    // 是否主查询节点
    is_primary: bool,
    // 标记是否是列表查询
    is_list: bool,

    // 节点原始请求的JSON数据(map结构)
    query_map: HashMap<String, serde_json::Value>,

    // 分页参数(当前页码)
    page: serde_json::Value,
    // 分页参数(每页数量)
    count: serde_json::Value,
    // SQL执行器，负责生成和执行SQL
    sql_executor: QueryExecutor,

    // 关联字段映射表(属性 -> 主节点字段路径)
    relate_kv: HashMap<String, String>,

    // 查询结果集
    results: Vec<HashMap<String, serde_json::Value>>,
}


impl QueryContext {
    async fn response(&mut self, db: &DBConn) ->  serde_json::Value {
        // 解析查询节点
        self.parse_query_map();
        if self.err.is_some() {
            return serde_json::json!({ "code": self.code, "msg": self.err.clone().unwrap() });
        }
        // 重新定位查询节点
        self.relocate_query_node();

        // 分流主/从节点 (使用 partition 优化)
        let (primary_node_list, relate_node_list): (Vec<_>, Vec<_>) = self
            .query_node
            .values() // 获取 HashMap 中的所有值 (Rc<RefCell<QueryNode>>)
            .cloned() // 克隆 Rc 指针，以便所有权可以转移
            .partition(|node| node.borrow().is_primary); // 根据 is_primary 分区

        // 主节点查询
        for node in &primary_node_list {
            self.query_primary_node(&mut node.borrow_mut(), db).await;
             // 检查主节点查询是否有错误 (如果需要在这里检查)
             if self.err.is_some() {
                 return serde_json::json!({ "code": self.code, "msg": self.err.clone().unwrap() });
             }
        }
        // 从节点查询
        for node in relate_node_list { // 注意：这里不再需要 &
            self.query_relate_node(&mut node.borrow_mut(), db).await;
             // 检查从节点查询是否有错误
             if self.err.is_some() {
                 return serde_json::json!({ "code": self.code, "msg": self.err.clone().unwrap() });
             }
        }

        // 构建结果 - 主从节点处理
        let code = self.code;
        let msg = self.err.clone();
        let flat_response_map = self.result(primary_node_list);
        build_rpc_value(code as u32, msg, transform(flat_response_map))
    }

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
    fn result(&self, primary_node: Vec<Rc<RefCell<QueryNode>>>) -> HashMap<String, serde_json::Value> {
        let mut flat_response_map = HashMap::with_capacity(primary_node.len());
        
        // 遍历所有主节点
        for node in primary_node {
            let node_ref = node.borrow();
            let node_name = node_ref.name.clone();
            let node_path = node_ref.path.clone();
            let relate_kv = node_ref.relate_kv.clone();
            let results = node_ref.results.clone();
            let is_list = node_ref.is_list;

            // 处理列表类型节点
            if is_list {
                let primary_node_result_list = results.into_iter()
                    .map(|result| self.build_primary_value(&node_name, result, &relate_kv))
                    .collect::<Vec<_>>();
                // 获取父级命名空间并插入结果列表
                let namespace = self.get_path_parent(&node_path);
                flat_response_map.insert(namespace, serde_json::json!(primary_node_result_list));
            } 
            // 处理单个节点
            else {
                // 获取第一个结果或空映射表
                let result = results.first().cloned().unwrap_or_default();
                flat_response_map.insert(node_path.clone(), serde_json::json!(result));
                
                // 处理关联的从节点数据
                for (relate_field, relate_node_field_path) in &relate_kv {
                    if let Some(relate_field_data) = self.get_relate_data(result.clone(), relate_field, relate_node_field_path) {
                        let node_data_path = self.get_path_parent(relate_node_field_path);
                        flat_response_map.insert(node_data_path, relate_field_data);
                    }
                }
            }
        }
        flat_response_map
    }

    fn build_primary_value(&self, name: &str, value: HashMap<String, serde_json::Value>, relate_kv: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
        let mut result_item_value = HashMap::<String, serde_json::Value>::new();
        // 主节点数据
        result_item_value.insert(name.to_string(), serde_json::to_value(value.clone()).unwrap());

        // 从节点数据
        for (relate_field, relate_node_field_path) in relate_kv {
            let relate_field_data = self.get_relate_data(value.clone(), relate_field, relate_node_field_path);
            if relate_field_data.is_some() {
                let node_data_path = relate_node_field_path.split('/').nth_back(1).unwrap();
                result_item_value.insert(node_data_path.to_string(), relate_field_data.unwrap());
            }
        }
        result_item_value
    }
    
    fn get_relate_data(&self, primary_value: HashMap<String, serde_json::Value>, relate_field: &str, relate_node_field_path: &str) -> Option<serde_json::Value> {
        let relate_node_path = self.get_path_parent(&relate_node_field_path);
        let primary_node_field = &relate_node_field_path.split("/").last()?;
        let primary_node_field_value = &primary_value.get(relate_field)?;
        let relate_field_key = format!("{}/{}", primary_node_field, primary_node_field_value);
        let relate_field_data_opt = self.relate_node_field_values.get(&relate_node_path)?.get(&relate_field_key);
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
    
    fn parse_query_map(&mut self) {
        // 先克隆 query_map 以避免同时借用
        let query_map = self.query_map.clone();
        for (node_name, node_value) in query_map {
            // 表名校验
            if !node_name.contains("[]") && !node_name.contains('.') {
                self.err = Some(format!("表名or别名格式错误: {}", node_name));
                self.code = 400;
                return;
            }
            match node_value {
                serde_json::Value::Object(node_query_map_value) => {
                    let node_query_map = serde_json_map_to_hashmap(&node_query_map_value);
                    // 构建查询节点
                    let shared_node = self.new_query_node(&node_name, &node_name, node_query_map);
                    // 检查错误状态
                    if self.err.is_some() { return; }
                    // 写入查询节点
                    if shared_node.borrow().path.ends_with("[]") {
                        self.parent_node.insert(node_name.to_string(), shared_node);
                    } else {
                        self.query_node.insert(node_name.to_string(), shared_node);
                    }
                }
                _ => {
                    self.err = Some(format!("值类型不对, key: {}, value: {:?}", node_name, node_value));
                    self.code = 400;
                    return;
                }
            }
        }
    }

    fn relocate_query_node(&mut self) {
        let query_node = &self.query_node;
        for (node_path, shared_node) in query_node {
            // 非列表节点，需要查看父节点是否list
            if !shared_node.borrow().name.contains("[]") && node_path.contains('/') {
                let parent_node_path = self.get_path_parent(&shared_node.borrow().path);
                if let Some(parent_node) = self.parent_node.get(parent_node_path.as_str()) {
                    if !parent_node.borrow().is_list { return; }
                    let parent_borrow = parent_node.borrow();
                    let mut shared_node_borrow = shared_node.borrow_mut();
                    shared_node_borrow.is_list = shared_node_borrow.is_primary || !shared_node_borrow.relate_kv.contains_key("id");
                    shared_node_borrow.sql_executor.page_size(parent_borrow.page.clone(), parent_borrow.count.clone());
                }
            }
            // 反向处理主节点关联信息
            let relate_kv = &shared_node.borrow().relate_kv;
            for (field, primary_full_path) in relate_kv {
                let primary_node_path = self.get_path_parent(&primary_full_path);
                let primary_node_field = &primary_full_path.split("/").last().unwrap();
                let relate_node_path = format!("{}/{}", node_path, field);
                self.query_node.get(primary_node_path.as_str()).map(|primary_node| {
                    primary_node.borrow_mut().relate_kv.insert((&primary_node_field).to_string(), relate_node_path);
                });
            }

        }
    }

    fn new_query_node(&mut self, name: &str, path: &str, query_map: HashMap<String, serde_json::Value>) -> Rc<RefCell<QueryNode>> {
        let shared_node = Rc::new(RefCell::new(QueryNode {
            name: name.to_string(),
            path: path.to_string(),
            is_primary: true,
            is_list: path.ends_with("[]"),
            query_map,

            page: serde_json::json!(0),
            count: serde_json::json!(1),
            sql_executor: QueryExecutor::new(),

            relate_kv: HashMap::new(),
            results: Vec::new(),
        }));

        // 解析节点
        if shared_node.borrow().is_list {
            self.parse_list(&mut shared_node.borrow_mut());
        } else {
            self.parse_one(&mut shared_node.borrow_mut());
        }

        shared_node
    }

    pub fn parse_one(&mut self, node: &mut QueryNode) {
        if let Err(err) = &node.sql_executor.parse_table(&node.name.to_lowercase()) {
            self.err = Some(err.to_string());
            self.code = 400;
            return;
        }
        self.parse_relate_kvs(node);
    }

    pub fn parse_list(&mut self, node: &mut QueryNode) {
        if self.err.is_some() { return; }
        let node_path = &node.path;
        let query_map = &node.query_map;

        // 遍历请求参数进行解析
        for (field, value) in query_map {
            if value.is_null() {
                self.err = Some(format!("field of [{}] value error, {} is nil", node_path, field));
                self.code = 400;
                return;
            }
            // 根据字段名进行不同处理
            match field.as_str() {
                // 处理分页参数
                "page" => node.page = value.to_owned(),
                // 处理数量参数
                "count" => node.count = value.to_owned(),
                // 处理其他字段
                _ => { // 只处理对象类型的值
                    if let serde_json::Value::Object(child_query_map) = value {
                        let child_name = field.as_str();
                        let child_path = format!("{}/{}", node.path, field);
                        let child_query_hash_map = serde_json_map_to_hashmap(child_query_map);
                        // 构建查询节点
                        let shared_node = self.new_query_node(child_name, &child_path, child_query_hash_map);
                        // 检查错误状态
                        if self.err.is_some() { return; }
                        // 写入查询节点
                        if child_name.ends_with("[]") {
                            self.parent_node.insert(child_path, shared_node);
                        } else {
                            self.query_node.insert(child_path, shared_node);
                        }
                    }
                }
            }
        }
    }

    pub fn parse_relate_kvs(&mut self, node: &mut QueryNode) {
        let query_map = &node.query_map;
        // 最后解析@column
        let mut sorted_query_map: Vec<(&String, &serde_json::Value)> = query_map.iter().map(|(k, v)| (k, v)).collect();
        sorted_query_map.sort_by(|a, b| b.0.cmp(&a.0));

        // 遍历键值对进行解析
        for (field, value) in sorted_query_map {
            if value.is_null() {
                self.err = Some(format!("field value error, {} is nil", field));
                self.code = 400;
                return;
            }
            // 处理字符串类型的值
            if let serde_json::Value::String(query_path) = value {
                // 处理关联查询字段(以@结尾的字段)
                if field.ends_with("@") {
                    node.is_primary = false;
                    // 去掉@后缀
                    let field_without_suffix = &field[0..field.len()-1];

                    // 构建完整查询路径
                    let primary_full_path = if query_path.starts_with("/") {
                        format!("{}{}", &node.path, query_path)
                    } else {
                        query_path.to_string()
                    };

                    // 添加到节点关联字段映射表
                    node.relate_kv.insert((&field_without_suffix).to_string(), (&primary_full_path).to_string());
                    // 关联值初始化默认值
                    self.relate_field_values.insert((&primary_full_path).to_string(), serde_json::Value::Null);
                } else if field.starts_with("@column") {
                    let mut columns = value.as_str().unwrap().to_string();
                    let required_fields: Vec<&str> = node.relate_kv.keys().filter(|k| !columns.contains(k.as_str())).map(|k| k.as_str()).collect();
                    if !required_fields.is_empty() {
                        columns.push_str(", ");
                        columns.push_str(&required_fields.join(", "));
                    }
                    node.sql_executor.parse_condition(field, &serde_json::Value::String(columns.to_string()));
                } else { // 普通字符串字段作为查询条件
                    node.sql_executor.parse_condition(field, value);
                }
            } else {
                // 非字符串类型的值直接作为查询条件
                node.sql_executor.parse_condition(field, value);
            }
        }
    }
    fn get_path_parent(&self, node_path: &str) -> String {
        node_path.rsplit_once('/').map(|(parent, _)| parent.to_string()).unwrap_or_default()
    }
}