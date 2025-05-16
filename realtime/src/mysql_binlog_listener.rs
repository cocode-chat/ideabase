use tokio::task::JoinHandle;
use mysql_binlog_connector_rust::{binlog_client::BinlogClient, event::event_data::EventData};

/// 监听 MySQL 的 binlog 变更
/// 
/// 该函数连接到 MySQL 数据库并开始监听 binlog 事件，处理数据变更操作（插入、更新、删除）
/// 
/// # 参数
/// 
/// * `data_source` - 数据库连接信息，包含用户名、密码、主机等
/// * `server_id` - 连接 MySQL 时使用的服务器 ID，需要唯一标识客户端
/// * `binlog_filename` - 要读取的 binlog 文件名，如 "mysql-bin.000001"
pub fn start_mysql_binlog_listener(mysql_jdbc: String, server_id: u64, binlog_filename: &str) -> JoinHandle<()> {
    let binlog_filename = binlog_filename.to_string();
    tokio::spawn(async move {
        read_mysql_binlog(mysql_jdbc, server_id, &binlog_filename).await;
    })
}

async fn read_mysql_binlog(mysql_jdbc: String, server_id: u64, binlog_filename: &str) {
    let mut binlog_client = BinlogClient {
        server_id,
        url: mysql_jdbc,
        binlog_filename: binlog_filename.to_string(),
        binlog_position: 0,
        gtid_enabled: false,
        gtid_set: String::new(),
        heartbeat_interval_secs: 10,
        timeout_secs: 6,
    };

    loop {
        match binlog_client.connect().await {
            Ok(mut stream) => {
                log::info!("MySQL.binlog Stream connected...");
                loop {
                    match stream.read().await {
                        Ok((_, data)) => handle_binlog_event(data),
                        Err(e) => {
                            log::error!("读取MySQL.binlog事件失败: {:?}", e);
                            break; // 连接断开，跳出内层循环，重试连接
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("连接 MySQL binlog 失败: {:?}", e);
            }
        }
        // 等待一段时间后重试连接
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

fn handle_binlog_event(data: EventData) {
    match data {
        EventData::WriteRows(e) => {
            for row in e.rows {
                log::info!("插入行: {:?}", row.column_values);
            }
        }
        EventData::UpdateRows(e) => {
            for (before, after) in e.rows {
                log::info!("更新行: {:?} -> {:?}", before.column_values, after.column_values);
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
