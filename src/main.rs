use std::env;

use config::{load_config::load_server_config, record::Recorder, Config};
use lazy_static::lazy_static;
use service::ip_check::{self, IpCheckService, IpType};
use tokio::{runtime::Runtime, select};
use tracing::{error, info, trace, warn, Instrument};
use util::log_collector::log_collector_init;

mod config;
mod service;
mod util;

lazy_static! {
    /// 全局配置
    pub static ref GLOBAL_CONFIG: (String, Config) = load_server_config().unwrap();
}

fn main() {
    println!("{:?}", env::args().collect::<Vec<String>>());

    let recorder = Recorder::new(GLOBAL_CONFIG.0.clone());

    // 启动并测试日志记录
    // _guards 用于保证日志记录器在程序结束前不会被回收
    let _log_guards = match log_collector_init(
        &GLOBAL_CONFIG.1.log.log_to_file,
        &GLOBAL_CONFIG.1.log.log_path,
        &GLOBAL_CONFIG.1.log.record_directive,
        &GLOBAL_CONFIG.1.log.console_directive,
    ) {
        Ok(guards) => guards,
        Err(error) => {
            println!("LogCollectorInit: [Error] {}", error);
            return;
        }
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    // 获取当前时间，输出启动日志
    info!("Dynamic DNS Service started");

    //let ipv4 = ip_check::get_ip(ip_check::IpType::V6).await;
    //println!("{:?}", ipv4.unwrap());

    // 用于关闭服务的信号
    let (server_shutdown_sender, _server_shutdown_receiver) =
        tokio::sync::broadcast::channel::<()>(1);
    // 用于等待各子任务结束的信号（所有子任务均需持有_guard_sender，_guard_receiver由主线程持有）
    let (_guard_sender, mut _guard_receiver) = tokio::sync::mpsc::channel::<()>(1);

    let service_handle;
    // 启动异步周期任务：按照间隔时间检查 IP 是否发生变化
    {
        let shutdown_receiver = server_shutdown_sender.subscribe(); // 监听关闭信号广播
        let _guard_sender = _guard_sender.clone();
        service_handle = runtime.spawn(
            async move {
                let ip_type = match GLOBAL_CONFIG.1.ip_type.as_str() {
                    "ipv4" => IpType::V4,
                    "ipv6" => IpType::V6,
                    _ => unreachable!(),
                };
                let mut ip_check_service = IpCheckService::new(
                    ip_type,
                    GLOBAL_CONFIG.1.check.check_interval,
                    GLOBAL_CONFIG.1.check.enable_recheck,
                    GLOBAL_CONFIG.1.check.recheck_interval,
                    recorder,
                );
                ip_check_service.start(shutdown_receiver).await;
                // 等待关闭信号
            }
            .instrument(tracing::info_span!("IpCheckTask")),
        );
    }

    // 监听 服务状态 与 停止信号Ctrl+C
    {
        let shutdown_sender = server_shutdown_sender; // 用于向各子线程发送关闭信号
        let service_handle = service_handle;
        let _guard_sender = _guard_sender.clone(); // 用于向主线程发送关闭信号（示意ShutdownListener退出）
        runtime.block_on(
            async move {
                trace!("Waiting for Stop signal");
                select! {
                    _ = service_handle => {
                        warn!("Service Exited by self!");
                        let _ = shutdown_sender.send(()); // 通知其他任务关闭
                        return;
                    }
                    _ = tokio::signal::ctrl_c() => {
                        info!("Received Stop signal, shutting down");
                        let _ = shutdown_sender.send(());
                    }
                }
                drop(_guard_sender); // 释放_guard_sender
            }
            .instrument(tracing::info_span!("ShutdownListener")),
        );
    }

    drop(_guard_sender); // 释放主线程持有的_guard_sender

    // 等待所有任务结束
    runtime.block_on(async {
        info!("Waiting for All Task to exit");
        _guard_receiver.recv().await;
        info!("All Task Exited.");
    });
}
