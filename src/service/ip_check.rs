use tokio::net::UdpSocket;
use tracing::{info, trace, warn};

use crate::{config::record::Recorder, GLOBAL_CONFIG};

use super::alidns::dns_operation::AliyunDnsOperate;

pub struct IpCheckService {
    check_interval: u64,
    enable_recheck: bool,
    recheck_interval: u64,
    recorder: Recorder,
    dns_operate: AliyunDnsOperate,
}

impl IpCheckService {
    pub fn new(
        check_interval: u64,
        enable_recheck: bool,
        recheck_interval: u64,
        recorder: Recorder,
    ) -> IpCheckService {
        IpCheckService {
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
            let ip = Self::get_ip().await;

            if ip.is_none() {
                warn!(
                    "Failed to get IP address, retry in {} seconds",
                    self.recheck_interval
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(self.recheck_interval)).await;
                continue;
            }
            let ip = ip.unwrap();

            // record check time and IP
            record.last_check = chrono::Utc::now().timestamp();
            self.recorder.update_record(record.clone());

            // check if IP changed
            if ip.0.eq(&record.last_ip.0) && ip.1.eq(&record.last_ip.1) {
                trace!("IP not changed");
            } else {
                // IP changed, check if need to update
                if !need_recheck {
                    // update IP
                    info!("IP updated to:\n\tipv4: {}\t\nipv6: {}", ip.0, ip.1);
                    record.last_ip = ip.clone();
                    record.last_update = record.last_check;
                    self.recorder.update_record(record.clone());

                    // update DNS records
                    for dns_record in GLOBAL_CONFIG.1.dns_records.iter() {
                        if dns_record.record_type.eq("A") {
                            self.dns_operate
                                .update_dns_record(&ip.0, dns_record)
                                .await
                                .unwrap();
                            info!(
                                "Updated A record for {}.{} to {}",
                                dns_record.hostname, GLOBAL_CONFIG.1.domain_name, ip.0
                            );
                        } else if dns_record.record_type.eq("AAAA") {
                            self.dns_operate
                                .update_dns_record(&ip.1, dns_record)
                                .await
                                .unwrap();
                            info!(
                                "Updated AAAA record for {}.{} to {}",
                                dns_record.hostname, GLOBAL_CONFIG.1.domain_name, ip.1
                            );
                        }
                    }
                    info!("All DNS records updated successfully");
                    need_recheck = self.enable_recheck;
                } else {
                    trace!("IP changed, recheck in {} seconds", self.recheck_interval);
                    need_recheck = false;
                    tokio::select! {
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(self.recheck_interval)) => continue,    // wait for recheck
                        _ = shutdown_receiver.recv() => break,
                    }
                }
            }

            // wait for next check
            tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(self.check_interval)) => (),
                _ = shutdown_receiver.recv() => break,
            }
        }
        drop(shutdown_receiver);
    }

    /// Get IP address
    async fn get_ip() -> Option<(String, String)> {
        let bind_addr = ("0.0.0.0:0", "[::]:0");
        let connect_addr = ("8.8.8.8:80", "[2001:4860:4860::8888]:80");

        let socket_v4 = match UdpSocket::bind(bind_addr.0).await {
            Ok(s) => s,
            Err(_) => return None,
        };
        match socket_v4.connect(connect_addr.0).await {
            Ok(()) => (),
            Err(_) => return None,
        };

        let socket_v6 = match UdpSocket::bind(bind_addr.1).await {
            Ok(s) => s,
            Err(_) => return None,
        };
        match socket_v6.connect(connect_addr.1).await {
            Ok(()) => (),
            Err(_) => return None,
        };

        Some((
            match socket_v4.local_addr() {
                Ok(addr) => addr.ip().to_string(),
                Err(_) => return None,
            },
            match socket_v6.local_addr() {
                Ok(addr) => addr.ip().to_string(),
                Err(_) => return None,
            },
        ))
    }
}
