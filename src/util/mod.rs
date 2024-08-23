pub mod log_collector;

/// 生成一个32位随机字符串，由小写字母和数字组成
pub fn random_signature_nonce() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let nonce: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    nonce
}
