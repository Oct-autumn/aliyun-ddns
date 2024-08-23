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
                            "Please enter the path of server server_config.",
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

                        check_config(&config);

                        Ok((config_path, config))
                    }
                };
            }
            "-t" | "--test" => {
                // TODO: 测试配置文件
            }
            "-h" | "--help" => {
                //显示帮助信息
                println!("Help: ");
                println!("\t-c | --config PATH  > Path to the configuration file");
                println!("\t-t | --test         > Test the configuration file");
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
                    "An Error occurred when parsing config file.\n\tDetails:{}",
                    e.message()
                ),
            ));
        }
        Ok(t) => t,
    };

    return Ok(config);
}

/// 检查配置文件是否符合要求
fn check_config(_config: &Config) {}
