#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PageRequest {
    pub limit: u32,
    pub cursor: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProxyHistoryQuery {
    pub page: PageRequest,
    pub url_filter: String,
    pub method_filter: String,
    pub status_filter: Option<u32>,
    pub has_notes: bool,
    pub color: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PingInfo {
    pub server: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfo {
    pub capabilities: Vec<String>,
    pub max_message_bytes: u32,
    pub max_response_bytes: u32,
    pub max_page_size: u32,
    pub max_concurrent_calls_per_connection: u32,
    pub max_rpc_timeout_seconds: u32,
}
