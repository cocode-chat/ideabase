use tokio::task::JoinHandle;
use mysql_binlog_connector_rust::{binlog_client::BinlogClient, event::event_data::EventData};
use common::yaml::DataSource;

/// 监听 MySQL 的 binlog 变更
/// 
/// 该函数连接到 MySQL 数据库并开始监听 binlog 事件，处理数据变更操作（插入、更新、删除）
/// 
/// # 参数
/// 
/// * `data_source` - 数据库连接信息，包含用户名、密码、主机等
/// * `server_id` - 连接 MySQL 时使用的服务器 ID，需要唯一标识客户端
/// * `binlog_filename` - 要读取的 binlog 文件名，如 "mysql-bin.000001"
pub fn start_mysql_binlog_listener(data_source: DataSource, server_id: u64, binlog_filename: &str) -> JoinHandle<()> {
    let binlog_filename = binlog_filename.to_string();
    tokio::spawn(async move {
        read_mysql_binlog(data_source, server_id, &binlog_filename).await;
    })
}

async fn read_mysql_binlog(data_source: DataSource, server_id: u64, binlog_filename: &str) {
    let username = data_source.username;
    let password = data_source.password;
    let host = data_source.host;
    let port = data_source.port;
    // let database = data_source.database;
    let jdbc_url = format!("mysql://{}:{}@{}:{}", username, password, host, port);

    // 2. 构造 BinlogClient
    let mut client = BinlogClient {
        server_id,
        url: jdbc_url,
        binlog_filename: binlog_filename.to_string(),
        binlog_position: 4,
        gtid_enabled: false,
        gtid_set: String::new(),
        heartbeat_interval_secs: 10,
        timeout_secs: 6,
    };

    // 3. 建立连接并获取流
    let mut stream = client.connect().await.unwrap();
    log::debug!("MySQL.binlog Stream connected...");

    // 4. 循环读取事件
    loop {
        let (_, data) = stream.read().await.unwrap();
        match data {
            EventData::WriteRows(e) => {
                for row in e.rows {
                    log::debug!("插入行: {:?}", row.column_values);
                }
            }
            EventData::UpdateRows(e) => {
                for (before, after) in e.rows {
                    log::debug!("更新行: {:?} -> {:?}", before.column_values, after.column_values);
                }
            }
            EventData::DeleteRows(e) => {
                for row in e.rows {
                    log::info!("删除行: {:?}", row.column_values);
                }
            }
            other => {
                log::debug!("其他事件: {:?}", other);
            }
        }
    }
}
