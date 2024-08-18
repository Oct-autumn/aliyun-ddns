pub mod load_config;

use serde::{Deserialize, Serialize};

static LOG_PREFIX: &str = "aliyun-ddns";
static DEFAULT_LOG_LEVEL: &str = "info";
static DEFAULT_DISPLAY_LEVEL: &str = "trace";

/// Last run record
#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    last_ip: String,
    last_check: u64,
    last_update: u64,
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

/// Static config
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub domain: String,
    pub auth: Auth,
    pub log: Log,
    pub interval: Interval,
}

/// Authentication Info
#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {
    auth_id: String,
    auth_token: String,
}

/// Log config
#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    log_path: String,
    log_prefix: String,
    log_level: String,
    display_level: String,
}

/// Interval config
#[derive(Serialize, Deserialize, Debug)]
pub struct Interval {
    check_interval: u64,
    update_interval: u64,
}

impl Config {
    fn new() -> Config {
        Config {
            domain: String::new(),
            auth: Auth::new(),
            log: Log::new(),
            interval: Interval::new(),
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
            log_path: String::new(),
            log_prefix: String::from(LOG_PREFIX),
            log_level: String::from(DEFAULT_LOG_LEVEL),
            display_level: String::from(DEFAULT_DISPLAY_LEVEL),
        }
    }
}

impl Interval {
    fn new() -> Interval {
        Interval {
            check_interval: 0,
            update_interval: 0,
        }
    }
}
