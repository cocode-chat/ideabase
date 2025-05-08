pub mod common;
pub mod core;


use ::common::yaml::DataSource;
use crate::core::DBConn;

// 初始化数据库连接池
pub async fn init_datasource_conn(data_source: DataSource) -> DBConn {
    DBConn::new(data_source).await
}