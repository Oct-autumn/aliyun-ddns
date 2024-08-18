use config::load_config::load_server_config;
use util::log_collector::log_collector_init;

mod config;
mod util;

#[tokio::main]
async fn main() {
    let global_config = load_server_config().unwrap();

    // 启动并测试日志记录
    // _guards 用于保证日志记录器在程序结束前不会被回收
    let _guards = match log_collector_init(
        &global_config.1.log.console_directive,
        &global_config.1.log.record_directive,
        global_config.1.log.path.as_str(),
    ) {
        Ok(guards) => guards,
        Err(error) => {
            println!("LogCollectorInit: [Error] {}", error);
            return;
        }
    };

    // Get the current time
    let now = chrono::Utc::now();
    println!("Dynamic DNS Service started at {}", now);
}
