use std::collections::HashMap;

use tracing::{info, trace, warn};

use crate::{
    config::{record::Recorder, IP},
    service::get_ip::{get_ip_via_nic, get_ip_via_socket},
    GLOBAL_CONFIG,
};

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
        // Initialization
        let mut need_recheck = self.enable_recheck;
        let mut record = self.recorder.get_record();
        let mut dns_records: HashMap<String, Vec<(String, String, bool)>> = HashMap::new();

        // Collect DNS records
        for dns_record in GLOBAL_CONFIG.1.records.iter() {
            dns_records
                .entry(dns_record.nic_name.clone().unwrap_or("".to_string()))
                .or_insert(Vec::new())
                .push((
                    dns_record.record_type.clone(),
                    dns_record.hostname.clone(),
                    dns_record.use_temporary_addr,
                ));
        }

        loop {
            // Check IP
            let mut ip_map: HashMap<String, IP>;
            {
                let ip_via_socket = get_ip_via_socket().await;
                let ip_via_nic = get_ip_via_nic();

                if ip_via_socket.is_none() || ip_via_nic.is_empty() {
                    warn!(
                        "Something wrong happened when getting IPs, retry in {} seconds",
                        self.recheck_interval
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(self.recheck_interval))
                        .await;
                    continue;
                }

                // collect IPs
                ip_map = ip_via_nic
                    .iter()
                    .map(|(name, ip)| (name.clone(), ip.clone()))
                    .collect();
                ip_map.insert("".to_string(), ip_via_socket.unwrap());
            }

            // record check time
            record.last_check = chrono::Utc::now().timestamp();
            self.recorder.update_record(record.clone());

            // check if IP changed
            let changed_list = self.check_if_changed(&ip_map);
            if changed_list.is_empty() {
                trace!("IP not changed");
            } else {
                // IP changed, check if need to update
                if !need_recheck {
                    // update IP
                    info!("IP changed, updating DNS records");

                    record.last_ip = ip_map.clone();
                    record.last_update = record.last_check;
                    self.recorder.update_record(record.clone());

                    // update DNS records
                    for nic_name in changed_list {
                        let ips = ip_map.get(&nic_name).unwrap();

                        for dns_record in dns_records.get(&nic_name).unwrap() {
                            let (record_type, hostname, use_temporary_addr) = dns_record;
                            let ip = if record_type == "A" {
                                ips.v4.clone()
                            } else if *use_temporary_addr {
                                ips.v6_temp.clone()
                            } else {
                                ips.v6.clone()
                            };

                            if ip.is_none() {
                                warn!("No IP address found for {}", hostname);
                                continue;
                            }

                            let ip = ip.unwrap().ip().to_string();
                            let result = self
                                .dns_operate
                                .update_dns_record(hostname, record_type, &ip)
                                .await;

                            if result.is_err() {
                                warn!("Failed to update DNS record for {}", hostname);
                            }
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

    fn check_if_changed(&self, ip: &HashMap<String, IP>) -> Vec<String> {
        let mut changed_list = Vec::new();
        let record = self.recorder.get_record();

        for (name, ips) in ip.iter() {
            if !record.last_ip.contains_key(name) {
                changed_list.push(name.clone());
                continue;
            }

            let last_ip = record.last_ip.get(name).unwrap();
            if last_ip != ips {
                changed_list.push(name.clone());
            }
        }

        changed_list
    }
}
