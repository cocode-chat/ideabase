mod controllers;

use common::log::init_tk_log;
use common::yaml::{load_env, GlobalEnv};
use database::{init_datasource_conn, core::DBConn};
use realtime::init_mysql_binlog_listener;

#[macro_use] extern crate lazy_static;
lazy_static! {
    pub static ref G_ENV: GlobalEnv = load_env();
}

// 全局数据库连接池
pub static G_DB: once_cell::sync::OnceCell<DBConn> = once_cell::sync::OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 日志
    init_tk_log();
    // 数据源
    let db_conn = init_datasource_conn((&G_ENV).datasource.clone()).await.expect("datasource init error");
    G_DB.set(db_conn).unwrap();
    // 实时数据库
    init_mysql_binlog_listener((&G_ENV).datasource.clone());

    // http server
    let http_server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(controllers::configure_cors())
            .configure(controllers::register_routes)
    });
    log::info!("IDEA-BASE starting at http://0.0.0.0:8080");
    http_server.workers(4).bind(("0.0.0.0", 8080))?.run().await
}