use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug,Deserialize,Clone)]
pub struct Settings {

    pub connect_timeout: u64,
    pub network_timeout: u64,
    pub charset: String,
    pub http: HttpSettings,
    pub tracker_server: Vec<String>,
    pub connect_first_by: String,
    pub connection_pool: ConnectionPool,
}


#[derive(Debug,Deserialize,Clone)]
pub struct HttpSettings {
    pub tracker_http_port : u16,
    pub anti_steal_token: String,
    pub secret_key: String,

}

#[derive(Debug,Deserialize,Clone)]
pub struct ConnectionPool{
    pub enabled: bool,
    pub max_count_per_entry: u32,
    pub max_idle_time: u64,
    pub max_wait_time_in_ms: u64,
}

fn load_settings(config_file:&str) -> Result<Settings,ConfigError>{
    let builder = Config::builder()
        .add_source(File::with_name(config_file).format(FileFormat::Ini))
        .set_default("tracker_server",Vec::<String>::new())?
        .set_default("connect_timeout",2)?
        .set_default("network_timeout",30)?
        .set_default("charset","UTF-8")?
        .set_default("http.tracker_http_port",8080)?
        .set_default("http.anti_steal_token","no")?
        .set_default("http.secret_key","FastDFS1234567890")?
        .set_default("connect_first_by","tracker")?
        .set_default("connection_pool.enabled",true)?
        .set_default("connection_pool.max_count_per_entry",10)?
        .set_default("connection_pool.max_idle_time",3600)?
        .set_default("connection_pool.max_wait_time_in_ms",1000)?;
    let config = builder.build()?;

    // 尝试转换为 Settings 结构体，使用 expect 因为 Infallible 不会发生错误
    let settings:Settings = config.try_deserialize().expect("配置转换失败");

    Ok(settings)
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();


/**
 * 第一次调用时加载配置文件，之后直接返回配置，默认路径为 fastdfs.conf
 */
pub fn get_settings(location_opt: Option<&str>) -> &'static Settings {
    SETTINGS.get_or_init(|| {
        let path = if let Some(location) = location_opt {
            location
        }else{
            "fastdfs.conf"
        };
        load_settings(path).unwrap()
    })
}
