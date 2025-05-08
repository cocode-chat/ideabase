mod controllers;

use once_cell::sync::OnceCell;
use common::log::init_tk_log;
use common::yaml::{load_env, GlobalEnv};
use database::{init_datasource_conn, core::DBConn};
use realtime::init_mysql_binlog_listener;
use realtime::mysql_binlog_listener::start_mysql_binlog_listener;

#[macro_use] extern crate lazy_static;
lazy_static! {
    pub static ref G_ENV: GlobalEnv = load_env();
}

pub static G_DB: OnceCell<DBConn> = OnceCell::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 基础初始化
    init_tk_log();
    init_datasource_conn((&G_ENV).datasource.clone()).await;

    // 实时数据库
    init_mysql_binlog_listener((&G_ENV).datasource.clone());

    // build http server
    let http_server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .wrap(configure_cors())
            .wrap(actix_web::middleware::Logger::default())
            .configure(controllers::register_routes)
    });

    // run http server
    log::info!("IDEA-BASE starting at http://0.0.0.0:8080");
    http_server.workers(4).bind(("0.0.0.0", 8080))?.run().await
}

pub fn configure_cors() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allowed_origin("https://ideabase.chat")
        .allowed_methods(vec!["*"])
        .allowed_headers(vec!["content-type"])
        .supports_credentials()
        .max_age(3600)
}