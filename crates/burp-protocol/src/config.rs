use std::time::Duration;

pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_CALL_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_QUEUE_CAPACITY: usize = 64;

#[derive(Clone, Debug)]
pub struct ClientTlsConfig {
    pub ca_certificate: Vec<u8>,
    pub client_certificate: Vec<u8>,
    pub client_private_key: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct BurpClientConfig {
    pub endpoint: String,
    pub call_timeout: Duration,
    pub queue_capacity: usize,
    pub max_message_bytes: usize,
    pub tls: Option<ClientTlsConfig>,
}

impl Default for BurpClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:9877".to_owned(),
            call_timeout: DEFAULT_CALL_TIMEOUT,
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            tls: None,
        }
    }
}
