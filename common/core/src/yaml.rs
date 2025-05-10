use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Yaml}};

// 解析配置项目
pub fn load_env() -> GlobalEnv {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| String::from("dev"));
    let yml_dir = std::env::var("YML_DIR").unwrap_or_else(|_| String::from("yaml"));
    let main_conf = format!("{}/application.yaml", yml_dir);
    let active_conf = format!("{}/application-{}.yaml", yml_dir, profile);
    log::info!("load conf {}, {}", main_conf, active_conf);

    let global_env = Figment::new()
        .merge(Yaml::file(main_conf)).merge(Yaml::file(active_conf))
        .extract().expect("application yaml conf parse error");
    log::info!("core.env \n {}", serde_json::to_string_pretty(&global_env).unwrap());
    global_env
}


/// 配置变量
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalEnv {
    // 缓存配置
    pub cache: Cache,
    // 关系数据源
    pub datasource: DataSource,
    // 向量数据源
    pub vector: VectorDb,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cache {
    // 本地缓存目录
    pub dir: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataSource {
    pub host: String,
    pub port: u32,
    pub username: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorDb {
    pub schema: String,
    pub host: String,
    pub port: u32,
}

