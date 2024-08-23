use std::collections::HashMap;

use reqwest::header::HeaderMap;
use tracing::debug;

static ALGORITHM: &str = "ACS3-HMAC-SHA256";

/// 按照rfc3986规则对字符串进行编码
fn url_encode(input: &str) -> String {
    // 正则表达式，匹配不需要编码的字符
    let re = regex::Regex::new(r"[A-Za-z0-9\-\._~]").unwrap();
    let mut encoded = String::new();

    for c in input.chars() {
        if re.is_match(&c.to_string()) {
            // 字符A~Z、a~z、0~9以及字符'-'、'_'、'.'、'~'不编码。
            encoded.push(c);
        } else {
            // 其他字符编码成%加字符对应ASCII码的16进制。
            encoded.push_str(&format!("%{:02X}", c as u8));
        }
    }

    encoded
}

/// 构造规范化查询字符串
/// 1. 将查询字符串中的参数按照参数名的字符代码升序排列，具有重复名称的参数应按值进行排序。
/// 2. 使用UTF-8字符集按照RFC3986的规则对每个参数的参数名和参数值分别进行URI编码，具体规则与上一节中的CanonicalURI编码规则相同。
/// 3. 使用等号（=）连接编码后的请求参数名和参数值，对于没有值的参数使用空字符串。
/// 4. 多个请求参数之间使用与号（&）连接。
fn generate_canonical_query_string(query: &HashMap<&str, String>) -> String {
    let mut canonical_query_string = String::new();

    let mut query_vec: Vec<(&&str, &String)> = query.iter().collect();
    // 按照参数名的字符代码升序排列（具有重复名称的参数按值进行排序）
    query_vec.sort_by(|a, b| {
        if a.0 == b.0 {
            a.1.cmp(b.1)
        } else {
            a.0.cmp(b.0)
        }
    });

    // 使用等号（=）连接编码后的请求参数名和参数值，对于没有值的参数使用空字符串
    for (key, value) in query_vec.iter() {
        canonical_query_string.push_str(&format!("{}={}&", url_encode(key), url_encode(value)));
    }

    // 去掉最后一个&符号
    canonical_query_string.pop();

    canonical_query_string
}

/// 构造规范化请求头字符串和已签名消息头列表
///
/// 规范化消息头（CanonicalHeaderEntry）的格式如下：
/// 1. 将所有需要签名的参数的名称转换为小写。
/// 2. 将所有参数按照参数名称的字符顺序以升序排列。
/// 3. 将参数的值除去首尾空格。对于有多个值的参数，将多个值分别除去首尾空格后按值升序排列，然后用逗号（,）连接。
/// 4. 将步骤2、3的结果以英文冒号（:）连接，并在尾部添加换行符，组成一个规范化消息头（CanonicalHeaderEntry）。
/// 5. 如果没有需要签名的消息头，使用空字符串作为规范化消息头列表。
///
/// 已签名消息头列表（SignedHeaders）的格式如下：
/// 1. 将CanonicalHeaders中包含的请求头的名称转为小写。
/// 2. 多个请求头名称（小写）按首字母升序排列并以英文分号（;）分隔，例如 content-type;host;x-acs-date。
fn generate_canonical_header_and_signed_headers_string(header: &HeaderMap) -> (String, String) {
    let mut canonical_header_string = String::new();
    let mut signed_headers_string = String::new();

    let mut header_vec: Vec<_> = header.iter().collect();

    // 找出需要签名的参数: 以x-acs-为前缀、host、content-type
    // 删除其它参数
    header_vec.retain(|(key, _)| {
        let key = key.as_str();
        key.starts_with("x-acs-") || key == "host" || key == "content-type"
    });

    // 将所有参数按照参数名称的字符顺序以升序排列
    header_vec.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));

    {
        // 将步骤2、3的结果以英文冒号（:）连接，并在尾部添加换行符，组成一个规范化消息头（CanonicalHeaderEntry）
        for (key, value) in header_vec.iter() {
            canonical_header_string.push_str(&format!("{}:{}\n", key, value.to_str().unwrap()));
        }
    }

    {
        // 多个请求头名称以英文分号（;）分隔
        for (key, _) in header_vec.iter() {
            signed_headers_string.push_str(&format!("{};", key));
        }
        // 去掉最后一个分号
        signed_headers_string.pop();
    }

    (canonical_header_string, signed_headers_string)
}

/// 构造载荷哈希值
pub fn generate_hashed_request_payload(json_payload: Option<&String>) -> String {
    if json_payload.is_none() {
        // 如果没有请求体，则直接返回空字符串
        String::from("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    } else {
        // 使用SHA256算法计算请求体的哈希值
        let sha = ring::digest::digest(&ring::digest::SHA256, json_payload.unwrap().as_bytes());
        // 将哈希值转换为16进制字符串
        sha.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// 构造待签名字符串
/// StringToSign =
///     SignatureAlgorithm + '\n' +   //签名算法
///     HashedCanonicalRequest        //规范化请求的哈希值
fn generate_string_to_sign(canonical_request: &String) -> String {
    // 使用SHA256算法计算请求体的哈希值
    let sha = ring::digest::digest(&ring::digest::SHA256, canonical_request.as_bytes());
    // 将哈希值转换为16进制字符串
    let sha: String = sha.as_ref().iter().map(|b| format!("{:02x}", b)).collect();
    // 1. 签名算法
    // 2. 规范化请求的哈希值
    format!("{}\n{}", ALGORITHM, sha)
}

/// 计算签名
/// Signature = Base16( HMAC-SHA256( AccessKeySecret, StringToSign ) )
fn calculate_signature(string_to_sign: &String, secret: &String) -> String {
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret.as_bytes());
    let signature = ring::hmac::sign(&key, string_to_sign.as_bytes());
    signature
        .as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// 构造认证请求头
/// Authorization:<SignatureAlgorithm> Credential=<AccessKeyId>,SignedHeaders=<SignedHeaders>,Signature=<Signature>
pub fn generate_authorization_header(
    access_key_id: &String,
    access_key_secret: &String,
    http_method: &str,
    uri: &str,
    query: &HashMap<&str, String>,
    headers: &HeaderMap,
    json_payload: Option<&String>,
) -> String {
    // 1. 构造规范化请求
    let canonical_query = generate_canonical_query_string(query);
    let (canonical_header, signed_headers) =
        generate_canonical_header_and_signed_headers_string(headers);
    let hashed_request_payload = generate_hashed_request_payload(json_payload);
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        http_method, uri, canonical_query, canonical_header, signed_headers, hashed_request_payload
    );
    // 2. 构造待签名字符串
    let string_to_sign = generate_string_to_sign(&canonical_request);
    // 3. 计算签名
    let signature = calculate_signature(&string_to_sign, access_key_secret);

    debug!("CanonicalRequest:\n{}\n-END-", canonical_request);
    debug!("StringToSign:\n{}\n-END-", string_to_sign);

    // 4. 构造请求头
    format!(
        "{} Credential={},SignedHeaders={},Signature={}",
        ALGORITHM, access_key_id, signed_headers, signature
    )
}
