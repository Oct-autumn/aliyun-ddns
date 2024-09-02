use std::net::IpAddr;

use pnet::ipnetwork::IpNetwork;
use tokio::net::UdpSocket;
use tracing::debug;

use crate::config::IP;

/// Get IP address from UDP socket
pub async fn get_ip_via_socket() -> Option<IP> {
    let bind_addr = ("0.0.0.0:0", "[::]:0");
    let connect_addr = ("8.8.8.8:80", "[2001:4860:4860::8888]:80");

    // Get IPv4 address
    let mut v4_addr: Option<IpNetwork> = None;
    match UdpSocket::bind(bind_addr.0).await {
        Ok(s) => {
            match s.connect(connect_addr.0).await {
                Ok(()) => v4_addr = Some(IpNetwork::from(s.local_addr().unwrap().ip())),
                Err(_) => (),
            };
        }
        Err(_) => (),
    };

    // Get IPv6 address
    let mut v6_addr: Option<IpNetwork> = None;
    match UdpSocket::bind(bind_addr.1).await {
        Ok(s) => {
            match s.connect(connect_addr.1).await {
                Ok(()) => v6_addr = Some(IpNetwork::from(s.local_addr().unwrap().ip())),
                Err(_) => (),
            };
        }
        Err(_) => (),
    };

    if v4_addr.is_none() && v6_addr.is_none() {
        None
    } else {
        Some(IP {
            v4: v4_addr,
            v6: v6_addr,
        })
    }
}

/// Get IP address from Network Status
pub fn get_ip_via_nic() -> Vec<(String, IP)> {
    // v4 v6 v6-temp
    // the define of v6-temp: https://tools.ietf.org/html/rfc4941
    let interfaces = pnet::datalink::interfaces();
    let mut ip_list = Vec::new();

    debug!("Interfaces: {:?}", interfaces);

    for interface in interfaces {
        let mut ip = IP { v4: None, v6: None };
        for ip_addr in interface.ips {
            match ip_addr.ip() {
                IpAddr::V4(_) => {
                    ip.v4 = Some(ip_addr);
                }
                IpAddr::V6(_) => {
                    ip.v6 = Some(ip_addr);
                }
            }
        }
        ip_list.push((interface.name, ip));
    }

    ip_list
}
