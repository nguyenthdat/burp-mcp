//! Typed, bounded Rust client for the loopback Burp RPC seam.

mod mapping;
mod model;
pub use model::{PageRequest, PingInfo, ProxyHistoryQuery, ServerInfo};
#[doc(hidden)]
pub mod protocol {
    tonic::include_proto!("burp.v1");
}
#[cfg(feature = "interop")]
pub use protocol as interop_proto;
use protocol as proto;

use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Status};

mod config;
pub use config::{BurpClientConfig, DEFAULT_CALL_TIMEOUT, DEFAULT_MAX_MESSAGE_BYTES, DEFAULT_QUEUE_CAPACITY};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Burp client queue is full; retry after an in-flight call completes")]
    QueueFull,
    #[error("Burp client queue is closed")]
    QueueClosed,
    #[error("Burp client request was cancelled")]
    ResponseCancelled,
    #[error("Burp RPC request failed: {0}")]
    Rpc(#[from] Status),
    #[error("invalid Burp client configuration: {0}")]
    InvalidConfig(&'static str),
}

enum Command {
    Ping {
        request: proto::PingRequest,
        response: oneshot::Sender<Result<proto::PingResponse, ClientError>>,
    },
    EchoBytes {
        request: proto::EchoBytesRequest,
        response: oneshot::Sender<Result<proto::EchoBytesResponse, ClientError>>,
    },
    ProxyHistory {
        request: proto::ProxyHistoryRequest,
        response: oneshot::Sender<Result<proto::ProxyHistoryResponse, ClientError>>,
    },
    ProxyDetail {
        request: proto::ProxyDetailRequest,
        response: oneshot::Sender<Result<proto::ProxyDetailResponse, ClientError>>,
    },
    SitemapSnapshot {
        request: proto::SitemapSnapshotRequest,
        response: oneshot::Sender<Result<proto::SitemapSnapshotResponse, ClientError>>,
    },
    TargetInfo {
        request: proto::TargetInfoRequest,
        response: oneshot::Sender<Result<proto::TargetInfoResponse, ClientError>>,
    },
    ScopeCheck {
        request: proto::ScopeCheckRequest,
        response: oneshot::Sender<Result<proto::ScopeCheckResponse, ClientError>>,
    },
    ScanIssues {
        request: proto::ScanIssuesRequest,
        response: oneshot::Sender<Result<proto::ScanIssuesResponse, ClientError>>,
    },
    CookieJar {
        request: proto::CookieJarRequest,
        response: oneshot::Sender<Result<proto::CookieJarResponse, ClientError>>,
    },
    SetCookie {
        request: proto::SetCookieRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    AddIssue {
        request: proto::AddIssueRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ScanIssueDetail {
        request: proto::ScanIssueDetailRequest,
        response: oneshot::Sender<Result<proto::ScanIssueEntry, ClientError>>,
    },
    GenerateScannerReport {
        request: proto::GenerateScannerReportRequest,
        response: oneshot::Sender<Result<proto::GenerateScannerReportResponse, ClientError>>,
    },
    InterceptState {
        request: proto::InterceptStateRequest,
        response: oneshot::Sender<Result<proto::InterceptStateResponse, ClientError>>,
    },
    ProxyInterceptConfig {
        request: proto::ProxyInterceptConfigRequest,
        response: oneshot::Sender<Result<proto::ProxyInterceptConfigResponse, ClientError>>,
    },
    ProxyWebSocketHistory {
        request: proto::ProxyWebSocketHistoryRequest,
        response: oneshot::Sender<Result<proto::ProxyWebSocketHistoryResponse, ClientError>>,
    },
    SendToIntruder {
        request: proto::SendToIntruderRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    RegisterPayloadProcessor {
        request: proto::RegisterPayloadProcessorRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ListPayloadProcessors {
        request: proto::ListPayloadProcessorsRequest,
        response: oneshot::Sender<Result<proto::ListPayloadProcessorsResponse, ClientError>>,
    },
    RemovePayloadProcessor {
        request: proto::RemovePayloadProcessorRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    RegisterPayloadGenerator {
        request: proto::RegisterPayloadGeneratorRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ListPayloadGenerators {
        request: proto::ListPayloadGeneratorsRequest,
        response: oneshot::Sender<Result<proto::ListPayloadGeneratorsResponse, ClientError>>,
    },
    RemovePayloadGenerator {
        request: proto::RemovePayloadGeneratorRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    CreatePayloadList {
        request: proto::CreatePayloadListRequest,
        response: oneshot::Sender<Result<proto::PayloadListEntry, ClientError>>,
    },
    ImportPayloadList {
        request: proto::ImportPayloadListRequest,
        response: oneshot::Sender<Result<proto::PayloadListEntry, ClientError>>,
    },
    ListPayloadLists {
        request: proto::ListPayloadListsRequest,
        response: oneshot::Sender<Result<proto::ListPayloadListsResponse, ClientError>>,
    },
    GetPayloadList {
        request: proto::GetPayloadListRequest,
        response: oneshot::Sender<Result<proto::GetPayloadListResponse, ClientError>>,
    },
    UpdatePayloadList {
        request: proto::UpdatePayloadListRequest,
        response: oneshot::Sender<Result<proto::PayloadListEntry, ClientError>>,
    },
    DeletePayloadList {
        request: proto::DeletePayloadListRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ExtensionInfo {
        request: proto::ExtensionInfoRequest,
        response: oneshot::Sender<Result<proto::ExtensionInfoResponse, ClientError>>,
    },
    ServerInfo {
        request: proto::ServerInfoRequest,
        response: oneshot::Sender<Result<proto::ServerInfoResponse, ClientError>>,
    },
    SendRequest {
        request: proto::SendRequestRequest,
        response: oneshot::Sender<Result<proto::SendRequestResponse, ClientError>>,
    },
    SendRequests {
        request: proto::SendRequestsRequest,
        response: oneshot::Sender<Result<proto::SendRequestsResponse, ClientError>>,
    },
    SendToRepeater {
        request: proto::SendToRepeaterRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    SetHighlight {
        request: proto::SetHighlightRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    SetNote {
        request: proto::SetNoteRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    MutateScope {
        request: proto::MutateScopeRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    InspectConfig {
        request: proto::ExportConfigRequest,
        response: oneshot::Sender<Result<proto::InspectConfigResponse, ClientError>>,
    },
    ExportConfig {
        request: proto::ExportConfigRequest,
        response: oneshot::Sender<Result<proto::ConfigResponse, ClientError>>,
    },
    ImportConfig {
        request: proto::ImportConfigRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    RegisterHttpHandler {
        request: proto::RegisterHttpHandlerRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ClearHttpHandler {
        request: proto::ClearHttpHandlerRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    RegisterProxyRule {
        request: proto::RegisterProxyRuleRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ClearProxyRules {
        request: proto::ClearProxyRulesRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    CreateSessionRule {
        request: proto::CreateSessionRuleRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ListProxyRules {
        request: proto::ListProxyRulesRequest,
        response: oneshot::Sender<Result<proto::ListProxyRulesResponse, ClientError>>,
    },
    ListSessionRules {
        request: proto::ListSessionRulesRequest,
        response: oneshot::Sender<Result<proto::ListSessionRulesResponse, ClientError>>,
    },
    RemoveSessionRules {
        request: proto::RemoveSessionRulesRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    CreateMacro {
        request: proto::CreateMacroRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ListMacros {
        request: proto::ListMacrosRequest,
        response: oneshot::Sender<Result<proto::ListMacrosResponse, ClientError>>,
    },
    RunMacro {
        request: proto::RunMacroRequest,
        response: oneshot::Sender<Result<proto::RunMacroResponse, ClientError>>,
    },
    RemoveMacro {
        request: proto::RemoveMacroRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    StartConcurrentRequestCheck {
        request: proto::StartConcurrentRequestCheckRequest,
        response: oneshot::Sender<Result<proto::JobStatusResponse, ClientError>>,
    },
    StartBoundedInputMatrix {
        request: proto::StartBoundedInputMatrixRequest,
        response: oneshot::Sender<Result<proto::JobStatusResponse, ClientError>>,
    },
    StartCrawl {
        request: proto::StartCrawlRequest,
        response: oneshot::Sender<Result<proto::JobStatusResponse, ClientError>>,
    },
    StartAudit {
        request: proto::StartAuditRequest,
        response: oneshot::Sender<Result<proto::JobStatusResponse, ClientError>>,
    },
    GetJobStatus {
        request: proto::GetJobStatusRequest,
        response: oneshot::Sender<Result<proto::JobStatusResponse, ClientError>>,
    },
    CancelJob {
        request: proto::CancelJobRequest,
        response: oneshot::Sender<Result<proto::JobStatusResponse, ClientError>>,
    },
    GetJobResult {
        request: proto::GetJobResultRequest,
        response: oneshot::Sender<Result<proto::JobResultResponse, ClientError>>,
    },
    GenerateCollaboratorPayloads {
        request: proto::GenerateCollaboratorPayloadsRequest,
        response: oneshot::Sender<Result<proto::GenerateCollaboratorPayloadsResponse, ClientError>>,
    },
    PollCollaboratorInteractions {
        request: proto::PollCollaboratorInteractionsRequest,
        response: oneshot::Sender<Result<proto::PollCollaboratorInteractionsResponse, ClientError>>,
    },
    CreateWebSocket {
        request: proto::CreateWebSocketRequest,
        response: oneshot::Sender<Result<proto::CreateWebSocketResponse, ClientError>>,
    },
    SendWebSocketText {
        request: proto::SendWebSocketTextRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    SendWebSocketBinary {
        request: proto::SendWebSocketBinaryRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    CloseWebSocket {
        request: proto::CloseWebSocketRequest,
        response: oneshot::Sender<Result<proto::ActionResponse, ClientError>>,
    },
    ListWebSockets {
        request: proto::ListWebSocketsRequest,
        response: oneshot::Sender<Result<proto::ListWebSocketsResponse, ClientError>>,
    },
    ManagedWebSocketHistory {
        request: proto::ManagedWebSocketHistoryRequest,
        response: oneshot::Sender<Result<proto::ManagedWebSocketHistoryResponse, ClientError>>,
    },
    ImportBambda {
        request: proto::ImportBambdaRequest,
        response: oneshot::Sender<Result<proto::ScriptImportResponse, ClientError>>,
    },
    ImportBCheck {
        request: proto::ImportBCheckRequest,
        response: oneshot::Sender<Result<proto::ScriptImportResponse, ClientError>>,
    },
}

#[derive(Clone)]
pub struct BurpClient {
    sender: mpsc::Sender<Command>,
}

impl BurpClient {
    pub async fn ping(
        &self,
        request: proto::PingRequest,
    ) -> Result<proto::PingResponse, ClientError> {
        self.send(|response| Command::Ping { request, response })
            .await
    }
    pub async fn probe_ping(&self, client: String) -> Result<PingInfo, ClientError> {
        let response = self.ping(proto::PingRequest { client }).await?;
        Ok(PingInfo {
            server: response.server,
            version: response.version,
        })
    }

    pub async fn probe_echo(&self, payload: Vec<u8>) -> Result<Vec<u8>, ClientError> {
        self.echo_bytes(proto::EchoBytesRequest {
            payload,
            delay_millis: 0,
        })
        .await
        .map(|response| response.payload)
    }

    pub async fn probe_server_info(&self) -> Result<ServerInfo, ClientError> {
        let response = self.server_info(proto::ServerInfoRequest {}).await?;
        Ok(ServerInfo {
            capabilities: response.capabilities,
            max_message_bytes: response.max_message_bytes,
            max_response_bytes: response.max_response_bytes,
            max_page_size: response.max_page_size,
            max_concurrent_calls_per_connection: response.max_concurrent_calls_per_connection,
            max_rpc_timeout_seconds: response.max_rpc_timeout_seconds,
        })
    }
    pub async fn probe_proxy_history(&self, query: ProxyHistoryQuery) -> Result<(), ClientError> {
        self.proxy_history(mapping::proxy_history_request(query))
            .await
            .map(|_| ())
    }

    pub async fn echo_bytes(
        &self,
        request: proto::EchoBytesRequest,
    ) -> Result<proto::EchoBytesResponse, ClientError> {
        self.send(|response| Command::EchoBytes { request, response })
            .await
    }

    pub async fn proxy_history(
        &self,
        request: proto::ProxyHistoryRequest,
    ) -> Result<proto::ProxyHistoryResponse, ClientError> {
        self.send(|response| Command::ProxyHistory { request, response })
            .await
    }
    pub async fn proxy_detail(
        &self,
        request: proto::ProxyDetailRequest,
    ) -> Result<proto::ProxyDetailResponse, ClientError> {
        self.send(|response| Command::ProxyDetail { request, response })
            .await
    }
    pub async fn sitemap_snapshot(
        &self,
        request: proto::SitemapSnapshotRequest,
    ) -> Result<proto::SitemapSnapshotResponse, ClientError> {
        self.send(|response| Command::SitemapSnapshot { request, response })
            .await
    }
    pub async fn target_info(
        &self,
        request: proto::TargetInfoRequest,
    ) -> Result<proto::TargetInfoResponse, ClientError> {
        self.send(|response| Command::TargetInfo { request, response })
            .await
    }

    pub async fn scope_check(
        &self,
        request: proto::ScopeCheckRequest,
    ) -> Result<proto::ScopeCheckResponse, ClientError> {
        self.send(|response| Command::ScopeCheck { request, response })
            .await
    }
    pub async fn scan_issues(
        &self,
        request: proto::ScanIssuesRequest,
    ) -> Result<proto::ScanIssuesResponse, ClientError> {
        self.send(|response| Command::ScanIssues { request, response })
            .await
    }
    pub async fn cookie_jar(
        &self,
        request: proto::CookieJarRequest,
    ) -> Result<proto::CookieJarResponse, ClientError> {
        self.send(|response| Command::CookieJar { request, response })
            .await
    }

    pub async fn set_cookie(
        &self,
        request: proto::SetCookieRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SetCookie { request, response })
            .await
    }

    pub async fn add_issue(
        &self,
        request: proto::AddIssueRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::AddIssue { request, response })
            .await
    }

    pub async fn scan_issue_detail(
        &self,
        request: proto::ScanIssueDetailRequest,
    ) -> Result<proto::ScanIssueEntry, ClientError> {
        self.send(|response| Command::ScanIssueDetail { request, response })
            .await
    }
    pub async fn generate_scanner_report(
        &self,
        request: proto::GenerateScannerReportRequest,
    ) -> Result<proto::GenerateScannerReportResponse, ClientError> {
        self.send(|response| Command::GenerateScannerReport { request, response })
            .await
    }

    pub async fn intercept_state(
        &self,
        request: proto::InterceptStateRequest,
    ) -> Result<proto::InterceptStateResponse, ClientError> {
        self.send(|response| Command::InterceptState { request, response })
            .await
    }
    pub async fn proxy_intercept_config(
        &self,
        request: proto::ProxyInterceptConfigRequest,
    ) -> Result<proto::ProxyInterceptConfigResponse, ClientError> {
        self.send(|response| Command::ProxyInterceptConfig { request, response })
            .await
    }

    pub async fn proxy_websocket_history(
        &self,
        request: proto::ProxyWebSocketHistoryRequest,
    ) -> Result<proto::ProxyWebSocketHistoryResponse, ClientError> {
        self.send(|response| Command::ProxyWebSocketHistory { request, response })
            .await
    }

    pub async fn send_to_intruder(
        &self,
        request: proto::SendToIntruderRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SendToIntruder { request, response })
            .await
    }
    pub async fn register_payload_processor(
        &self,
        request: proto::RegisterPayloadProcessorRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RegisterPayloadProcessor { request, response })
            .await
    }

    pub async fn list_payload_processors(
        &self,
        request: proto::ListPayloadProcessorsRequest,
    ) -> Result<proto::ListPayloadProcessorsResponse, ClientError> {
        self.send(|response| Command::ListPayloadProcessors { request, response })
            .await
    }

    pub async fn remove_payload_processor(
        &self,
        request: proto::RemovePayloadProcessorRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RemovePayloadProcessor { request, response })
            .await
    }

    pub async fn register_payload_generator(
        &self,
        request: proto::RegisterPayloadGeneratorRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RegisterPayloadGenerator { request, response })
            .await
    }

    pub async fn list_payload_generators(
        &self,
        request: proto::ListPayloadGeneratorsRequest,
    ) -> Result<proto::ListPayloadGeneratorsResponse, ClientError> {
        self.send(|response| Command::ListPayloadGenerators { request, response })
            .await
    }

    pub async fn remove_payload_generator(
        &self,
        request: proto::RemovePayloadGeneratorRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RemovePayloadGenerator { request, response })
            .await
    }
    pub async fn create_payload_list(
        &self,
        request: proto::CreatePayloadListRequest,
    ) -> Result<proto::PayloadListEntry, ClientError> {
        self.send(|response| Command::CreatePayloadList { request, response })
            .await
    }
    pub async fn import_payload_list(
        &self,
        request: proto::ImportPayloadListRequest,
    ) -> Result<proto::PayloadListEntry, ClientError> {
        self.send(|response| Command::ImportPayloadList { request, response })
            .await
    }
    pub async fn list_payload_lists(
        &self,
        request: proto::ListPayloadListsRequest,
    ) -> Result<proto::ListPayloadListsResponse, ClientError> {
        self.send(|response| Command::ListPayloadLists { request, response })
            .await
    }
    pub async fn get_payload_list(
        &self,
        request: proto::GetPayloadListRequest,
    ) -> Result<proto::GetPayloadListResponse, ClientError> {
        self.send(|response| Command::GetPayloadList { request, response })
            .await
    }
    pub async fn update_payload_list(
        &self,
        request: proto::UpdatePayloadListRequest,
    ) -> Result<proto::PayloadListEntry, ClientError> {
        self.send(|response| Command::UpdatePayloadList { request, response })
            .await
    }
    pub async fn delete_payload_list(
        &self,
        request: proto::DeletePayloadListRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::DeletePayloadList { request, response })
            .await
    }

    pub async fn extension_info(
        &self,
        request: proto::ExtensionInfoRequest,
    ) -> Result<proto::ExtensionInfoResponse, ClientError> {
        self.send(|response| Command::ExtensionInfo { request, response })
            .await
    }

    pub async fn server_info(
        &self,
        request: proto::ServerInfoRequest,
    ) -> Result<proto::ServerInfoResponse, ClientError> {
        self.send(|response| Command::ServerInfo { request, response })
            .await
    }

    pub async fn send_request(
        &self,
        request: proto::SendRequestRequest,
    ) -> Result<proto::SendRequestResponse, ClientError> {
        self.send(|response| Command::SendRequest { request, response })
            .await
    }

    pub async fn send_requests(
        &self,
        request: proto::SendRequestsRequest,
    ) -> Result<proto::SendRequestsResponse, ClientError> {
        self.send(|response| Command::SendRequests { request, response })
            .await
    }

    pub async fn send_to_repeater(
        &self,
        request: proto::SendToRepeaterRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SendToRepeater { request, response })
            .await
    }
    pub async fn set_highlight(
        &self,
        request: proto::SetHighlightRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SetHighlight { request, response })
            .await
    }

    pub async fn set_note(
        &self,
        request: proto::SetNoteRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SetNote { request, response })
            .await
    }

    pub async fn mutate_scope(
        &self,
        request: proto::MutateScopeRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::MutateScope { request, response })
            .await
    }
    pub async fn inspect_config(
        &self,
        request: proto::ExportConfigRequest,
    ) -> Result<proto::InspectConfigResponse, ClientError> {
        self.send(|response| Command::InspectConfig { request, response })
            .await
    }
    pub async fn export_config(
        &self,
        request: proto::ExportConfigRequest,
    ) -> Result<proto::ConfigResponse, ClientError> {
        self.send(|response| Command::ExportConfig { request, response })
            .await
    }

    pub async fn import_config(
        &self,
        request: proto::ImportConfigRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::ImportConfig { request, response })
            .await
    }
    pub async fn register_http_handler(
        &self,
        request: proto::RegisterHttpHandlerRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RegisterHttpHandler { request, response })
            .await
    }

    pub async fn clear_http_handler(
        &self,
        request: proto::ClearHttpHandlerRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::ClearHttpHandler { request, response })
            .await
    }
    pub async fn register_proxy_rule(
        &self,
        request: proto::RegisterProxyRuleRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RegisterProxyRule { request, response })
            .await
    }

    pub async fn clear_proxy_rules(
        &self,
        request: proto::ClearProxyRulesRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::ClearProxyRules { request, response })
            .await
    }
    pub async fn list_proxy_rules(
        &self,
        request: proto::ListProxyRulesRequest,
    ) -> Result<proto::ListProxyRulesResponse, ClientError> {
        self.send(|response| Command::ListProxyRules { request, response })
            .await
    }

    pub async fn create_session_rule(
        &self,
        request: proto::CreateSessionRuleRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::CreateSessionRule { request, response })
            .await
    }

    pub async fn list_session_rules(
        &self,
        request: proto::ListSessionRulesRequest,
    ) -> Result<proto::ListSessionRulesResponse, ClientError> {
        self.send(|response| Command::ListSessionRules { request, response })
            .await
    }

    pub async fn remove_session_rules(
        &self,
        request: proto::RemoveSessionRulesRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RemoveSessionRules { request, response })
            .await
    }

    pub async fn create_macro(
        &self,
        request: proto::CreateMacroRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::CreateMacro { request, response })
            .await
    }

    pub async fn list_macros(
        &self,
        request: proto::ListMacrosRequest,
    ) -> Result<proto::ListMacrosResponse, ClientError> {
        self.send(|response| Command::ListMacros { request, response })
            .await
    }

    pub async fn run_macro(
        &self,
        request: proto::RunMacroRequest,
    ) -> Result<proto::RunMacroResponse, ClientError> {
        self.send(|response| Command::RunMacro { request, response })
            .await
    }

    pub async fn remove_macro(
        &self,
        request: proto::RemoveMacroRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::RemoveMacro { request, response })
            .await
    }

    pub async fn start_concurrent_request_check(
        &self,
        request: proto::StartConcurrentRequestCheckRequest,
    ) -> Result<proto::JobStatusResponse, ClientError> {
        self.send(|response| Command::StartConcurrentRequestCheck { request, response })
            .await
    }

    pub async fn start_bounded_input_matrix(
        &self,
        request: proto::StartBoundedInputMatrixRequest,
    ) -> Result<proto::JobStatusResponse, ClientError> {
        self.send(|response| Command::StartBoundedInputMatrix { request, response })
            .await
    }

    pub async fn start_crawl(
        &self,
        request: proto::StartCrawlRequest,
    ) -> Result<proto::JobStatusResponse, ClientError> {
        self.send(|response| Command::StartCrawl { request, response })
            .await
    }
    pub async fn start_audit(
        &self,
        request: proto::StartAuditRequest,
    ) -> Result<proto::JobStatusResponse, ClientError> {
        self.send(|response| Command::StartAudit { request, response })
            .await
    }

    pub async fn get_job_status(
        &self,
        request: proto::GetJobStatusRequest,
    ) -> Result<proto::JobStatusResponse, ClientError> {
        self.send(|response| Command::GetJobStatus { request, response })
            .await
    }

    pub async fn cancel_job(
        &self,
        request: proto::CancelJobRequest,
    ) -> Result<proto::JobStatusResponse, ClientError> {
        self.send(|response| Command::CancelJob { request, response })
            .await
    }

    pub async fn get_job_result(
        &self,
        request: proto::GetJobResultRequest,
    ) -> Result<proto::JobResultResponse, ClientError> {
        self.send(|response| Command::GetJobResult { request, response })
            .await
    }
    pub async fn generate_collaborator_payloads(
        &self,
        request: proto::GenerateCollaboratorPayloadsRequest,
    ) -> Result<proto::GenerateCollaboratorPayloadsResponse, ClientError> {
        self.send(|response| Command::GenerateCollaboratorPayloads { request, response })
            .await
    }

    pub async fn poll_collaborator_interactions(
        &self,
        request: proto::PollCollaboratorInteractionsRequest,
    ) -> Result<proto::PollCollaboratorInteractionsResponse, ClientError> {
        self.send(|response| Command::PollCollaboratorInteractions { request, response })
            .await
    }
    pub async fn create_websocket(
        &self,
        request: proto::CreateWebSocketRequest,
    ) -> Result<proto::CreateWebSocketResponse, ClientError> {
        self.send(|response| Command::CreateWebSocket { request, response })
            .await
    }
    pub async fn send_websocket_text(
        &self,
        request: proto::SendWebSocketTextRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SendWebSocketText { request, response })
            .await
    }
    pub async fn send_websocket_binary(
        &self,
        request: proto::SendWebSocketBinaryRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::SendWebSocketBinary { request, response })
            .await
    }
    pub async fn close_websocket(
        &self,
        request: proto::CloseWebSocketRequest,
    ) -> Result<proto::ActionResponse, ClientError> {
        self.send(|response| Command::CloseWebSocket { request, response })
            .await
    }
    pub async fn list_websockets(
        &self,
        request: proto::ListWebSocketsRequest,
    ) -> Result<proto::ListWebSocketsResponse, ClientError> {
        self.send(|response| Command::ListWebSockets { request, response })
            .await
    }

    pub async fn managed_websocket_history(
        &self,
        request: proto::ManagedWebSocketHistoryRequest,
    ) -> Result<proto::ManagedWebSocketHistoryResponse, ClientError> {
        self.send(|response| Command::ManagedWebSocketHistory { request, response })
            .await
    }
    pub async fn import_bambda(
        &self,
        request: proto::ImportBambdaRequest,
    ) -> Result<proto::ScriptImportResponse, ClientError> {
        self.send(|response| Command::ImportBambda { request, response })
            .await
    }

    pub async fn import_bcheck(
        &self,
        request: proto::ImportBCheckRequest,
    ) -> Result<proto::ScriptImportResponse, ClientError> {
        self.send(|response| Command::ImportBCheck { request, response })
            .await
    }

    async fn send<T>(
        &self,
        command: impl FnOnce(oneshot::Sender<Result<T, ClientError>>) -> Command,
    ) -> Result<T, ClientError> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .try_send(command(response))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => ClientError::QueueFull,
                mpsc::error::TrySendError::Closed(_) => ClientError::QueueClosed,
            })?;
        receiver.await.map_err(|_| ClientError::ResponseCancelled)?
    }
}


pub fn spawn_client(config: BurpClientConfig) -> Result<BurpClient, ClientError> {
    if config.queue_capacity == 0 {
        return Err(ClientError::InvalidConfig(
            "queue capacity must be positive",
        ));
    }
    if config.call_timeout == Duration::ZERO {
        return Err(ClientError::InvalidConfig("call timeout must be positive"));
    }
    if config.max_message_bytes == 0 {
        return Err(ClientError::InvalidConfig("message limit must be positive"));
    }
    let (sender, receiver) = mpsc::channel(config.queue_capacity);
    tokio::spawn(run_actor(config, receiver));
    Ok(BurpClient { sender })
}

async fn run_actor(config: BurpClientConfig, mut receiver: mpsc::Receiver<Command>) {
    let mut client: Option<proto::burp_service_client::BurpServiceClient<Channel>> = None;
    while let Some(command) = receiver.recv().await {
        if client.is_none() {
            client = connect(&config).await;
        }
        let Some(current_client) = client.as_mut() else {
            respond_offline(command);
            continue;
        };
        let result = execute(current_client, &config, command).await;
        if result {
            client = None;
        }
    }
}

pub async fn connect_client(
    endpoint: &str,
    timeout: Duration,
    max_message_bytes: usize,
) -> Result<proto::burp_service_client::BurpServiceClient<Channel>, tonic::transport::Error> {
    let channel = Endpoint::from_shared(endpoint.to_owned())?
        .connect_timeout(timeout)
        .timeout(timeout)
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .connect()
        .await?;
    Ok(proto::burp_service_client::BurpServiceClient::new(channel)
        .max_decoding_message_size(max_message_bytes)
        .max_encoding_message_size(max_message_bytes))
}

async fn connect(
    config: &BurpClientConfig,
) -> Option<proto::burp_service_client::BurpServiceClient<Channel>> {
    connect_client(
        &config.endpoint,
        config.call_timeout,
        config.max_message_bytes,
    )
    .await
    .ok()
}

async fn execute(
    client: &mut proto::burp_service_client::BurpServiceClient<Channel>,
    config: &BurpClientConfig,
    command: Command,
) -> bool {
    match command {
        Command::Ping { request, response } => {
            let result = client
                .ping(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::EchoBytes { request, response } => {
            let result = client
                .echo_bytes(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ProxyHistory { request, response } => {
            let result = client
                .proxy_history(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ProxyDetail { request, response } => {
            let result = client
                .proxy_detail(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SitemapSnapshot { request, response } => {
            let result = client
                .sitemap_snapshot(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::TargetInfo { request, response } => {
            let result = client
                .target_info(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ScopeCheck { request, response } => {
            let result = client
                .scope_check(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ScanIssues { request, response } => {
            let result = client
                .scan_issues(with_deadline(request, config.call_timeout))
                .await
                .map(|r| r.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CookieJar { request, response } => {
            let result = client
                .cookie_jar(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SetCookie { request, response } => {
            let result = client
                .set_cookie(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::AddIssue { request, response } => {
            let result = client
                .add_issue(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ScanIssueDetail { request, response } => {
            let result = client
                .scan_issue_detail(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::GenerateScannerReport { request, response } => {
            let result = client
                .generate_scanner_report(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::InterceptState { request, response } => {
            let result = client
                .intercept_state(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ProxyInterceptConfig { request, response } => {
            let result = client
                .proxy_intercept_config(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ProxyWebSocketHistory { request, response } => {
            let result = client
                .proxy_web_socket_history(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SendToIntruder { request, response } => {
            let result = client
                .send_to_intruder(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RegisterPayloadProcessor { request, response } => {
            let result = client
                .register_payload_processor(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListPayloadProcessors { request, response } => {
            let result = client
                .list_payload_processors(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RemovePayloadProcessor { request, response } => {
            let result = client
                .remove_payload_processor(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RegisterPayloadGenerator { request, response } => {
            let result = client
                .register_payload_generator(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListPayloadGenerators { request, response } => {
            let result = client
                .list_payload_generators(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RemovePayloadGenerator { request, response } => {
            let result = client
                .remove_payload_generator(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CreatePayloadList { request, response } => {
            let result = client
                .create_payload_list(with_deadline(request, config.call_timeout))
                .await
                .map(|value| value.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ImportPayloadList { request, response } => {
            let result = client
                .import_payload_list(with_deadline(request, config.call_timeout))
                .await
                .map(|value| value.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListPayloadLists { request, response } => {
            let result = client
                .list_payload_lists(with_deadline(request, config.call_timeout))
                .await
                .map(|value| value.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::GetPayloadList { request, response } => {
            let result = client
                .get_payload_list(with_deadline(request, config.call_timeout))
                .await
                .map(|value| value.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::UpdatePayloadList { request, response } => {
            let result = client
                .update_payload_list(with_deadline(request, config.call_timeout))
                .await
                .map(|value| value.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::DeletePayloadList { request, response } => {
            let result = client
                .delete_payload_list(with_deadline(request, config.call_timeout))
                .await
                .map(|value| value.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ExtensionInfo { request, response } => {
            let result = client
                .extension_info(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ServerInfo { request, response } => {
            let result = client
                .server_info(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SendRequest { request, response } => {
            let result = client
                .send_request(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SendRequests { request, response } => {
            let result = client
                .send_requests(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SendToRepeater { request, response } => {
            let result = client
                .send_to_repeater(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SetHighlight { request, response } => {
            let result = client
                .set_highlight(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::InspectConfig { request, response } => {
            let result = client
                .inspect_config(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SetNote { request, response } => {
            let result = client
                .set_note(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::MutateScope { request, response } => {
            let result = client
                .mutate_scope(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ExportConfig { request, response } => {
            let result = client
                .export_config(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ImportConfig { request, response } => {
            let result = client
                .import_config(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RegisterHttpHandler { request, response } => {
            let result = client
                .register_http_handler(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ClearHttpHandler { request, response } => {
            let result = client
                .clear_http_handler(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RegisterProxyRule { request, response } => {
            let result = client
                .register_proxy_rule(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListProxyRules { request, response } => {
            let result = client
                .list_proxy_rules(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ClearProxyRules { request, response } => {
            let result = client
                .clear_proxy_rules(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CreateSessionRule { request, response } => {
            let result = client
                .create_session_rule(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListSessionRules { request, response } => {
            let result = client
                .list_session_rules(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RemoveSessionRules { request, response } => {
            let result = client
                .remove_session_rules(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CreateMacro { request, response } => {
            let result = client
                .create_macro(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListMacros { request, response } => {
            let result = client
                .list_macros(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RunMacro { request, response } => {
            let result = client
                .run_macro(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::RemoveMacro { request, response } => {
            let result = client
                .remove_macro(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::StartConcurrentRequestCheck { request, response } => {
            let result = client
                .start_concurrent_request_check(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::StartBoundedInputMatrix { request, response } => {
            let result = client
                .start_bounded_input_matrix(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::StartCrawl { request, response } => {
            let result = client
                .start_crawl(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::StartAudit { request, response } => {
            let result = client
                .start_audit(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::GetJobStatus { request, response } => {
            let result = client
                .get_job_status(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::GenerateCollaboratorPayloads { request, response } => {
            let result = client
                .generate_collaborator_payloads(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::PollCollaboratorInteractions { request, response } => {
            let result = client
                .poll_collaborator_interactions(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CreateWebSocket { request, response } => {
            let result = client
                .create_web_socket(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SendWebSocketText { request, response } => {
            let result = client
                .send_web_socket_text(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::SendWebSocketBinary { request, response } => {
            let result = client
                .send_web_socket_binary(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CloseWebSocket { request, response } => {
            let result = client
                .close_web_socket(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ListWebSockets { request, response } => {
            let result = client
                .list_web_sockets(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ManagedWebSocketHistory { request, response } => {
            let result = client
                .managed_web_socket_history(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ImportBambda { request, response } => {
            let result = client
                .import_bambda(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::ImportBCheck { request, response } => {
            let result = client
                .import_b_check(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::CancelJob { request, response } => {
            let result = client
                .cancel_job(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
        Command::GetJobResult { request, response } => {
            let result = client
                .get_job_result(with_deadline(request, config.call_timeout))
                .await
                .map(|response| response.into_inner())
                .map_err(ClientError::Rpc);
            let reconnect = result.as_ref().is_err_and(is_transport_failure);
            let _ = response.send(result);
            reconnect
        }
    }
}

fn with_deadline<T>(message: T, timeout: Duration) -> Request<T> {
    let mut request = Request::new(message);
    request.set_timeout(timeout);
    request
}

fn is_transport_failure(error: &ClientError) -> bool {
    matches!(error, ClientError::Rpc(status) if matches!(status.code(), tonic::Code::Unavailable | tonic::Code::Unknown | tonic::Code::DeadlineExceeded))
}

fn respond_offline(command: Command) {
    let status =
        Status::unavailable("Burp gRPC service is offline; start the Burp extension and retry");
    match command {
        Command::Ping { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::EchoBytes { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ProxyHistory { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ProxyDetail { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SitemapSnapshot { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::TargetInfo { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ScopeCheck { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ScanIssues { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CookieJar { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ServerInfo { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SendRequest { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SendRequests { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SetCookie { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::AddIssue { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ScanIssueDetail { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::InterceptState { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::GenerateScannerReport { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ProxyInterceptConfig { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ProxyWebSocketHistory { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SendToIntruder { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RegisterPayloadProcessor { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListPayloadProcessors { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RemovePayloadProcessor { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RegisterPayloadGenerator { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CreatePayloadList { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ImportPayloadList { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListPayloadLists { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::GetPayloadList { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::UpdatePayloadList { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::DeletePayloadList { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListPayloadGenerators { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RemovePayloadGenerator { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ExtensionInfo { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SendToRepeater { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SetHighlight { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SetNote { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::MutateScope { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ExportConfig { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ImportConfig { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RegisterHttpHandler { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ClearHttpHandler { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RegisterProxyRule { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ClearProxyRules { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::InspectConfig { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CreateSessionRule { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListSessionRules { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RemoveSessionRules { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CreateMacro { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListMacros { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RunMacro { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::RemoveMacro { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::StartConcurrentRequestCheck { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::StartBoundedInputMatrix { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListProxyRules { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::StartCrawl { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::StartAudit { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::GetJobStatus { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::GenerateCollaboratorPayloads { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::PollCollaboratorInteractions { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CreateWebSocket { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SendWebSocketText { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::SendWebSocketBinary { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CloseWebSocket { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ImportBambda { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ImportBCheck { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ListWebSockets { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::ManagedWebSocketHistory { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::CancelJob { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
        Command::GetJobResult { response, .. } => {
            let _ = response.send(Err(ClientError::Rpc(status)));
        }
    }
}
