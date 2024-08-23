use tokio::net::UdpSocket;
use tracing::{info, trace, warn};

use crate::config::record::Recorder;

use super::alidns::dns_operation::AliyunDnsOperate;

pub enum IpType {
    V4,
    V6,
}

pub struct IpCheckService {
    pub ip_type: IpType,
    check_interval: u64,
    enable_recheck: bool,
    recheck_interval: u64,
    recorder: Recorder,
    dns_operate: AliyunDnsOperate,
}

impl IpCheckService {
    pub fn new(
        ip_type: IpType,
        check_interval: u64,
        enable_recheck: bool,
        recheck_interval: u64,
        recorder: Recorder,
    ) -> IpCheckService {
        IpCheckService {
            ip_type,
            check_interval,
            enable_recheck,
            recheck_interval,
            recorder,
            dns_operate: AliyunDnsOperate::new(),
        }
    }

    pub async fn start(&mut self, mut shutdown_receiver: tokio::sync::broadcast::Receiver<()>) {
        // 初始化
        let mut need_recheck = self.enable_recheck;
        let mut record = self.recorder.get_record();
        loop {
            // Check IP
            let ip = match Self::get_ip(&self.ip_type).await {
                Some(ip) => ip,
                None => {
                    warn!("Failed to get IP address");
                    String::new()
                }
            };
            // record check time and IP
            record.last_check = chrono::Utc::now().timestamp();
            self.recorder.update_record(record.clone());

            if !ip.is_empty() {
                // check if IP changed
                if ip.eq(&record.last_ip) {
                } else {
                    // IP changed, check if need to update
                    if !need_recheck {
                        // update IP
                        info!("IP updated to {}", ip);
                        record.last_ip = ip.clone();
                        record.last_update = record.last_check;
                        self.recorder.update_record(record.clone());
                        self.dns_operate.update_dns_record(&ip).await.unwrap();
                        info!("IP updated successfully");
                    } else {
                        trace!("IP changed, recheck in {} seconds", self.recheck_interval);
                        need_recheck = false;
                        tokio::select! {
                            _ = tokio::time::sleep(tokio::time::Duration::from_secs(self.recheck_interval)) => continue,    // wait for recheck
                            _ = shutdown_receiver.recv() => break,
                        }
                    }
                }
            }
            need_recheck = self.enable_recheck;
            // wait for next check
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(self.check_interval)) => (),
                _ = shutdown_receiver.recv() => break,
            }
        }
        drop(shutdown_receiver);
    }

    /// Get IP address
    async fn get_ip(ip_type: &IpType) -> Option<String> {
        let bind_addr = match ip_type {
            IpType::V4 => "0.0.0.0:0",
            IpType::V6 => "[::]:0",
        };
        let connect_addr = match ip_type {
            IpType::V4 => "8.8.8.8:80",
            IpType::V6 => "[2001:4860:4860::8888]:80",
        };

        let socket = match UdpSocket::bind(bind_addr).await {
            Ok(s) => s,
            Err(_) => return None,
        };

        match socket.connect(connect_addr).await {
            Ok(()) => (),
            Err(_) => return None,
        };

        match socket.local_addr() {
            Ok(addr) => return Some(addr.ip().to_string()),
            Err(_) => return None,
        };
    }
}
