# Aliyun-DDNS
This is a simple Rust program to update the DNS record of a domain hosted on Aliyun. It is designed to be run as a Service.

Note: Now it only supports monitoring one type of ip addr (ipv4 or ipv6) at a time. If you want to monitor both, you need to run two instances of this program. In the future, I may add the feature to monitor multi ip addr and corresponding it with multi DNS record at the same time.

## Future Plan
- [x] Support monitoring both ipv4 and ipv6 at the same time.
- [x] Support paring multi-DNS records with specified ip.
- [x] Support monitoring ip by NIC.
- [ ] Support for multi-platform (Arm, Windows, etc.).