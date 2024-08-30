/// DNS service provider: Aliyun
///     https://help.aliyun.com/zh/dns/api-alidns-2015-01-09-overview
///     Note: Using Signature Method V3
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, RequestBuilder,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{util::random_signature_nonce, GLOBAL_CONFIG};

use super::request_auth::{generate_authorization_header, generate_hashed_request_payload};

static HOST: &str = "alidns.cn-shanghai.aliyuncs.com";
static API_VERSION: &str = "2015-01-09";

#[derive(Deserialize, Serialize, Debug)]
pub struct DnsRecordList {
    #[serde(rename = "TotalCount")]
    total_count: i64,
    #[serde(rename = "PageSize")]
    page_size: i64,
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "DomainRecords")]
    domain_records: DnsRecordListWrapper,
    #[serde(rename = "PageNumber")]
    page_number: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DnsRecordListWrapper {
    #[serde(rename = "Record")]
    record: Vec<DnsRecord>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DnsRecord {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Type")]
    record_type: String,
    #[serde(rename = "Remark")]
    remark: Option<String>,
    #[serde(rename = "TTL")]
    ttl: i64,
    #[serde(rename = "RecordId")]
    record_id: String,
    #[serde(rename = "Priority")]
    priority: Option<i64>,
    #[serde(rename = "RR")]
    rr: String,
    #[serde(rename = "DomainName")]
    domain_name: String,
    #[serde(rename = "Weight")]
    weight: i32,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "Line")]
    line: String,
    #[serde(rename = "Locked")]
    locked: bool,
    #[serde(rename = "CreateTimestamp")]
    create_timestamp: i64,
    #[serde(rename = "UpdateTimestamp")]
    update_timestamp: i64,
}

#[derive(Deserialize, Serialize, Debug)]
struct OperationResult {
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "RecordId")]
    record_id: String,
}

pub struct AliyunDnsOperate {
    access_key_id: String,
    access_key_secret: String,
    client: Client,
}

impl AliyunDnsOperate {
    pub fn new() -> AliyunDnsOperate {
        AliyunDnsOperate {
            access_key_id: GLOBAL_CONFIG.1.auth.auth_id.clone(),
            access_key_secret: GLOBAL_CONFIG.1.auth.auth_token.clone(),
            client: Client::new(),
        }
    }

    pub async fn update_dns_record(
        &self,
        new_ip: &String,
        record_type: &String,
        hostname: &String,
    ) -> Result<()> {
        let list = self.get_dns_record_list(hostname).await?;

        // 获取目标解析记录的ID
        let record_id: String = {
            let mut ret = None;
            for record in list.domain_records.record {
                if record.rr == *hostname && record.record_type == *record_type {
                    ret = Some(record.record_id);
                    break;
                }
            }
            // 如果没有找到对应的解析记录，则返回错误
            if ret.is_none() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "No record with type {} found for hostname: {}",
                        record_type, hostname
                    ),
                ));
            }
            ret.unwrap()
        };

        // 构造请求体
        let method = "GET";
        let action = "UpdateDomainRecord";

        // 请求参数
        let mut query: HashMap<&str, String> = HashMap::new();
        query.insert("RecordId", record_id);
        query.insert("RR", hostname.clone());
        query.insert("Type", record_type.to_string());
        query.insert("Value", new_ip.clone());

        // 请求头
        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert("x-acs-action", HeaderValue::from_static(action));

        // 生成请求并发送
        let result = self
            .generate_authed_request(method, &query, &headers, None)
            .send()
            .await;

        // 解析JSON返回结果
        match result {
            Ok(response) => {
                let text = response.text().await.unwrap();
                debug!("Response: {}", text);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to update DNS record: {}", e);
                Err(Error::new(ErrorKind::Other, e.to_string()))
            }
        }
    }

    /// 获取目标域名和具有类似主机记录值解析记录列表
    pub async fn get_dns_record_list(&self, hostname: &String) -> Result<DnsRecordList> {
        let method = "GET";
        let action = "DescribeDomainRecords";
        let domain = GLOBAL_CONFIG.1.domain_name.clone();

        // 请求参数
        let mut query: HashMap<&str, String> = HashMap::new();
        query.insert("DomainName", domain);
        query.insert("RRKeyWord", hostname.clone());

        // 请求头
        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert("x-acs-action", HeaderValue::from_static(action));

        // 生成请求并发送
        let result = self
            .generate_authed_request(method, &query, &headers, None)
            .send()
            .await;

        // 解析JSON返回结果
        match result {
            Ok(response) => {
                let text = response.text().await.unwrap();
                debug!("Response: {}", text);
                let dns_record_list: DnsRecordList = match serde_json::from_str(&text) {
                    Ok(list) => list,
                    Err(e) => {
                        eprintln!("Something wrong happened when parsing the response: {}", e);
                        return Err(Error::new(ErrorKind::Other, e.to_string()));
                    }
                };
                Ok(dns_record_list)
            }
            Err(e) => {
                eprintln!("Failed to get DNS record list: {}", e);
                Err(Error::new(ErrorKind::Other, e.to_string()))
            }
        }
    }

    /// 加入必要的请求头
    fn add_indispensable_headers(headers: &mut HeaderMap, payload: Option<&String>) {
        headers.insert("x-acs-version", HeaderValue::from_static(API_VERSION));
        headers.insert(
            "x-acs-signature-nonce",
            HeaderValue::from_str(random_signature_nonce().as_str()).unwrap(),
        );
        headers.insert(
            "x-acs-date",
            // 格式：yyyy-MM-ddTHH:mm:ssZ
            HeaderValue::from_str(
                &format!("{}", chrono::Utc::now().format("%G-%m-%dT%H:%M:%SZ")).as_str(),
            )
            .unwrap(),
        );
        headers.insert("host", HeaderValue::from_static(HOST));
        headers.insert(
            "x-acs-content-sha256",
            HeaderValue::from_str(generate_hashed_request_payload(payload).as_str()).unwrap(),
        );
    }

    fn generate_authed_request(
        &self,
        method: &str,
        query: &HashMap<&str, String>,
        headers: &HeaderMap,
        payload: Option<&String>,
    ) -> RequestBuilder {
        // 加入必要的请求头
        let mut headers = headers.clone();
        Self::add_indispensable_headers(&mut headers, payload);

        // 生成认证头
        let auth_head = generate_authorization_header(
            &self.access_key_id,
            &self.access_key_secret,
            method,
            "/",
            &query,
            &headers,
            payload,
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(auth_head.as_str()).unwrap(),
        );

        match method {
            "GET" => self
                .client
                .get(&format!("https://{}", HOST))
                .query(query)
                .headers(headers),
            "POST" => self
                .client
                .post(&format!("https://{}", HOST))
                .query(query)
                .headers(headers)
                .body(payload.unwrap().clone()),
            _ => unreachable!("Unsupported HTTP method"),
        }
    }
}
