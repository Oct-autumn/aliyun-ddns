pub mod load_config;
pub mod record;

use std::collections::HashMap;

use pnet::ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

pub static LOG_PREFIX: &str = "aliyun-ddns";
static DEFAULT_LOG_LEVEL: &str = "info";

/// IP地址信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IP {
    pub v4: Option<IpNetwork>,
    pub v6: Option<IpNetwork>,
    pub v6_temp: Option<IpNetwork>,
}

impl PartialEq for IP {
    fn eq(&self, other: &Self) -> bool {
        self.v4 == other.v4 && self.v6 == other.v6 && self.v6_temp == other.v6_temp
    }
}

/// 记录上次的运行信息（单独存储于特定文件中）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    pub last_ip: HashMap<String, IP>,
    pub last_check: i64,
    pub last_update: i64,
}

impl Record {
    fn new() -> Record {
        Record {
            last_ip: HashMap::new(),
            last_check: 0,
            last_update: 0,
        }
    }
}

/// 配置信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "empty_string", rename = "domain-name")]
    pub domain_name: String,
    #[serde(rename = "record")]
    pub records: Vec<MonitorRecord>,
    pub auth: Auth,
    pub log: Log,
    pub check: Check,
}

/// 关联的解析记录
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitorRecord {
    #[serde(default = "empty_string", rename = "record-type")]
    pub record_type: String,
    #[serde(default = "empty_string")]
    pub hostname: String,
    #[serde(default = "empty", rename = "nic-name")]
    pub nic_name: Option<String>,
    #[serde(default = "default_temporary_addr", rename = "use-temporary-addr")]
    pub use_temporary_addr: bool,
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
            domain_name: empty_string(),
            records: Vec::new(),
            auth: Auth::new(),
            log: Log::new(),
            check: Check::new(),
        }
    }
}

//impl DNSRecord {
//    fn new() -> DNSRecord {
//        DNSRecord {
//            record_type: empty_string(),
//            hostname: empty_string(),
//        }
//    }
//}

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

fn empty_string() -> String {
    String::new()
}
fn empty() -> Option<String> {
    None
}
fn default_temporary_addr() -> bool {
    true
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
    5
}
