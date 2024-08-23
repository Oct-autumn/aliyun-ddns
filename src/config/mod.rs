pub mod load_config;
pub mod record;

use serde::{Deserialize, Serialize};

pub static LOG_PREFIX: &str = "aliyun-ddns";
static DEFAULT_LOG_LEVEL: &str = "info";

/// 记录上次的运行信息（单独存储于特定文件中）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub last_ip: String,
    pub last_check: i64,
    pub last_update: i64,
}

impl Record {
    fn new() -> Record {
        Record {
            last_ip: String::new(),
            last_check: 0,
            last_update: 0,
        }
    }
}

/// 配置信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_ip_type", rename = "ip-type")]
    pub ip_type: String,
    #[serde(default = "empty_string", rename = "domain-name")]
    pub domain_name: String,
    #[serde(default = "empty_string")]
    pub hostname: String,
    pub auth: Auth,
    pub log: Log,
    pub check: Check,
}

/// Authentication Info
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth {
    /// api认证ID
    #[serde(default = "empty_string", rename = "auth-id")]
    pub auth_id: String,
    /// api认证token
    #[serde(default = "empty_string", rename = "auth-token")]
    pub auth_token: String,
}

/// Log config
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Log {
    /// 是否启用日志文件
    #[serde(default = "default_file_log", rename = "log-to-file")]
    pub log_to_file: bool,

    /// 日志文件存放目录
    #[serde(default = "default_log_path", rename = "log-path")]
    pub log_path: String,
    /// 记录日志的指令
    #[serde(default = "default_log_directive", rename = "record-directive")]
    pub record_directive: String,

    /// 控制台日志的指令
    #[serde(default = "default_log_directive", rename = "console-directive")]
    pub console_directive: String,
}

/// Interval config
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Check {
    #[serde(default = "default_check_interval", rename = "check-interval")]
    pub check_interval: u64,
    #[serde(default = "default_recheck", rename = "enable-recheck")]
    pub enable_recheck: bool,
    #[serde(default = "default_recheck_interval", rename = "recheck-interval")]
    pub recheck_interval: u64,
}

impl Config {
    fn new() -> Config {
        Config {
            ip_type: default_ip_type(),
            domain_name: empty_string(),
            hostname: empty_string(),
            auth: Auth::new(),
            log: Log::new(),
            check: Check::new(),
        }
    }
}

impl Auth {
    fn new() -> Auth {
        Auth {
            auth_id: String::new(),
            auth_token: String::new(),
        }
    }
}

impl Log {
    fn new() -> Log {
        Log {
            log_to_file: default_file_log(),
            log_path: default_log_path(),
            record_directive: default_log_directive(),
            console_directive: default_log_directive(),
        }
    }
}

impl Check {
    fn new() -> Check {
        Check {
            check_interval: default_check_interval(),
            enable_recheck: default_recheck(),
            recheck_interval: default_recheck_interval(),
        }
    }
}

fn default_ip_type() -> String {
    String::from("ipv4")
}

fn empty_string() -> String {
    String::new()
}
fn default_file_log() -> bool {
    false
}
fn default_log_path() -> String {
    String::from("./log")
}
fn default_log_directive() -> String {
    String::from(DEFAULT_LOG_LEVEL)
}
fn default_check_interval() -> u64 {
    43200
}
fn default_recheck() -> bool {
    false
}
fn default_recheck_interval() -> u64 {
    1
}
