use std::io::{Error, ErrorKind};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, Layer};

pub fn log_collector_init(
    enable_file_log: &bool,
    log_path: &str,
    record_log_directive: &String,
    console_log_directive: &String,
) -> Result<Option<(WorkerGuard, WorkerGuard)>, Error> {
    let log_file_prefix = crate::config::LOG_PREFIX;
    let console_layer = {
        // 构建控制台过滤器
        let console_filter_layer = match EnvFilter::builder().parse(console_log_directive.as_str())
        {
            Ok(filter_layer) => filter_layer,
            Err(_) => {
                // 如果过滤器解析失败，使用默认的 warn 级别
                println!("LogCollectorInit: [Warning] console_directive is invalid, use default log_directive: info");
                EnvFilter::builder()
                    .with_default_directive(Level::INFO.into())
                    .parse_lossy("")
            }
        };
        // 构建控制台输出层
        let console_layer = fmt::layer()
            .with_timer(fmt::time::ChronoLocal::new(
                "%Y-%m-%dT%H:%M:%S%.6f UTC%:z".to_owned(),
            ))
            .with_writer(std::io::stdout)
            .pretty()
            .with_filter(console_filter_layer);
        console_layer
    };
    if *enable_file_log {
        let (file_layer, guard) = {
            // 构建文件过滤器
            let file_filter_layer = match EnvFilter::builder().parse(record_log_directive.as_str())
            {
                Ok(filter_layer) => filter_layer,
                Err(_) => {
                    // 如果过滤器解析失败，使用默认的 info 级别
                    println!("LogCollectorInit: [Warning] record_directive is invalid, use default log_directive: info");
                    EnvFilter::builder()
                        .with_default_directive(Level::INFO.into())
                        .parse_lossy("")
                }
            };
            // 构建文件输出层
            let file_appender = rolling::daily(log_path, log_file_prefix);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            let file_layer = fmt::layer()
                .with_timer(fmt::time::ChronoLocal::new(
                    "%Y-%m-%dT%H:%M:%S%.6f UTC%:z".to_owned(),
                ))
                .with_writer(non_blocking)
                .with_ansi(false)
                .pretty()
                .with_filter(file_filter_layer);

            (file_layer, guard)
        };

        let (latest_file_layer, latest_guard) = {
            // 构建文件过滤器
            let file_filter_layer = match EnvFilter::builder().parse(record_log_directive.as_str())
            {
                Ok(filter_layer) => filter_layer,
                Err(_) => {
                    // 如果过滤器解析失败，使用默认的 info 级别
                    println!("LogCollectorInit: [Warning] record_directive is invalid, use default log_directive: info");
                    EnvFilter::builder()
                        .with_default_directive(Level::INFO.into())
                        .parse_lossy("")
                }
            };
            // 构建最新文件输出层
            let _ = std::fs::remove_file(format!("{}/latest.log", log_path)); // 检查是否存在latest.log文件，如果存在则删除
            let latest_file_appender = rolling::never(log_path, "latest.log");
            let (latest_non_blocking, latest_guard) =
                tracing_appender::non_blocking(latest_file_appender);
            let latest_file_layer = fmt::layer()
                .with_timer(fmt::time::ChronoLocal::new(
                    "%Y-%m-%dT%H:%M:%S%.6f UTC%:z".to_owned(),
                ))
                .with_writer(latest_non_blocking)
                .with_ansi(false)
                .pretty()
                .with_filter(EnvFilter::from(file_filter_layer));
            (latest_file_layer, latest_guard)
        };

        // 格式化输出
        let subscriber = tracing_subscriber::registry()
            .with(console_layer)
            .with(file_layer)
            .with(latest_file_layer);

        // 设为默认日志记录器
        match tracing::subscriber::set_global_default(subscriber) {
            Ok(_) => {}
            Err(error) => {
                return Err(Error::new(ErrorKind::Other, error));
            }
        };
        return Ok(Some((guard, latest_guard)));
    } else {
        // 格式化输出
        let subscriber = tracing_subscriber::registry().with(console_layer);

        // 设为默认日志记录器
        match tracing::subscriber::set_global_default(subscriber) {
            Ok(_) => {}
            Err(error) => {
                return Err(Error::new(ErrorKind::Other, error));
            }
        };
        return Ok(None);
    }
}
