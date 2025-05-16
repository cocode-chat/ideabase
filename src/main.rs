mod controllers;
pub mod global;
pub mod service;

use common::log::init_tk_log;
use common::yaml::{load_env_yaml, GlobalEnv};
use database::{core::init_datasource_conn, core::DBConn};
use rag::handler::etl_handler::init_vector_db;

#[macro_use] extern crate lazy_static;
lazy_static! {
    pub static ref G_ENV: GlobalEnv = load_env_yaml();
}

// 全局数据库连接池
pub static G_DB: once_cell::sync::OnceCell<DBConn> = once_cell::sync::OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 日志
    init_tk_log();

    // 数据源
    if let Ok(mysql_url) = std::env::var("MYSQL_URL") {
        let db_conn = init_datasource_conn(&mysql_url).await.expect("datasource init error");
        G_DB.set(db_conn).unwrap();
        
        // 实时数据 - 在单独的线程中执行
        // tokio::spawn(async move {
        //     init_mysql_binlog_listener(&mysql_url);
        // });
        
        // 向量数据库 - 在单独的线程中执行
        let db_conn = G_DB.get().unwrap().clone();
        tokio::spawn(async move {
            log::info!("tokio::spawn ... init vector db");
            init_vector_db(&db_conn).await;
        });
    } else {
        log::error!("MYSQL_URL not set");
        std::process::exit(-11);
    }

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