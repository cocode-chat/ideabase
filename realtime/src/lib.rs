use crate::mysql_binlog_listener::start_mysql_binlog_listener;

pub mod mysql_binlog_listener;


pub fn init_mysql_binlog_listener(mysql_url: &str) {
    let server_id = 100000;
    let binlog_filename = "mysql-bin.000001".to_string();
    let handle = start_mysql_binlog_listener(mysql_url.to_string(), server_id, &binlog_filename);
    log::info!("mysql.binlog_listener.handle-id: {}", handle.id());
}