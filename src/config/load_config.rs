use std::env;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Result};

use crate::config::Config;

/// 解析参数并读取配置文件
///
/// # Return
///     Result<(String, Config)>: (config_path, Config)
pub fn load_server_config() -> Result<(String, Config)> {
    // configuration file path
    let mut config_path: String = String::from("");

    let args: Vec<String> = env::args().collect();
    let mut it = args.iter();
    let mut arg = it.next();

    let mut config = Config::new();

    // Args:    -c | --config PATH     > Path to the configuration file
    //          -h | --help            > Show help message

    while arg.is_some() {
        match arg.unwrap().as_str() {
            "-c" | "--config" => {
                //读取配置文件
                arg = it.next();
                return match arg {
                    None => {
                        //return错误
                        Err(Error::new(
                            ErrorKind::NotFound,
                            "Please enter the path of service config.",
                        ))
                    }
                    Some(config_file_path_arg) => {
                        config_path = config_file_path_arg.clone();
                        config = parse_and_read_config_file(&format!(
                            "{}/config.toml",
                            config_file_path_arg
                        ))?;
                        println!(
                            "ConfigLoad: [Warning] Using config file. CLI args will be ignored."
                        );

                        match check_config(&config) {
                            Err(e) => Err(Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "An Error occurred when checking config file.\n\tDetails: {}",
                                    e
                                ),
                            )),
                            Ok(_) => Ok((config_path, config)),
                        }
                    }
                };
            }
            "-t" | "--test" => {
                // 测试配置文件
                arg = it.next();
                return match arg {
                    None => {
                        //return错误
                        Err(Error::new(
                            ErrorKind::NotFound,
                            "Please enter the path of service config.",
                        ))
                    }
                    Some(config_file_path_arg) => {
                        config = parse_and_read_config_file(config_file_path_arg)?;

                        match check_config(&config) {
                            Err(e) => Err(Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "An Error occurred when checking config file.\n\tDetails: {}",
                                    e
                                ),
                            )),
                            Ok(_) => {
                                println!("ConfigLoad: [Info] Config file is valid.");
                                std::process::exit(0);
                            }
                        }
                    }
                };
            }
            "-h" | "--help" => {
                //显示帮助信息
                println!("Help: ");
                println!("\t-c | --config PATH  > Path to the configuration file");
                println!("\t-t | --test PATH    > Test the configuration file");
                println!("\t-h | --help         > Show help message");
                return Err(Error::new(ErrorKind::Other, "Help message displayed."));
            }
            _ => {
                arg = it.next();
            }
        }
    }

    return Ok((config_path, config));
}

/// 解析并读取配置文件
fn parse_and_read_config_file(config_file_path_arg: &String) -> Result<Config> {
    let mut config_file = File::open(config_file_path_arg)?;

    let mut config_s: String = String::from("");
    config_file.read_to_string(&mut config_s)?;

    let config = match toml::from_str(&*config_s) {
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "An Error occurred when parsing config file.\n\tDetails: {}",
                    e.message()
                ),
            ));
        }
        Ok(t) => t,
    };

    return Ok(config);
}

/// 检查配置文件是否符合要求
fn check_config(config: &Config) -> Result<()> {
    // 检查是否配置了主域名，是否合法
    if config.domain_name.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "Domain name is empty."));
    }

    // 检查是否至少配置了一个子域名
    if config.dns_records.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "Sub domain is empty."));
    }
    // 检查各子域名的配置是否合法
    for record in &config.dns_records {
        // 检查子域名记录类型
        if record.record_type != "A" && record.record_type != "AAAA" {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Record type is invalid.",
            ));
        }
        // 检查子域名记录值
        if record.hostname.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "Hostname is empty."));
        }
    }

    // 检查是否配置了认证ID和Token
    if config.auth.auth_id.is_empty() || config.auth.auth_token.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Authentication ID or Token is empty.",
        ));
    }

    // 检查日志记录配置是否合法
    if config.log.log_to_file {
        if config.log.log_path.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "Log path is empty."));
        }
    }

    // 检查IP检查服务配置是否合法
    if config.check.check_interval == 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Check interval is invalid.",
        ));
    }
    if config.check.recheck_interval == 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Recheck interval is invalid.",
        ));
    }

    Ok(())
}
