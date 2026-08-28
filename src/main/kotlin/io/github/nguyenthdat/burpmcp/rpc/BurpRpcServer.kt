package io.github.nguyenthdat.burpmcp.rpc

import burp.api.montoya.MontoyaApi
import io.github.nguyenthdat.burpmcp.CookieFacade
import io.github.nguyenthdat.burpmcp.CookieQuery
import io.github.nguyenthdat.burpmcp.AnnotationFacade
import io.github.nguyenthdat.burpmcp.HttpFacade
import io.github.nguyenthdat.burpmcp.ConfigFacade
import io.github.nguyenthdat.burpmcp.HttpRequestSpec
import io.github.nguyenthdat.burpmcp.HttpHandlerFacade
import io.github.nguyenthdat.burpmcp.HttpHandlerRule
import io.github.nguyenthdat.burpmcp.ProxyFacade
import io.github.nguyenthdat.burpmcp.ProxyHistoryQuery
import io.github.nguyenthdat.burpmcp.ProxyRuleFacade
import io.github.nguyenthdat.burpmcp.ProxyRule
import io.github.nguyenthdat.burpmcp.ProxyInterceptConfigFacade
import io.github.nguyenthdat.burpmcp.ProxyInterceptConfigPatch
import io.github.nguyenthdat.burpmcp.ProxyInterceptConfig
import io.github.nguyenthdat.burpmcp.ProxyWebSocketInterceptController
import io.github.nguyenthdat.burpmcp.ProxyInterceptRuleConfig
import io.github.nguyenthdat.burpmcp.ProxyInterceptController
import io.github.nguyenthdat.burpmcp.InterceptDecision
import io.github.nguyenthdat.burpmcp.ProxyListenerConfig
import io.github.nguyenthdat.burpmcp.ProxySettingsFacade
import io.github.nguyenthdat.burpmcp.ScriptFilterConfig
import io.github.nguyenthdat.burpmcp.ScannerFacade
import io.github.nguyenthdat.burpmcp.ScanIssueQuery
import io.github.nguyenthdat.burpmcp.ScanCatalogFacade
import io.github.nguyenthdat.burpmcp.ScanConfigurationDefinition
import io.github.nguyenthdat.burpmcp.ScanResourcePoolDefinition
import io.github.nguyenthdat.burpmcp.resolveAudit
import io.github.nguyenthdat.burpmcp.resolveCrawl
import io.github.nguyenthdat.burpmcp.SitemapFacade
import io.github.nguyenthdat.burpmcp.SitemapQuery
import io.github.nguyenthdat.burpmcp.TargetFacade
import io.github.nguyenthdat.burpmcp.SessionRule
import io.github.nguyenthdat.burpmcp.IntruderPayloadFacade
import io.github.nguyenthdat.burpmcp.PayloadGeneratorSpec
import io.github.nguyenthdat.burpmcp.PayloadProcessorOperation
import io.github.nguyenthdat.burpmcp.PayloadProcessorSpec
import io.github.nguyenthdat.burpmcp.SessionRuleFacade
import io.github.nguyenthdat.burpmcp.HttpBatchJobOutput
import io.github.nguyenthdat.burpmcp.JobFacade
import io.github.nguyenthdat.burpmcp.JobSnapshot
import io.github.nguyenthdat.burpmcp.AuditJobOutput
import io.github.nguyenthdat.burpmcp.LongOperationFacade
import io.github.nguyenthdat.burpmcp.PayloadListFacade
import io.github.nguyenthdat.burpmcp.PayloadListDefinition
import io.github.nguyenthdat.burpmcp.TaskJobOutput
import io.github.nguyenthdat.burpmcp.CollaboratorFacade
import io.github.nguyenthdat.burpmcp.WebSocketFacade
import io.github.nguyenthdat.burpmcp.BurpCapabilityFacade
import io.github.nguyenthdat.burpmcp.MacroDefinition
import io.github.nguyenthdat.burpmcp.MacroItemDefinition
import io.github.nguyenthdat.burpmcp.MacroParameterDefinition
import io.github.nguyenthdat.burpmcp.grpc.v1.AddIssueRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CreateMacroRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListMacrosRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListMacrosResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.MacroDefinition as MacroDefinitionProto
import io.github.nguyenthdat.burpmcp.grpc.v1.MacroItem as MacroItemProto
import io.github.nguyenthdat.burpmcp.grpc.v1.MacroParameter as MacroParameterProto
import io.github.nguyenthdat.burpmcp.grpc.v1.RemoveMacroRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RunMacroItem
import io.github.nguyenthdat.burpmcp.grpc.v1.RunMacroRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RunMacroResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ExtensionInfoRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GenerateScannerReportRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GenerateScannerReportResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ExtensionInfoResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptStateRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedMessagesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedMessagesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ControlInterceptedMessageRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedMessageResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedMessage
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptControllerConfigRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptControllerConfigResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptAction
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptStateResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyInterceptConfigRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyInterceptConfigResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyInterceptRule
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyListener
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyScriptFilter
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxySettingsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxySettingsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxySettingsUpdateRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyWebSocketEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyWebSocketHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyWebSocketHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ManagedWebSocketHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ManagedWebSocketHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ManagedWebSocketMessageEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedWebSocketMessagesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedWebSocketMessagesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ControlInterceptedWebSocketMessageRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedWebSocketMessageResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.WebSocketInterceptControllerConfigRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptedWebSocketMessage
import io.github.nguyenthdat.burpmcp.grpc.v1.WebSocketInterceptControllerConfigResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.InspectConfigResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ListProxyRulesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListProxyRulesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyRuleEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.ScanIssueDetailRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SetCookieRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SendToIntruderRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListPayloadGeneratorsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListPayloadGeneratorsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ListPayloadProcessorsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListPayloadProcessorsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.PayloadGeneratorEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.PayloadProcessorEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.RegisterPayloadGeneratorRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RegisterPayloadProcessorRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CreatePayloadListRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.DeletePayloadListRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GetPayloadListRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GetPayloadListResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ImportPayloadListRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListPayloadListsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListPayloadListsResponse
import io.github.nguyenthdat.burpmcp.LoggerHistoryQuery
import io.github.nguyenthdat.burpmcp.OrganizerQuery
import io.github.nguyenthdat.burpmcp.grpc.v1.ClearLoggerRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.LoggerDetailRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.LoggerDetailResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.LoggerEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.LoggerHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.LoggerHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.OrganizerEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.OrganizerListRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.OrganizerListResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.SendToOrganizerRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.TestBCheckRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.TestBCheckResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.UpdateScanIssueStatusRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.UpdateScanIssueStatusResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.PayloadListEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.UpdatePayloadListRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RemovePayloadGeneratorRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.BurpEvent
import io.github.nguyenthdat.burpmcp.grpc.v1.EventsSinceRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EventsSinceResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.RemovePayloadProcessorRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CookieEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.CookieJarRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CookieJarResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.SendRequestRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SendRequestResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.SendRequestsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SendRequestsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ActionResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.SendToRepeaterRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SetHighlightRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SetNoteRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.MutateScopeRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ConfigResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ExportConfigRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ImportConfigRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ClearHttpHandlerRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RegisterHttpHandlerRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EditorGetRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EditorSnapshot
import io.github.nguyenthdat.burpmcp.grpc.v1.EditorPatchRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EditorRenewLeaseRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EditorRenewLeaseResponse
import io.github.nguyenthdat.burpmcp.PatchOp
import io.github.nguyenthdat.burpmcp.UnifiedEditorSnapshot
import io.github.nguyenthdat.burpmcp.grpc.v1.ClearProxyRulesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RegisterProxyRuleRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.UpsertSessionRuleRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GetSessionRuleRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListSessionRulesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListSessionRulesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.DeleteSessionRuleRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SessionRuleEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.CancelJobRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GetJobResultRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GetJobStatusRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.HttpJobResultItem
import io.github.nguyenthdat.burpmcp.grpc.v1.JobResultResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.JobStatusResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.StartBoundedInputMatrixRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.StartConcurrentRequestCheckRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.StartCrawlRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CollaboratorInteractionEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.GenerateCollaboratorPayloadsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.GenerateCollaboratorPayloadsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.PollCollaboratorInteractionsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.PollCollaboratorInteractionsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.CloseWebSocketRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CreateWebSocketRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CreateWebSocketResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ListWebSocketsRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListWebSocketsResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.SendWebSocketBinaryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SendWebSocketTextRequest
import io.github.nguyenthdat.burpmcp.ScriptImportFacade
import io.github.nguyenthdat.burpmcp.grpc.v1.StartAuditRequest
import com.google.protobuf.Any
import com.google.rpc.Status as RpcStatus
import io.github.nguyenthdat.burpmcp.grpc.v1.ErrorCode
import io.github.nguyenthdat.burpmcp.grpc.v1.RpcError
import io.github.nguyenthdat.burpmcp.grpc.v1.BurpServiceGrpc
import io.github.nguyenthdat.burpmcp.grpc.v1.ImportBambdaRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ImportBCheckRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ScriptImportResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.EchoBytesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.PageInfo
import io.github.nguyenthdat.burpmcp.grpc.v1.PingRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.PingResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyDetailRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyDetailResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyHistoryEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.SitemapEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.SitemapSnapshotRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.SitemapSnapshotResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ScopeCheckRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ScopeCheckResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.TargetInfoRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.TargetInfoResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ScanIssueEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.ScanIssuesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ScanIssuesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ServerInfoResponse
import io.grpc.Context
import io.grpc.Contexts
import io.grpc.Metadata
import io.grpc.Server
import io.grpc.ServerCall
import io.grpc.ServerCallHandler
import io.grpc.ServerInterceptor
import io.grpc.ServerInterceptors
import io.grpc.Status
import io.grpc.netty.shaded.io.grpc.netty.NettyServerBuilder
import io.grpc.netty.shaded.io.grpc.netty.GrpcSslContexts
import io.grpc.netty.shaded.io.netty.handler.ssl.ClientAuth
import io.github.nguyenthdat.burpmcp.GrpcSecurityMode
import io.github.nguyenthdat.burpmcp.GrpcSettings
import io.github.nguyenthdat.burpmcp.TlsBundle
import io.grpc.protobuf.StatusProto
import io.grpc.stub.StreamObserver
import java.net.InetAddress
import java.net.InetSocketAddress
import java.time.Clock
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.ExecutorService
import java.util.concurrent.ThreadPoolExecutor
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.min

private const val GRPC_DEFAULT_PAGE_SIZE: Int = RpcLimits.DEFAULT_PAGE_SIZE
private const val GRPC_RESPONSE_OVERHEAD_BYTES: Int = RpcLimits.RESPONSE_OVERHEAD_BYTES
private const val GRPC_SHUTDOWN_SECONDS: Long = RpcLimits.SHUTDOWN_SECONDS

internal class BurpRpcServer(
    private val api: MontoyaApi,
    private val settings: GrpcSettings,
    private val tlsBundle: TlsBundle? = null,
    private val clock: Clock = Clock.systemUTC(),
    private val serverFactory:
        (InetSocketAddress, BurpServiceGrpc.BurpServiceImplBase, ExecutorService, TlsBundle?) -> Server =
        { address, service, executor, bundle ->
            NettyServerBuilder
                .forAddress(address)
                .apply {
                    if (bundle != null) {
                        sslContext(
                            GrpcSslContexts.forServer(bundle.serverCertificate.toFile(), bundle.serverPrivateKey.toFile())
                                .trustManager(bundle.caCertificate.toFile())
                                .clientAuth(ClientAuth.REQUIRE)
                                .build(),
                        )
                    }
                }
                .executor(executor)
                .addService(ServerInterceptors.intercept(service, RpcDeadlineInterceptor))
                .maxInboundMessageSize(GRPC_MAX_MESSAGE_BYTES)
                .maxInboundMetadataSize(GRPC_MAX_METADATA_BYTES)
                .maxConcurrentCallsPerConnection(GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION)
                .permitKeepAliveWithoutCalls(false)
                .build()
        },
) : AutoCloseable {
    init {
        require((settings.securityMode == GrpcSecurityMode.REMOTE_MTLS) == (tlsBundle != null)) {
            "Remote mTLS settings require a TLS bundle; local plaintext settings forbid one"
        }
    }
    private val running = AtomicBoolean(false)
    private var server: Server? = null
    private var executor: ExecutorService? = null
    private var service: BurpRpcService? = null

    fun start() {
        check(running.compareAndSet(false, true)) { "gRPC server is already running" }
        val address = bindAddress(settings)
        val workerPool =
            ThreadPoolExecutor(
                8,
                8,
                0,
                TimeUnit.MILLISECONDS,
                ArrayBlockingQueue(64),
                { runnable -> Thread(runnable, "burp-mcp-grpc").apply { isDaemon = true } },
                ThreadPoolExecutor.AbortPolicy(),
            )
        executor = workerPool
        try {
            val currentService = BurpRpcService(api, clock)
            service = currentService
            server = serverFactory(address, currentService, workerPool, tlsBundle).start()
        } catch (exception: Exception) {
            workerPool.shutdownNow()
            executor = null
            running.set(false)
            service = null
            throw exception
        }
    }

    fun isRunning(): Boolean = running.get() && server?.isShutdown == false

    override fun close() {
        val current = server
        server = null
        running.set(false)
        if (current != null) {
            current.shutdown()
            if (!current.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)) {
                current.shutdownNow()
                current.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)
            }
        }
        service?.close()
        service = null
        executor?.let { workerPool ->
            workerPool.shutdown()
            if (!workerPool.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)) {
                workerPool.shutdownNow()
                workerPool.awaitTermination(GRPC_SHUTDOWN_SECONDS, TimeUnit.SECONDS)
            }
        }
        executor = null
    }

    internal companion object {
        fun bindAddress(settings: GrpcSettings): InetSocketAddress {
            settings.validate()
            val address = InetAddress.getByName(settings.bindAddress)
            if (settings.securityMode == GrpcSecurityMode.LOCAL_PLAINTEXT) {
                require(address.isLoopbackAddress) { "Plaintext gRPC server address must be loopback" }
            }
            return InetSocketAddress(address, settings.port)
        }
    }
}

private object RpcDeadlineInterceptor : ServerInterceptor {
    override fun <ReqT, RespT> interceptCall(
        call: ServerCall<ReqT, RespT>,
        headers: Metadata,
        next: ServerCallHandler<ReqT, RespT>,
    ): io.grpc.ServerCall.Listener<ReqT> {
        val deadline = Context.current().deadline
        if (deadline == null) {
            val failure = structuredStatus(Status.INVALID_ARGUMENT, ErrorCode.ERROR_CODE_INVALID_ARGUMENT, "every gRPC call must set a deadline")
            call.close(failure.status, failure.trailers ?: Metadata())
            return object : io.grpc.ServerCall.Listener<ReqT>() {}
        }
        if (deadline.timeRemaining(TimeUnit.MILLISECONDS) > TimeUnit.SECONDS.toMillis(GRPC_MAX_RPC_TIMEOUT_SECONDS)) {
            val failure =
                structuredStatus(
                    Status.INVALID_ARGUMENT,
                    ErrorCode.ERROR_CODE_INVALID_ARGUMENT,
                    "gRPC deadline must not exceed ${GRPC_MAX_RPC_TIMEOUT_SECONDS}s",
                )
            call.close(failure.status, failure.trailers ?: Metadata())
            return object : io.grpc.ServerCall.Listener<ReqT>() {}
        }
        return Contexts.interceptCall(Context.current(), call, headers, next)
    }
}
internal fun structuredStatus(
    status: Status,
    code: ErrorCode,
    message: String,
    retryable: Boolean = false,
): io.grpc.StatusRuntimeException {
    val detail =
        RpcError
            .newBuilder()
            .setCode(code)
            .setMessage(message)
            .setRetryable(retryable)
            .build()
    val rpcStatus =
        RpcStatus
            .newBuilder()
            .setCode(status.code.value())
            .setMessage(message)
            .addDetails(Any.pack(detail))
            .build()
    return StatusProto.toStatusRuntimeException(rpcStatus)
}
internal fun <T> StreamObserver<T>.respond(block: () -> T) {
    try {
        onNext(block())
        onCompleted()
    } catch (exception: io.grpc.StatusException) {
        onError(exception)
    } catch (exception: IllegalArgumentException) {
        onError(
            structuredStatus(
                Status.INVALID_ARGUMENT,
                ErrorCode.ERROR_CODE_INVALID_ARGUMENT,
                exception.message ?: "invalid argument",
            ),
        )
    } catch (exception: NoSuchElementException) {
        onError(
            structuredStatus(
                Status.NOT_FOUND,
                ErrorCode.ERROR_CODE_NOT_FOUND,
                exception.message ?: "resource not found",
            ),
        )
    } catch (exception: IllegalStateException) {
        onError(
            structuredStatus(
                Status.FAILED_PRECONDITION,
                ErrorCode.ERROR_CODE_INTERNAL,
                exception.message ?: "operation cannot be completed in the current state",
            ),
        )
    } catch (exception: Throwable) {
        val status =
            structuredStatus(
                Status.INTERNAL,
                ErrorCode.ERROR_CODE_INTERNAL,
                "internal server error",
            )
        onError(status.status.withCause(exception).asRuntimeException(status.trailers))
    }
}

internal class BurpRpcService(
    private val api: MontoyaApi,
    private val clock: Clock,
    private val resources: BurpServiceResources = BurpServiceResources(api),
    private val proxyFacade: ProxyFacade = resources.proxy,
    private val sitemapFacade: SitemapFacade = resources.sitemap,
    private val targetFacade: TargetFacade = resources.target,
    private val scannerFacade: ScannerFacade = resources.scanner,
    private val scanCatalogFacade: ScanCatalogFacade = resources.scanCatalog,
    private val cookieFacade: CookieFacade = resources.cookies,
    private val httpFacade: HttpFacade = resources.http,
    private val annotationFacade: AnnotationFacade = resources.annotations,
    private val collaboratorFacade: CollaboratorFacade = resources.collaborator,
    private val scriptImportFacade: ScriptImportFacade = resources.scripts,
    private val webSocketFacade: WebSocketFacade = resources.webSockets,
    private val configFacade: ConfigFacade = resources.config,
    private val httpHandlerFacade: HttpHandlerFacade = resources.httpHandlers,
    private val proxyRuleFacade: ProxyRuleFacade = resources.proxyRules,
    private val proxyInterceptConfigFacade: ProxyInterceptConfigFacade = resources.proxyIntercept,
    private val interceptController: ProxyInterceptController = resources.interceptController,
    private val webSocketInterceptController: ProxyWebSocketInterceptController = resources.webSocketInterceptController,
    private val proxySettingsFacade: ProxySettingsFacade = resources.proxySettings,
    private val macroFacade: io.github.nguyenthdat.burpmcp.MacroFacade = resources.macros,
    private val sessionRuleFacade: SessionRuleFacade = resources.sessionRules,
    private val payloadListFacade: PayloadListFacade = resources.payloadLists,
    private val jobFacade: JobFacade = resources.jobs,
    private val longOperationFacade: LongOperationFacade = resources.longOperations,
    private val capabilityFacade: BurpCapabilityFacade = resources.capabilities,
    private val intruderPayloadFacade: IntruderPayloadFacade = resources.intruderPayloads,
) : BurpServiceGrpc.BurpServiceImplBase() {
    private val systemGrpcService = SystemGrpcService(api, clock)

    override fun ping(request: PingRequest, responseObserver: StreamObserver<PingResponse>) =
        responseObserver.respond { systemGrpcService.pingValue(request) }

    override fun echoBytes(request: EchoBytesRequest, responseObserver: StreamObserver<EchoBytesResponse>) =
        responseObserver.respond { systemGrpcService.echoBytesValue(request) }

    override fun serverInfo(request: ServerInfoRequest, responseObserver: StreamObserver<ServerInfoResponse>) =
        responseObserver.respond { systemGrpcService.serverInfoValue(request) }

    override fun editorGet(
        request: EditorGetRequest,
        responseObserver: StreamObserver<EditorSnapshot>,
    ) = responseObserver.respond {
        val targetHint = request.targetHint.takeIf(String::isNotBlank)
        val ttl = if (request.hasTtlSeconds()) request.ttlSeconds.toLong() else null
        resources.enhancedEditor.capture(targetHint, ttl).toProto()
    }

    override fun editorPatch(
        request: EditorPatchRequest,
        responseObserver: StreamObserver<EditorSnapshot>,
    ) = responseObserver.respond {
        val op = when (request.patchOperationCase) {
            EditorPatchRequest.PatchOperationCase.REPLACE_ALL_TEXT ->
                PatchOp.ReplaceAllText(request.replaceAllText)
            EditorPatchRequest.PatchOperationCase.REPLACE_ALL_PAYLOAD ->
                PatchOp.ReplaceAllPayload(request.replaceAllPayload.toByteArray())
            EditorPatchRequest.PatchOperationCase.REPLACE_SELECTION ->
                PatchOp.ReplaceSelection(request.replaceSelection, 0, 0)
            EditorPatchRequest.PatchOperationCase.REGEX_PATCH ->
                PatchOp.RegexPatch(
                    pattern = request.regexPatch.pattern,
                    replacement = request.regexPatch.replacement,
                    replaceAll = request.regexPatch.replaceAll,
                    caseInsensitive = request.regexPatch.caseInsensitive,
                )
            EditorPatchRequest.PatchOperationCase.HEADER_PATCH ->
                PatchOp.HeaderPatch(
                    name = request.headerPatch.name,
                    value = request.headerPatch.value,
                    remove = request.headerPatch.remove,
                )
            EditorPatchRequest.PatchOperationCase.JSON_PATCH ->
                PatchOp.JsonPatch(
                    jsonPath = request.jsonPatch.jsonPath,
                    valueJson = request.jsonPatch.valueJson,
                )
            EditorPatchRequest.PatchOperationCase.PARAM_PATCH ->
                PatchOp.ParamPatch(
                    name = request.paramPatch.name,
                    value = request.paramPatch.value,
                    remove = request.paramPatch.remove,
                    paramType = request.paramPatch.paramType.takeIf(String::isNotBlank),
                )
            else -> throw IllegalArgumentException("no patch operation specified")
        }
        resources.enhancedEditor.patch(request.token, request.expectedSha256, op).toProto()
    }

    override fun editorRenewLease(
        request: EditorRenewLeaseRequest,
        responseObserver: StreamObserver<EditorRenewLeaseResponse>,
    ) = responseObserver.respond {
        val extendSeconds = request.extendSeconds.toLong().coerceAtLeast(10L)
        val newExpiry = resources.enhancedEditor.renew(request.token, extendSeconds)
        EditorRenewLeaseResponse.newBuilder()
            .setSuccess(true)
            .setNewExpiresAtMillis(newExpiry)
            .build()
    }


    override fun proxyHistory(
        request: ProxyHistoryRequest,
        responseObserver: StreamObserver<ProxyHistoryResponse>,
    ) = responseObserver.respond {
        val limit =
            when {
                !request.hasPage() || request.page.limit == 0 -> 100
                request.page.limit > GRPC_MAX_PAGE_SIZE ->
                    throw IllegalArgumentException("page limit must be at most $GRPC_MAX_PAGE_SIZE")
                else -> request.page.limit.toInt()
            }
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "proxy history cursor")
        val page =
            proxyFacade.history(
                ProxyHistoryQuery(
                    limit = limit,
                    offset = offset,
                    afterId = request.afterId.toInt().takeIf { request.hasAfterId() },
                    urlFilter = request.urlFilter.takeIf(String::isNotEmpty),
                    methodFilter = request.methodFilter.takeIf(String::isNotEmpty),
                    statusFilter = if (request.hasStatusFilter()) request.statusFilter.toInt() else null,
                    hasNotes = request.hasNotes,
                    colorFilter = request.color.takeIf(String::isNotEmpty),
                ),
            )
        val builder = ProxyHistoryResponse.newBuilder()
        var estimatedBytes = 0
        var boundedEnd = page.offset
        for (item in page.items) {
            if (Context.current().isCancelled) throw Status.CANCELLED.asException()
            val protoItem =
                ProxyHistoryEntry
                    .newBuilder()
                    .setIndex(item.index)
                    .setId(item.id)
                    .setMethod(item.method)
                    .setUrl(item.url)
                    .setStatus(item.status ?: 0)
                    .setLength(item.length?.toLong() ?: 0)
                    .setHasResponse(item.hasResponse)
                    .setNotes(item.notes ?: "")
                    .setHighlight(item.highlight ?: "")
                    .setRequest(com.google.protobuf.ByteString.copyFrom(item.request))
                    .setResponse(com.google.protobuf.ByteString.copyFrom(item.response ?: byteArrayOf()))
                    .setTime(item.time)
                    .setContentType(item.contentType)
                    .build()
            val itemBytes = protoItem.serializedSize
            if (estimatedBytes + itemBytes > GRPC_MAX_RESPONSE_BYTES - GRPC_RESPONSE_OVERHEAD_BYTES) break
            builder.addItems(protoItem)
            estimatedBytes += itemBytes
            boundedEnd++
        }
        builder.page =
            PageInfo
                .newBuilder()
                .setTotal(page.total)
                .setTruncated(boundedEnd < page.total)
                .setNextCursor(if (boundedEnd < page.total) boundedEnd.toString() else "")
                .build()
        builder.build()
    }

    override fun proxyDetail(
        request: ProxyDetailRequest,
        responseObserver: StreamObserver<ProxyDetailResponse>,
    ) = responseObserver.respond {
        val detail =
            proxyFacade.detail(request.index)
                ?: throw NoSuchElementException("proxy history index ${request.index} was not found")
        val response =
            ProxyDetailResponse
                .newBuilder()
                .setIndex(detail.index)
                .setRequest(com.google.protobuf.ByteString.copyFrom(detail.request))
                .setNotes(detail.notes ?: "")
                .setHighlight(detail.highlight ?: "")
        detail.response?.let { response.setResponse(com.google.protobuf.ByteString.copyFrom(it)) }
        response.build()
    }

    override fun sitemapSnapshot(
        request: SitemapSnapshotRequest,
        responseObserver: StreamObserver<SitemapSnapshotResponse>,
    ) = responseObserver.respond {
        val limit =
            when {
                !request.hasPage() || request.page.limit == 0 -> 200
                request.page.limit > GRPC_MAX_PAGE_SIZE -> throw IllegalArgumentException("page limit must be at most $GRPC_MAX_PAGE_SIZE")
                else -> request.page.limit.toInt()
            }
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "sitemap cursor")
        val page = sitemapFacade.snapshot(SitemapQuery(request.urlPrefix, limit, offset))
        val response = SitemapSnapshotResponse.newBuilder()
        var boundedEnd = page.offset
        var estimatedBytes = 0
        for (item in page.items) {
            val protoItem =
                SitemapEntry
                    .newBuilder()
                    .setUrl(item.url)
                    .setMethod(item.method)
                    .setStatus(item.status)
                    .setContentType(item.contentType)
                    .setResponseBody(com.google.protobuf.ByteString.copyFrom(item.responseBody))
                    .setRequestBytes(com.google.protobuf.ByteString.copyFrom(item.requestBytes))
                    .setResponseBytes(com.google.protobuf.ByteString.copyFrom(item.responseBytes))
                    .setRedirectUrl(item.redirectUrl)
                    .addAllResponseLinks(item.responseLinks)
                    .addAllFormActions(item.formActions)
                    .addAllScriptSources(item.scriptSources)
                    .build()
            if (estimatedBytes + protoItem.serializedSize > GRPC_MAX_RESPONSE_BYTES - GRPC_RESPONSE_OVERHEAD_BYTES) break
            response.addItems(protoItem)
            estimatedBytes += protoItem.serializedSize
            boundedEnd++
        }
        response.page =
            PageInfo
                .newBuilder()
                .setTotal(page.total)
                .setTruncated(boundedEnd < page.total)
                .setNextCursor(if (boundedEnd < page.total) boundedEnd.toString() else "")
                .build()
        response.build()
    }

    override fun eventsSince(
        request: EventsSinceRequest,
        responseObserver: StreamObserver<EventsSinceResponse>,
    ) = responseObserver.respond {
        val limit = if (request.limit == 0) 100 else request.limit.coerceAtMost(GRPC_MAX_PAGE_SIZE)
        val page = resources.events.since(request.afterSequence, limit.toInt())
        EventsSinceResponse.newBuilder()
            .addAllItems(page.items.map { event ->
                BurpEvent.newBuilder()
                    .setSequence(event.sequence)
                    .setKind(event.kind)
                    .setKey(event.key)
                    .setReconcileRequired(event.reconcileRequired)
                    .setObservedUnixMillis(event.observedUnixMillis)
                    .build()
            })
            .setLatestSequence(page.latestSequence)
            .setGapDetected(page.gapDetected)
            .setTruncated(page.truncated)
            .setNextSequence(page.nextSequence)
            .build()
    }

    override fun targetInfo(
        request: TargetInfoRequest,
        responseObserver: StreamObserver<TargetInfoResponse>,
    ) = responseObserver.respond {
        val limit = if (request.limit == 0) 500 else request.limit.toInt().coerceAtMost(500)
        val info = targetFacade.info(request.urlPrefix, limit)
        TargetInfoResponse
            .newBuilder()
            .addAllHosts(info.hosts)
            .addAllTechnologies(info.technologies)
            .setRequestsSampled(info.requestsSampled)
            .build()
    }

    override fun scopeCheck(
        request: ScopeCheckRequest,
        responseObserver: StreamObserver<ScopeCheckResponse>,
    ) = responseObserver.respond {
        val scope = targetFacade.scope(request.url)
        ScopeCheckResponse
            .newBuilder()
            .setUrl(scope.url)
            .setInScope(scope.inScope)
            .build()
    }

    override fun scanIssues(
        request: ScanIssuesRequest,
        responseObserver: StreamObserver<ScanIssuesResponse>,
    ) = responseObserver.respond {
        val limit =
            when {
                !request.hasPage() || request.page.limit == 0 -> 50
                request.page.limit > GRPC_MAX_PAGE_SIZE -> throw IllegalArgumentException("page limit must be at most $GRPC_MAX_PAGE_SIZE")
                else -> request.page.limit.toInt()
            }
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "scanner cursor")
        val page = scannerFacade.issues(ScanIssueQuery(limit, offset))
        ScanIssuesResponse
            .newBuilder()
            .addAllItems(
                page.items.map { item ->
                    ScanIssueEntry.newBuilder()
                        .setIndex(item.index)
                        .setName(item.name)
                        .setSeverity(item.severity)
                        .setConfidence(item.confidence)
                        .setUrl(item.url)
                        .setDetail(item.detail)
                        .build()
                },
            ).setPage(
                PageInfo.newBuilder()
                    .setTotal(page.total)
                    .setTruncated(page.offset + page.items.size < page.total)
                    .setNextCursor(if (page.offset + page.items.size < page.total) (page.offset + page.items.size).toString() else "")
                    .build(),
            ).build()
    }

    override fun scanIssueDetail(
        request: ScanIssueDetailRequest,
        responseObserver: StreamObserver<ScanIssueEntry>,
    ) = responseObserver.respond {
        val item = scannerFacade.issueDetail(request.index.toInt())
        ScanIssueEntry.newBuilder()
            .setIndex(item.index)
            .setName(item.name)
            .setSeverity(item.severity)
            .setConfidence(item.confidence)
            .setUrl(item.url)
            .setDetail(item.detail)
            .build()
    }

    override fun addIssue(
        request: AddIssueRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        scannerFacade.addIssue(
            request.name,
            request.url,
            request.detail,
            request.remediation,
            request.severity,
            request.confidence,
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("issue added").build()
    }

    override fun generateScannerReport(
        request: GenerateScannerReportRequest,
        responseObserver: StreamObserver<GenerateScannerReportResponse>,
    ) = responseObserver.respond {
        val report =
            scannerFacade.generateReport(
                format = request.format,
                path = request.path,
                issueIndexes = request.issueIndexesList.map { it.toInt() },
            )
        GenerateScannerReportResponse.newBuilder()
            .setPath(report.path)
            .setFormat(report.format)
            .setIssueCount(report.issueCount)
            .setSizeBytes(report.sizeBytes)
            .build()
    }

    override fun cookieJar(
        request: CookieJarRequest,
        responseObserver: StreamObserver<CookieJarResponse>,
    ) = responseObserver.respond {
        val limit = if (request.limit == 0) 100 else request.limit.toInt()
        require(limit <= GRPC_MAX_PAGE_SIZE) { "cookie limit must be at most $GRPC_MAX_PAGE_SIZE" }
        val cookies = cookieFacade.cookies(CookieQuery(request.domain.takeIf(String::isNotEmpty), limit))
        CookieJarResponse
            .newBuilder()
            .addAllItems(
                cookies.map { cookie ->
                    CookieEntry
                        .newBuilder()
                        .setName(cookie.name)
                        .setValue(cookie.value)
                        .setDomain(cookie.domain ?: "")
                        .setPath(cookie.path ?: "")
                        .setExpiration(cookie.expiration ?: "")
                        .build()
                },
            ).build()
    }

    override fun setCookie(
        request: SetCookieRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        cookieFacade.setCookie(
            request.name,
            request.value,
            request.domain,
            request.path.ifBlank { "/" },
            request.expiration.ifBlank { null },
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("cookie updated").build()
    }

    override fun interceptState(
        request: InterceptStateRequest,
        responseObserver: StreamObserver<InterceptStateResponse>,
    ) = responseObserver.respond {
        val enabled = proxyFacade.interceptState(if (request.hasEnabled()) request.enabled else null)
        InterceptStateResponse.newBuilder().setEnabled(enabled).build()
    }

    override fun interceptedMessages(
        request: InterceptedMessagesRequest,
        responseObserver: StreamObserver<InterceptedMessagesResponse>,
    ) = responseObserver.respond {
        val limit = if (!request.hasPage() || request.page.limit == 0) 100 else request.page.limit.toInt()
        require(limit <= GRPC_MAX_PAGE_SIZE) { "page limit must be at most $GRPC_MAX_PAGE_SIZE" }
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "intercepted message cursor")
        val (items, total) = interceptController.list(offset, limit)
        val end = offset + items.size
        InterceptedMessagesResponse.newBuilder()
            .addAllItems(items.map { it.toProto() })
            .setPage(
                PageInfo.newBuilder()
                    .setTotal(total)
                    .setTruncated(end < total)
                    .setNextCursor(if (end < total) end.toString() else "")
                    .build(),
            )
            .build()
    }

    override fun controlInterceptedMessage(
        request: ControlInterceptedMessageRequest,
        responseObserver: StreamObserver<InterceptedMessageResponse>,
    ) = responseObserver.respond {
        require(request.action != InterceptAction.INTERCEPT_ACTION_UNSPECIFIED) { "action is required" }
        val decision = when (request.action) {
            InterceptAction.INTERCEPT_ACTION_FORWARD -> InterceptDecision.FORWARD
            InterceptAction.INTERCEPT_ACTION_DROP -> InterceptDecision.DROP
            InterceptAction.INTERCEPT_ACTION_INTERCEPT -> InterceptDecision.INTERCEPT
            else -> error("unsupported intercept action")
        }
        val replacement = request.message.takeIf { !it.isEmpty }?.toByteArray()
        InterceptedMessageResponse.newBuilder()
            .setMessage(interceptController.resolve(request.id, decision, replacement).toProto())
            .build()
    }

    override fun interceptedWebSocketMessages(
        request: InterceptedWebSocketMessagesRequest,
        responseObserver: StreamObserver<InterceptedWebSocketMessagesResponse>,
    ) = responseObserver.respond {
        val limit = if (!request.hasPage() || request.page.limit == 0) 100 else request.page.limit.toInt()
        require(limit <= GRPC_MAX_PAGE_SIZE) { "page limit must be at most $GRPC_MAX_PAGE_SIZE" }
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "intercepted WebSocket cursor")
        val (items, total) = webSocketInterceptController.list(offset, limit)
        val end = offset + items.size
        InterceptedWebSocketMessagesResponse.newBuilder()
            .addAllItems(items.map { it.toProto() })
            .setPage(PageInfo.newBuilder().setTotal(total).setTruncated(end < total).setNextCursor(if (end < total) end.toString() else "").build())
            .build()
    }

    override fun controlInterceptedWebSocketMessage(
        request: ControlInterceptedWebSocketMessageRequest,
        responseObserver: StreamObserver<InterceptedWebSocketMessageResponse>,
    ) = responseObserver.respond {
        val decision = when (request.action) {
            InterceptAction.INTERCEPT_ACTION_FORWARD -> InterceptDecision.FORWARD
            InterceptAction.INTERCEPT_ACTION_DROP -> InterceptDecision.DROP
            InterceptAction.INTERCEPT_ACTION_INTERCEPT -> InterceptDecision.INTERCEPT
            else -> error("action is required")
        }
        val payload = request.payload.takeIf { request.replacePayload }?.toByteArray()
        InterceptedWebSocketMessageResponse.newBuilder()
            .setMessage(webSocketInterceptController.resolve(request.id, decision, payload).toProto())
            .build()
    }

    override fun webSocketInterceptControllerConfig(
        request: WebSocketInterceptControllerConfigRequest,
        responseObserver: StreamObserver<WebSocketInterceptControllerConfigResponse>,
    ) = responseObserver.respond {
        val state = webSocketInterceptController.configure(
            request.enabled.takeIf { request.hasEnabled() },
            request.timeoutSeconds.toInt().takeIf { request.hasTimeoutSeconds() },
        )
        WebSocketInterceptControllerConfigResponse.newBuilder()
            .setEnabled(state.enabled)
            .setTimeoutSeconds(state.timeoutSeconds)
            .setPending(state.pending)
            .build()
    }

    override fun interceptControllerConfig(
        request: InterceptControllerConfigRequest,
        responseObserver: StreamObserver<InterceptControllerConfigResponse>,
    ) = responseObserver.respond {
        val state = interceptController.configure(
            request.enabled.takeIf { request.hasEnabled() },
            request.timeoutSeconds.toInt().takeIf { request.hasTimeoutSeconds() },
        )
        InterceptControllerConfigResponse.newBuilder()
            .setEnabled(state.enabled)
            .setTimeoutSeconds(state.timeoutSeconds)
            .setPending(state.pending)
            .build()
    }
    override fun proxyInterceptConfig(
        request: ProxyInterceptConfigRequest,
        responseObserver: StreamObserver<ProxyInterceptConfigResponse>,
    ) = responseObserver.respond {
        val config =
            proxyInterceptConfigFacade.update(
                ProxyInterceptConfigPatch(
                    masterInterceptEnabled = request.masterInterceptEnabled.takeIf { request.hasMasterInterceptEnabled() },
                    requestDoIntercept = request.requestDoIntercept.takeIf { request.hasRequestDoIntercept() },
                    requestAutoContentLength = request.requestAutoContentLength.takeIf { request.hasRequestAutoContentLength() },
                    requestFixMissingNewLines = request.requestFixMissingNewLines.takeIf { request.hasRequestFixMissingNewLines() },
                    responseDoIntercept = request.responseDoIntercept.takeIf { request.hasResponseDoIntercept() },
                    responseAutoContentLength = request.responseAutoContentLength.takeIf { request.hasResponseAutoContentLength() },
                    websocketClientToServer = request.websocketClientToServer.takeIf { request.hasWebsocketClientToServer() },
                    websocketServerToClient = request.websocketServerToClient.takeIf { request.hasWebsocketServerToClient() },
                    websocketInScopeOnly = request.websocketInScopeOnly.takeIf { request.hasWebsocketInScopeOnly() },
                    requestRules = request.requestRulesList.map { it.toDomain() },
                    responseRules = request.responseRulesList.map { it.toDomain() },
                    replaceRequestRules = request.replaceRequestRules,
                    replaceResponseRules = request.replaceResponseRules,
                    responseUnhideHiddenFields = request.responseUnhideHiddenFields.takeIf { request.hasResponseUnhideHiddenFields() },
                    responseEnableDisabledFields = request.responseEnableDisabledFields.takeIf { request.hasResponseEnableDisabledFields() },
                    responseRemoveInputLengthLimits = request.responseRemoveInputLengthLimits.takeIf { request.hasResponseRemoveInputLengthLimits() },
                    responseRemoveJavaScriptValidation = request.responseRemoveJavascriptValidation.takeIf { request.hasResponseRemoveJavascriptValidation() },
                    responseRemoveAllJavaScript = request.responseRemoveAllJavascript.takeIf { request.hasResponseRemoveAllJavascript() },
                ),
            )
        config.toProto()
    }

    override fun proxySettings(
        request: ProxySettingsRequest,
        responseObserver: StreamObserver<ProxySettingsResponse>,
    ) = responseObserver.respond { proxySettingsResponse() }

    override fun proxySettingsUpdate(
        request: ProxySettingsUpdateRequest,
        responseObserver: StreamObserver<ProxySettingsResponse>,
    ) = responseObserver.respond {
        when (request.operationCase) {
            ProxySettingsUpdateRequest.OperationCase.LISTENER_UPSERT ->
                proxySettingsFacade.upsertListener(request.listenerUpsert.toDomain())
            ProxySettingsUpdateRequest.OperationCase.LISTENER_DELETE_PORT -> {
                require(proxySettingsFacade.deleteListener(request.listenerDeletePort.toInt())) { "proxy listener not found" }
            }
            ProxySettingsUpdateRequest.OperationCase.SCRIPT_FILTER_UPSERT ->
                proxySettingsFacade.upsertScriptFilter(request.scriptFilterUpsert.toDomain())
            ProxySettingsUpdateRequest.OperationCase.SCRIPT_FILTER_DELETE_TARGET ->
                proxySettingsFacade.deleteScriptFilter(request.scriptFilterDeleteTarget)
            ProxySettingsUpdateRequest.OperationCase.INTERCEPT_RULE_UPSERT -> {
                val mutation = request.interceptRuleUpsert
                require(mutation.hasRule()) { "intercept rule is required" }
                proxyInterceptConfigFacade.upsertRule(
                    mutation.kind,
                    mutation.index.toInt().takeIf { mutation.hasIndex() },
                    mutation.rule.toDomain(),
                )
            }
            ProxySettingsUpdateRequest.OperationCase.INTERCEPT_RULE_DELETE -> {
                val deletion = request.interceptRuleDelete
                proxyInterceptConfigFacade.deleteRule(deletion.kind, deletion.index.toInt())
            }
            ProxySettingsUpdateRequest.OperationCase.INTERCEPT_TOGGLE -> {
                val toggle = request.interceptToggle
                proxyInterceptConfigFacade.update(
                    ProxyInterceptConfigPatch(
                        masterInterceptEnabled = toggle.masterEnabled.takeIf { toggle.hasMasterEnabled() },
                        requestDoIntercept = toggle.requestEnabled.takeIf { toggle.hasRequestEnabled() },
                        responseDoIntercept = toggle.responseEnabled.takeIf { toggle.hasResponseEnabled() },
                    ),
                )
            }
            ProxySettingsUpdateRequest.OperationCase.OPERATION_NOT_SET ->
                throw IllegalArgumentException("one proxy settings operation is required")
        }
        proxySettingsResponse()
    }

    private fun proxySettingsResponse(): ProxySettingsResponse =
        ProxySettingsResponse.newBuilder()
            .addAllListeners(proxySettingsFacade.listeners().map { it.toProto() })
            .addAllScriptFilters(proxySettingsFacade.scriptFilters().map { it.toProto() })
            .setInterception(proxyInterceptConfigFacade.read().toProto())
            .build()

    override fun proxyWebSocketHistory(
        request: ProxyWebSocketHistoryRequest,
        responseObserver: StreamObserver<ProxyWebSocketHistoryResponse>,
    ) = responseObserver.respond {
        val limit = if (!request.hasPage() || request.page.limit == 0) 50 else request.page.limit.coerceAtMost(GRPC_MAX_PAGE_SIZE)
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "WebSocket cursor")
        val page = proxyFacade.webSocketHistory(limit, offset, request.afterId.toInt().takeIf { request.hasAfterId() })
        val end = page.offset + page.items.size
        ProxyWebSocketHistoryResponse.newBuilder()
            .addAllItems(page.items.map { item ->
                ProxyWebSocketEntry.newBuilder()
                    .setIndex(item.index)
                    .setId(item.id)
                    .setWebSocketId(item.webSocketId)
                    .setDirection(item.direction)
                    .setPayload(com.google.protobuf.ByteString.copyFrom(item.payload))
                    .setEditedPayload(com.google.protobuf.ByteString.copyFrom(item.editedPayload))
                    .setTime(item.time)
                    .setListenerPort(item.listenerPort)
                    .setUpgradeUrl(item.upgradeUrl)
                    .build()
            })
            .setPage(
                PageInfo.newBuilder()
                    .setTotal(page.total)
                    .setTruncated(end < page.total)
                    .setNextCursor(if (end < page.total) end.toString() else "")
                    .build(),
            )
            .build()
    }

    override fun sendToIntruder(
        request: SendToIntruderRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        capabilityFacade.sendToIntruder(
            request.request.toByteArray(),
            request.host,
            request.port.toInt(),
            request.https,
            request.tabName.ifBlank { null },
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("request opened in Intruder").build()
    }
    override fun registerPayloadProcessor(
        request: RegisterPayloadProcessorRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val registration =
            intruderPayloadFacade.registerProcessor(
                PayloadProcessorSpec(
                    id = request.id,
                    displayName = request.displayName,
                    operation = PayloadProcessorOperation.parse(request.operation),
                    argument = request.argument,
                    replacement = request.replacement,
                ),
            )
        ActionResponse.newBuilder().setSuccess(true).setMessage(registration.id).build()
    }

    override fun listPayloadProcessors(
        @Suppress("UNUSED_PARAMETER") request: ListPayloadProcessorsRequest,
        responseObserver: StreamObserver<ListPayloadProcessorsResponse>,
    ) = responseObserver.respond {
        ListPayloadProcessorsResponse.newBuilder().addAllItems(
            intruderPayloadFacade.listProcessors().map { item ->
                PayloadProcessorEntry.newBuilder()
                    .setId(item.id)
                    .setDisplayName(item.displayName)
                    .setOperation(item.operation)
                    .setRegistered(item.registered)
                    .build()
            },
        ).build()
    }

    override fun removePayloadProcessor(
        request: RemovePayloadProcessorRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val removed = intruderPayloadFacade.removeProcessor(request.id)
        ActionResponse.newBuilder().setSuccess(removed).setMessage(if (removed) "payload processor removed" else "payload processor not found").build()
    }

    override fun registerPayloadGenerator(
        request: RegisterPayloadGeneratorRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val payloads = when {
            request.payloadListId.isNotBlank() && request.payloadsCount > 0 -> throw IllegalArgumentException("provide payloads or payload_list_id, not both")
            request.payloadListId.isNotBlank() -> payloadListFacade.boundedSlice(request.payloadListId, request.payloadOffset.toInt())
            else -> request.payloadsList
        }
        val registration =
            intruderPayloadFacade.registerGenerator(
                PayloadGeneratorSpec(
                    id = request.id,
                    displayName = request.displayName,
                    payloads = payloads,
                    maxOutputCount = request.maxOutputCount.toInt().takeIf { it > 0 } ?: payloads.size,
                ),
            )
        ActionResponse.newBuilder().setSuccess(true).setMessage(registration.id).build()
    }

    override fun listPayloadGenerators(
        @Suppress("UNUSED_PARAMETER") request: ListPayloadGeneratorsRequest,
        responseObserver: StreamObserver<ListPayloadGeneratorsResponse>,
    ) = responseObserver.respond {
        ListPayloadGeneratorsResponse.newBuilder().addAllItems(
            intruderPayloadFacade.listGenerators().map { item ->
                PayloadGeneratorEntry.newBuilder()
                    .setId(item.id)
                    .setDisplayName(item.displayName)
                    .setPayloadCount(item.payloadCount)
                    .setMaxOutputCount(item.maxOutputCount)
                    .setRegistered(item.registered)
                    .build()
            },
        ).build()
    }

    override fun removePayloadGenerator(
        request: RemovePayloadGeneratorRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val removed = intruderPayloadFacade.removeGenerator(request.id)
        ActionResponse.newBuilder().setSuccess(removed).setMessage(if (removed) "payload generator removed" else "payload generator not found").build()
    }


    override fun extensionInfo(
        @Suppress("UNUSED_PARAMETER") request: ExtensionInfoRequest,
        responseObserver: StreamObserver<ExtensionInfoResponse>,
    ) = responseObserver.respond {
        val info = capabilityFacade.extensionInfo()
        ExtensionInfoResponse.newBuilder()
            .setFilename(info.filename)
            .setIsBapp(info.isBapp)
            .addAllCommandLineArguments(info.commandLineArguments)
            .build()
    }

    override fun createPayloadList(request: CreatePayloadListRequest, responseObserver: StreamObserver<PayloadListEntry>) = responseObserver.respond {
        payloadListFacade.create(request.id, request.displayName, request.payloadsList).toProto()
    }

    override fun importPayloadList(request: ImportPayloadListRequest, responseObserver: StreamObserver<PayloadListEntry>) = responseObserver.respond {
        payloadListFacade.import(request.id, request.displayName, request.content, request.format, request.keepEmpty).toProto()
    }

    override fun listPayloadLists(
        @Suppress("UNUSED_PARAMETER") request: ListPayloadListsRequest,
        responseObserver: StreamObserver<ListPayloadListsResponse>,
    ) = responseObserver.respond {
        ListPayloadListsResponse.newBuilder().addAllItems(payloadListFacade.list().map { it.toProto() }).build()
    }

    override fun getPayloadList(request: GetPayloadListRequest, responseObserver: StreamObserver<GetPayloadListResponse>) = responseObserver.respond {
        val limit = request.page.limit.toInt().takeIf { it > 0 } ?: 100
        val page = payloadListFacade.page(request.id, request.page.cursor.toIntOrNull() ?: 0, limit)
        GetPayloadListResponse.newBuilder().setList(page.list.toProto()).addAllPayloads(page.payloads)
            .setPage(PageInfo.newBuilder().setTotal(page.total).setTruncated(page.nextOffset != null).setNextCursor(page.nextOffset?.toString() ?: "").build()).build()
    }

    override fun updatePayloadList(request: UpdatePayloadListRequest, responseObserver: StreamObserver<PayloadListEntry>) = responseObserver.respond {
        payloadListFacade.update(request.id, request.operation, request.payloadsList, request.index.toInt(), request.indexesList.map { it.toInt() }, request.displayName.takeIf { request.hasDisplayName() }).toProto()
    }

    override fun deletePayloadList(request: DeletePayloadListRequest, responseObserver: StreamObserver<ActionResponse>) = responseObserver.respond {
        val deleted = payloadListFacade.delete(request.id)
        ActionResponse.newBuilder().setSuccess(deleted).setMessage(if (deleted) "payload list deleted" else "payload list not found").build()
    }

    override fun sendRequest(
        request: SendRequestRequest,
        responseObserver: StreamObserver<SendRequestResponse>,
    ) = responseObserver.respond {
        httpFacade.send(request.toSpec()).toProto()
    }

    override fun sendRequests(
        request: SendRequestsRequest,
        responseObserver: StreamObserver<SendRequestsResponse>,
    ) = responseObserver.respond {
        require(request.requestsCount <= 32) { "at most 32 requests may be sent in one batch" }
        SendRequestsResponse
            .newBuilder()
            .addAllResponses(httpFacade.sendParallel(request.requestsList.map { it.toSpec() }).map { it.toProto() })
            .build()
    }

    override fun sendToRepeater(
        request: SendToRepeaterRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val port = if (request.port == 0) if (request.https) 443 else 80 else request.port.toInt()
        httpFacade.sendToRepeater(
            request.request.toStringUtf8(),
            request.host,
            port,
            request.https,
            request.tabName.takeIf(String::isNotEmpty),
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("request opened in Repeater").build()
    }

    override fun setHighlight(
        request: SetHighlightRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val color = annotationFacade.highlight(request.index, request.color.takeIf(String::isNotEmpty))
        ActionResponse.newBuilder().setSuccess(true).setMessage(color).build()
    }

    override fun setNote(
        request: SetNoteRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        annotationFacade.annotate(request.index, request.note)
        ActionResponse.newBuilder().setSuccess(true).setMessage("note updated").build()
    }

    override fun inspectConfig(
        request: ExportConfigRequest,
        responseObserver: StreamObserver<InspectConfigResponse>,
    ) = responseObserver.respond {
        val inspection = configFacade.inspect(request.pathsList)
        InspectConfigResponse.newBuilder()
            .setConfig(inspection.config)
            .addAllPaths(inspection.paths)
            .setSizeBytes(inspection.sizeBytes)
            .build()
    }

    override fun mutateScope(
        request: MutateScopeRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        if (request.include) targetFacade.include(request.url) else targetFacade.exclude(request.url)
        ActionResponse.newBuilder().setSuccess(true).setMessage(if (request.include) "included" else "excluded").build()
    }

    override fun exportConfig(
        request: ExportConfigRequest,
        responseObserver: StreamObserver<ConfigResponse>,
    ) = responseObserver.respond {
        ConfigResponse.newBuilder().setConfig(configFacade.export(request.pathsList)).build()
    }

    override fun importConfig(
        request: ImportConfigRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        configFacade.import(request.config)
        ActionResponse.newBuilder().setSuccess(true).setMessage("configuration imported").build()
    }

    override fun registerHttpHandler(
        request: RegisterHttpHandlerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        httpHandlerFacade.register(
            HttpHandlerRule(
                request.headerName.takeIf(String::isNotEmpty),
                request.headerValue.takeIf(String::isNotEmpty),
                request.match.takeIf(String::isNotEmpty),
                request.replacement.takeIf(String::isNotEmpty),
            ),
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("HTTP handler registered").build()
    }

    override fun clearHttpHandler(
        @Suppress("UNUSED_PARAMETER") request: ClearHttpHandlerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        httpHandlerFacade.clear()
        ActionResponse.newBuilder().setSuccess(true).setMessage("HTTP handlers cleared").build()
    }

    override fun registerProxyRule(
        request: RegisterProxyRuleRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        proxyRuleFacade.register(
            ProxyRule(
                id = request.id.ifBlank { "default" },
                urlContains = request.urlContains,
                phase = request.phase.ifBlank { "request" },
                action = request.action.ifBlank { "forward" },
                match = request.match,
                replacement = request.replacement,
                headerName = request.headerName,
                headerValue = request.headerValue,
                enabled = request.enabled,
            ),
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("proxy rule registered").build()
    }

    override fun listProxyRules(
        @Suppress("UNUSED_PARAMETER") request: ListProxyRulesRequest,
        responseObserver: StreamObserver<ListProxyRulesResponse>,
    ) = responseObserver.respond {
        ListProxyRulesResponse.newBuilder()
            .addAllItems(
                proxyRuleFacade.list().map { rule ->
                    ProxyRuleEntry.newBuilder()
                        .setId(rule.id)
                        .setUrlContains(rule.urlContains)
                        .setPhase(rule.phase)
                        .setAction(rule.action)
                        .setMatch(rule.match)
                        .setReplacement(rule.replacement)
                        .setHeaderName(rule.headerName)
                        .setHeaderValue(rule.headerValue)
                        .setEnabled(rule.enabled)
                        .build()
                },
            ).build()
    }

    override fun clearProxyRules(
        request: ClearProxyRulesRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        proxyRuleFacade.clear(request.id.takeIf(String::isNotEmpty))
        ActionResponse.newBuilder().setSuccess(true).setMessage("proxy rules cleared").build()
    }

    override fun createSessionRule(
        request: UpsertSessionRuleRequest,
        responseObserver: StreamObserver<SessionRuleEntry>,
    ) = responseObserver.respond { sessionRuleFacade.create(request.toSessionRule()).toProto() }

    override fun getSessionRule(
        request: GetSessionRuleRequest,
        responseObserver: StreamObserver<SessionRuleEntry>,
    ) = responseObserver.respond { sessionRuleFacade.get(request.id).toProto() }

    override fun updateSessionRule(
        request: UpsertSessionRuleRequest,
        responseObserver: StreamObserver<SessionRuleEntry>,
    ) = responseObserver.respond { sessionRuleFacade.update(request.toSessionRule()).toProto() }

    override fun listSessionRules(
        @Suppress("UNUSED_PARAMETER") request: ListSessionRulesRequest,
        responseObserver: StreamObserver<ListSessionRulesResponse>,
    ) = responseObserver.respond {
        ListSessionRulesResponse.newBuilder()
            .addAllItems(sessionRuleFacade.list().map { it.toProto() })
            .build()
    }

    override fun deleteSessionRule(
        request: DeleteSessionRuleRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val removed = sessionRuleFacade.remove(request.id)
        ActionResponse.newBuilder()
            .setSuccess(removed)
            .setMessage(if (removed) "session rule deleted" else "session rule not found")
            .build()
    }

    override fun createMacro(
        request: CreateMacroRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val macro = macroFacade.create(request.macro.toDomain())
        ActionResponse.newBuilder().setSuccess(true).setMessage(macro.serialNumber.toString()).build()
    }

    override fun listMacros(
        @Suppress("UNUSED_PARAMETER") request: ListMacrosRequest,
        responseObserver: StreamObserver<ListMacrosResponse>,
    ) = responseObserver.respond {
        ListMacrosResponse.newBuilder().addAllMacros(macroFacade.list().map { it.toProto() }).build()
    }

    override fun runMacro(
        request: RunMacroRequest,
        responseObserver: StreamObserver<RunMacroResponse>,
    ) = responseObserver.respond {
        RunMacroResponse.newBuilder().addAllItems(
            macroFacade.run(request.description).map { exchange ->
                RunMacroItem.newBuilder()
                    .setRequest(exchange.request.toString(Charsets.ISO_8859_1))
                    .setResponse(exchange.response?.toString(Charsets.ISO_8859_1).orEmpty())
                    .setStatusCode(exchange.status ?: 0)
                    .setHasResponse(exchange.response != null)
                    .build()
            },
        ).build()
    }

    override fun removeMacro(
        request: RemoveMacroRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val removed = macroFacade.remove(request.description)
        ActionResponse.newBuilder().setSuccess(removed).setMessage(if (removed) "macro removed" else "macro not found").build()
    }

    override fun startConcurrentRequestCheck(
        request: StartConcurrentRequestCheckRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        val port = request.port.toInt().takeIf { it > 0 } ?: if (request.https) 443 else 80
        longOperationFacade.startRace(
            request.request.toStringUtf8(),
            request.host,
            port,
            request.https,
            request.count.toInt().takeIf { it > 0 } ?: 10,
            request.singlePacketAttack,
        ).toStatusProto()
    }

    override fun startBoundedInputMatrix(
        request: StartBoundedInputMatrixRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        val port = request.port.toInt().takeIf { it > 0 } ?: if (request.https) 443 else 80
        val inputs = when {
            request.payloadListId.isNotBlank() && request.inputsCount > 0 -> throw IllegalArgumentException("provide inputs or payload_list_id, not both")
            request.payloadListId.isNotBlank() -> payloadListFacade.boundedSlice(request.payloadListId, request.payloadOffset.toInt())
            else -> request.inputsList
        }
        val markerPayloads = request.markerPayloadsList.associate { it.marker to it.payloadsList }
        longOperationFacade.startInlineFuzzer(
            request.template.toStringUtf8(),
            request.host,
            port,
            request.https,
            request.marker.ifEmpty { "FUZZ" },
            inputs,
            request.attackMode.ifEmpty { "pitchfork" },
            markerPayloads,
        ).toStatusProto()
    }
    override fun startCrawl(
        request: StartCrawlRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        val spec = scanCatalogFacade.resolveCrawl(
            request.seedUrlsList,
            request.scanConfigurationId,
            request.resourcePoolId,
            request.timeoutSeconds,
            request.stableSeconds,
            request.includeOutOfScope,
        )
        longOperationFacade.startCrawl(spec).toStatusProto()
    }

    override fun startAudit(
        request: StartAuditRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        val spec = scanCatalogFacade.resolveAudit(
            request.url,
            request.auditType,
            request.scanConfigurationId,
            request.resourcePoolId,
            request.timeoutSeconds,
            request.stableSeconds,
            request.includeOutOfScope,
        )
        longOperationFacade.startAudit(spec).toStatusProto()
    }
    override fun stopAudit(
        request: CancelJobRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        (longOperationFacade.stopAudit(request.id) ?: error("audit not found")).toStatusProto()
    }
    override fun listScanConfigurations(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.ListScanConfigurationsRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ListScanConfigurationsResponse>,
    ) = responseObserver.respond {
        io.github.nguyenthdat.burpmcp.grpc.v1.ListScanConfigurationsResponse.newBuilder()
            .addAllItems(scanCatalogFacade.configurations().map { it.toProto() })
            .build()
    }

    override fun getScanConfiguration(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.GetScanConfigurationRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ScanConfigurationEntry>,
    ) = responseObserver.respond { scanCatalogFacade.configuration(request.id).toProto() }

    override fun createScanConfiguration(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.UpsertScanConfigurationRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ScanConfigurationEntry>,
    ) = responseObserver.respond { scanCatalogFacade.createConfiguration(request.toDomain()).toProto() }

    override fun updateScanConfiguration(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.UpsertScanConfigurationRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ScanConfigurationEntry>,
    ) = responseObserver.respond { scanCatalogFacade.updateConfiguration(request.toDomain()).toProto() }

    override fun deleteScanConfiguration(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.DeleteScanConfigurationRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val removed = scanCatalogFacade.deleteConfiguration(request.id)
        ActionResponse.newBuilder().setSuccess(removed).setMessage(if (removed) "scan configuration deleted" else "scan configuration not found or immutable").build()
    }

    override fun listScanResourcePools(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.ListScanResourcePoolsRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ListScanResourcePoolsResponse>,
    ) = responseObserver.respond {
        io.github.nguyenthdat.burpmcp.grpc.v1.ListScanResourcePoolsResponse.newBuilder()
            .addAllItems(scanCatalogFacade.pools().map { it.toProto() })
            .setScannerSupported(false)
            .setSupportMessage("Montoya API 2026.7 does not expose resource-pool binding for Scanner startCrawl/startAudit")
            .build()
    }

    override fun getScanResourcePool(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.GetScanResourcePoolRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ScanResourcePoolEntry>,
    ) = responseObserver.respond { scanCatalogFacade.pool(request.id).toProto() }

    override fun createScanResourcePool(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.UpsertScanResourcePoolRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ScanResourcePoolEntry>,
    ) = responseObserver.respond { scanCatalogFacade.createPool(request.toDomain()).toProto() }

    override fun updateScanResourcePool(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.UpsertScanResourcePoolRequest,
        responseObserver: StreamObserver<io.github.nguyenthdat.burpmcp.grpc.v1.ScanResourcePoolEntry>,
    ) = responseObserver.respond { scanCatalogFacade.updatePool(request.toDomain()).toProto() }

    override fun deleteScanResourcePool(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.DeleteScanResourcePoolRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        val removed = scanCatalogFacade.deletePool(request.id)
        ActionResponse.newBuilder().setSuccess(removed).setMessage(if (removed) "scan resource pool deleted" else "scan resource pool not found or immutable").build()
    }

    override fun removeAudit(
        request: CancelJobRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        longOperationFacade.removeAudit(request.id) ?: error("audit not found")
        ActionResponse.newBuilder().setSuccess(true).setMessage("audit removed").build()
    }


    override fun getJobStatus(
        request: GetJobStatusRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        (jobFacade.status(request.id) ?: throw NoSuchElementException("job not found")).toStatusProto()
    }

    override fun cancelJob(
        request: CancelJobRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        (jobFacade.cancel(request.id) ?: throw NoSuchElementException("job not found")).toStatusProto()
    }

    override fun getJobResult(
        request: GetJobResultRequest,
        responseObserver: StreamObserver<JobResultResponse>,
    ) = responseObserver.respond {
        val snapshot = jobFacade.result(request.id) ?: throw NoSuchElementException("job not found")
        val limit = request.page.limit.toInt().takeIf { it > 0 }?.coerceAtMost(GRPC_MAX_PAGE_SIZE) ?: GRPC_DEFAULT_PAGE_SIZE
        val offset = parseCursor(request.page.cursor, "job result cursor")
        snapshot.toResultProto(offset, limit)
    }

    override fun generateCollaboratorPayloads(
        request: GenerateCollaboratorPayloadsRequest,
        responseObserver: StreamObserver<GenerateCollaboratorPayloadsResponse>,
    ) = responseObserver.respond {
        val payloads = collaboratorFacade.generate(
            count = request.count.toInt().takeIf { it > 0 } ?: 1,
            targetUrl = request.targetUrl.takeIf(String::isNotBlank),
            injectionPoint = request.injectionPoint.takeIf(String::isNotBlank),
        )
        GenerateCollaboratorPayloadsResponse.newBuilder().addAllPayloads(payloads).build()
    }

    override fun pollCollaboratorInteractions(
        request: PollCollaboratorInteractionsRequest,
        responseObserver: StreamObserver<PollCollaboratorInteractionsResponse>,
    ) = responseObserver.respond {
        val items = collaboratorFacade.interactions()
        val limit = request.page.limit.toInt().takeIf { it > 0 }?.coerceAtMost(GRPC_MAX_PAGE_SIZE) ?: GRPC_DEFAULT_PAGE_SIZE
        val offset = parseCursor(request.page.cursor, "Collaborator cursor")
        val end = minOf(offset + limit, items.size)
        val pageItems = if (offset >= items.size) emptyList() else items.subList(offset, end)
        PollCollaboratorInteractionsResponse
            .newBuilder()
            .addAllItems(
                pageItems.map { item ->
                    CollaboratorInteractionEntry
                        .newBuilder()
                        .setId(item.id)
                        .setType(item.type)
                        .setClientIp(item.clientIp)
                        .setClientPort(item.clientPort)
                        .setTimestamp(item.timestamp)
                        .setTargetUrl(item.targetUrl.orEmpty())
                        .setInjectionPoint(item.injectionPoint.orEmpty())
                        .setPayload(item.payload.orEmpty())
                        .build()
                },
            ).setPage(
                PageInfo.newBuilder()
                    .setTotal(items.size)
                    .setTruncated(end < items.size)
                    .setNextCursor(if (end < items.size) end.toString() else "")
                    .build(),
            ).build()
    }

    override fun sendToComparer(
        request: io.github.nguyenthdat.burpmcp.grpc.v1.SendToComparerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        api.comparer().sendToComparer(
            burp.api.montoya.core.ByteArray.byteArray(*request.first.toByteArray()),
            burp.api.montoya.core.ByteArray.byteArray(*request.second.toByteArray()),
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("sent to comparer").build()
    }

    override fun createWebSocket(
        request: CreateWebSocketRequest,
        responseObserver: StreamObserver<CreateWebSocketResponse>,
    ) = responseObserver.respond {
        val port = request.port.toInt().takeIf { it > 0 } ?: if (request.https) 443 else 80
        val creation = webSocketFacade.create(request.host, port, request.https, request.path.ifEmpty { "/" })
        CreateWebSocketResponse.newBuilder().setId(creation.id.orEmpty()).setStatus(creation.status).build()
    }

    override fun sendWebSocketText(
        request: SendWebSocketTextRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        webSocketFacade.sendText(request.id, request.text)
        ActionResponse.newBuilder().setSuccess(true).setMessage("message sent").build()
    }

    override fun managedWebSocketHistory(
        request: ManagedWebSocketHistoryRequest,
        responseObserver: StreamObserver<ManagedWebSocketHistoryResponse>,
    ) = responseObserver.respond {
        val limit = request.page.limit.toInt().takeIf { it > 0 }?.coerceAtMost(GRPC_MAX_PAGE_SIZE) ?: GRPC_DEFAULT_PAGE_SIZE
        val offset = parseCursor(request.page.cursor, "managed WebSocket cursor")
        val page = webSocketFacade.history(request.id.takeIf(String::isNotEmpty), offset, limit)
        val end = page.offset + page.items.size
        ManagedWebSocketHistoryResponse.newBuilder()
            .addAllItems(
                page.items.map { item ->
                    ManagedWebSocketMessageEntry.newBuilder()
                        .setIndex(item.index)
                        .setWebsocketId(item.webSocketId)
                        .setDirection(item.direction)
                        .setType(item.type)
                        .setPayload(com.google.protobuf.ByteString.copyFrom(item.payload))
                        .build()
                },
            ).setPage(
                PageInfo.newBuilder()
                    .setTotal(page.total)
                    .setTruncated(end < page.total)
                    .setNextCursor(if (end < page.total) end.toString() else "")
                    .build(),
            ).build()
    }

    override fun sendWebSocketBinary(
        request: SendWebSocketBinaryRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        webSocketFacade.sendBinary(request.id, request.data.toByteArray())
        ActionResponse.newBuilder().setSuccess(true).setMessage("message sent").build()
    }

    override fun closeWebSocket(
        request: CloseWebSocketRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        webSocketFacade.close(request.id)
        ActionResponse.newBuilder().setSuccess(true).setMessage("WebSocket closed").build()
    }
    override fun importBambda(
        request: ImportBambdaRequest,
        responseObserver: StreamObserver<ScriptImportResponse>,
    ) = responseObserver.respond {
        val result = scriptImportFacade.importBambda(request.script)
        ScriptImportResponse.newBuilder().setSuccess(result.success).setStatus(result.status).addAllErrors(result.errors).build()
    }
    private fun PayloadListDefinition.toProto(): PayloadListEntry = PayloadListEntry.newBuilder()
        .setId(id).setDisplayName(displayName).setPayloadCount(payloads.size).setSizeBytes(sizeBytes).setFingerprint(fingerprint).build()


    override fun importBCheck(
        request: ImportBCheckRequest,
        responseObserver: StreamObserver<ScriptImportResponse>,
    ) = responseObserver.respond {
        val result = scriptImportFacade.importBCheck(request.script, request.enabled)
        ScriptImportResponse.newBuilder().setSuccess(result.success).setStatus(result.status).addAllErrors(result.errors).build()
    }


    override fun listWebSockets(
        @Suppress("UNUSED_PARAMETER") request: ListWebSocketsRequest,
        responseObserver: StreamObserver<ListWebSocketsResponse>,
    ) = responseObserver.respond {
        ListWebSocketsResponse.newBuilder().addAllIds(webSocketFacade.list()).build()
    }


    private fun parseCursor(cursor: String, field: String): Int {
        if (cursor.isBlank()) return 0
        val parsed = cursor.toIntOrNull()
        require(parsed != null && parsed >= 0) { "$field must be a non-negative decimal offset" }
        return parsed
    }

    private fun SendRequestRequest.toSpec(): HttpRequestSpec =
        HttpRequestSpec(
            method = method.ifEmpty { "GET" },
            url = url,
            body = body.toStringUtf8(),
            headers = headersList.associate { it.name to it.value },
        )

    private fun ScanConfigurationDefinition.toProto(): io.github.nguyenthdat.burpmcp.grpc.v1.ScanConfigurationEntry =
        io.github.nguyenthdat.burpmcp.grpc.v1.ScanConfigurationEntry.newBuilder()
            .setId(id).setName(name).setScanType(scanType).setAuditType(auditType)
            .setIncludeOutOfScope(includeOutOfScope).setTimeoutSeconds(timeoutSeconds)
            .setStableSeconds(stableSeconds).setResourcePoolId(resourcePoolId).setSource(source).build()

    private fun io.github.nguyenthdat.burpmcp.grpc.v1.UpsertScanConfigurationRequest.toDomain() =
        ScanConfigurationDefinition(id, name, scanType, auditType, includeOutOfScope, timeoutSeconds, stableSeconds, resourcePoolId)

    private fun UpsertSessionRuleRequest.toSessionRule(): SessionRule =
        SessionRule(
            id = id,
            description = description.ifBlank { "Burp MCP session rule" },
            actionType = actionType.ifBlank { "replace_text" },
            find = find,
            replacement = replacement,
            headerName = headerName,
            parameterName = parameterName,
            macroDescription = macroDescription,
            urlContains = urlContains,
            tools = toolsList.map(String::lowercase).toSet(),
            enabled = enabled,
        )

    private fun SessionRule.toProto(): SessionRuleEntry =
        SessionRuleEntry.newBuilder()
            .setId(id)
            .setFind(find)
            .setReplacement(replacement)
            .setDescription(description)
            .setActionType(actionType)
            .setHeaderName(headerName)
            .setParameterName(parameterName)
            .setMacroDescription(macroDescription)
            .setUrlContains(urlContains)
            .addAllTools(tools.sorted())
            .setEnabled(enabled)
            .build()


    private fun ScanResourcePoolDefinition.toProto(): io.github.nguyenthdat.burpmcp.grpc.v1.ScanResourcePoolEntry =
        io.github.nguyenthdat.burpmcp.grpc.v1.ScanResourcePoolEntry.newBuilder()
            .setId(id).setName(name).setKind(kind).setExistingPoolName(existingPoolName)
            .setConcurrentRequestLimit(concurrentRequestLimit).setThrottleMillis(throttleMillis)
            .setMaxRetries(maxRetries).setSource(source).build()

    private fun io.github.nguyenthdat.burpmcp.grpc.v1.UpsertScanResourcePoolRequest.toDomain() =
        ScanResourcePoolDefinition(id, name, kind, existingPoolName, concurrentRequestLimit, throttleMillis, maxRetries)

    private fun io.github.nguyenthdat.burpmcp.HttpExchange.toProto(): SendRequestResponse =
        SendRequestResponse
            .newBuilder()
            .setRequest(com.google.protobuf.ByteString.copyFrom(request))
            .setResponse(com.google.protobuf.ByteString.copyFrom(response ?: byteArrayOf()))
            .setStatus(status ?: 0)
            .setHasResponse(response != null)
            .build()

    fun close() {
        resources.close()
    }
    private fun extensionVersion(): String =
        BurpRpcService::class.java.`package`.implementationVersion ?: "development"

    private fun ProxyInterceptRule.toDomain(): ProxyInterceptRuleConfig =
        ProxyInterceptRuleConfig(
            enabled = enabled,
            booleanOperator = booleanOperator.ifBlank { "and" },
            matchType = matchType,
            matchRelationship = matchRelationship,
            matchCondition = matchCondition,
        )

    private fun ProxyInterceptRuleConfig.toProto(): ProxyInterceptRule =
        ProxyInterceptRule.newBuilder()
            .setEnabled(enabled)
            .setBooleanOperator(booleanOperator)
            .setMatchType(matchType)
            .setMatchRelationship(matchRelationship)
            .setMatchCondition(matchCondition)
            .build()

    private fun ProxyInterceptConfig.toProto(): ProxyInterceptConfigResponse =
        ProxyInterceptConfigResponse.newBuilder()
            .setMasterInterceptEnabled(masterInterceptEnabled)
            .setRequestDoIntercept(requestDoIntercept)
            .setRequestAutoContentLength(requestAutoContentLength)
            .setRequestFixMissingNewLines(requestFixMissingNewLines)
            .setResponseDoIntercept(responseDoIntercept)
            .setResponseAutoContentLength(responseAutoContentLength)
            .setWebsocketClientToServer(websocketClientToServer)
            .setWebsocketServerToClient(websocketServerToClient)
            .setWebsocketInScopeOnly(websocketInScopeOnly)
            .addAllRequestRules(requestRules.map { it.toProto() })
            .addAllResponseRules(responseRules.map { it.toProto() })
            .setResponseUnhideHiddenFields(responseUnhideHiddenFields)
            .setResponseEnableDisabledFields(responseEnableDisabledFields)
            .setResponseRemoveInputLengthLimits(responseRemoveInputLengthLimits)
            .setResponseRemoveJavascriptValidation(responseRemoveJavaScriptValidation)
            .setResponseRemoveAllJavascript(responseRemoveAllJavaScript)
            .build()
    private fun ProxyListener.toDomain(): ProxyListenerConfig =
        ProxyListenerConfig(
            port = port.toInt(),
            running = running,
            listenMode = listenMode.ifBlank { "loopback_only" },
            listenSpecificAddress = listenSpecificAddress,
            certificateMode = certificateMode.ifBlank { "per_host" },
            enableHttp2 = enableHttp2,
            supportInvisibleProxying = supportInvisibleProxying,
        )

    private fun ProxyListenerConfig.toProto(): ProxyListener =
        ProxyListener.newBuilder()
            .setPort(port)
            .setRunning(running)
            .setListenMode(listenMode)
            .setListenSpecificAddress(listenSpecificAddress)
            .setCertificateMode(certificateMode)
            .setEnableHttp2(enableHttp2)
            .setSupportInvisibleProxying(supportInvisibleProxying)
            .build()

    private fun ProxyScriptFilter.toDomain(): ScriptFilterConfig =
        ScriptFilterConfig(target, mode, script, scriptId, scriptName)

    private fun ScriptFilterConfig.toProto(): ProxyScriptFilter =
        ProxyScriptFilter.newBuilder()
            .setTarget(target)
            .setMode(mode)
            .setScript(script)
            .setScriptId(scriptId)
            .setScriptName(scriptName)
            .build()

    private fun MacroDefinitionProto.toDomain(): MacroDefinition =
        MacroDefinition(
            description = description,
            serialNumber = serialNumber.toLong(),
            items = itemsList.map { item ->
                MacroItemDefinition(
                    request = item.request,
                    method = item.method,
                    url = item.url,
                    response = item.response,
                    statusCode = item.statusCode.toInt(),
                    cookiesReceived = item.cookiesReceived,
                    requestParameters = item.requestParametersList.map { parameter ->
                        MacroParameterDefinition(
                            name = parameter.name,
                            originalValue = parameter.originalValue,
                            parameterHandling = parameter.parameterHandling,
                            presetValue = parameter.presetValue,
                            type = parameter.type,
                        )
                    },
                    customParameters = item.customParametersList,
                )
            },
        )

    private fun MacroDefinition.toProto(): MacroDefinitionProto =
        MacroDefinitionProto.newBuilder()
            .setDescription(description)
            .setSerialNumber(serialNumber.toULong().toLong())
            .addAllItems(items.map { item ->
                MacroItemProto.newBuilder()
                    .setRequest(item.request)
                    .setMethod(item.method)
                    .setUrl(item.url)
                    .setResponse(item.response)
                    .setStatusCode(item.statusCode)
                    .setCookiesReceived(item.cookiesReceived)
                    .addAllRequestParameters(item.requestParameters.map { parameter ->
                        MacroParameterProto.newBuilder()
                            .setName(parameter.name)
                            .setOriginalValue(parameter.originalValue)
                            .setParameterHandling(parameter.parameterHandling)
                            .setPresetValue(parameter.presetValue)
                            .setType(parameter.type)
                            .build()
                    })
                    .addAllCustomParameters(item.customParameters)
                    .build()
            })
            .build()

    private fun JobSnapshot.toStatusProto(): JobStatusResponse {
        val output = result as? AuditJobOutput
        return JobStatusResponse
            .newBuilder()
            .setId(id)
            .setOperation(operation)
            .setState(state.name.lowercase())
            .setError(error.orEmpty())
            .setScanType(output?.scanType ?: scanType)
            .setStateless(output?.stateless ?: stateless)
            .setStatusMessage(output?.statusMessage ?: statusMessage)
            .setRequestCount(output?.requestCount ?: 0)
            .setErrorCount(output?.errorCount ?: 0)
            .setIssueCount(output?.issueCount ?: 0)
            .build()
    }

    private fun JobSnapshot.toResultProto(offset: Int, limit: Int): JobResultResponse {
        val builder = JobResultResponse.newBuilder()
            .setId(id)
            .setOperation(operation)
            .setState(state.name.lowercase())
            .setError(error.orEmpty())
            .setScanType(scanType)
            .setStateless(stateless)
            .setStatusMessage(statusMessage)
        when (val output = result) {
            is HttpBatchJobOutput -> {
                val end = minOf(offset + limit, output.items.size)
                val pageItems = if (offset >= output.items.size) emptyList() else output.items.subList(offset, end)
                builder
                    .addAllItems(pageItems.map { item ->
                        HttpJobResultItem.newBuilder()
                            .setLabel(item.label)
                            .also { entry -> item.status?.let(entry::setStatus) }
                            .also { entry -> item.length?.let(entry::setLength) }
                            .setError(item.error.orEmpty())
                            .build()
                    })
                    .setPage(PageInfo.newBuilder()
                        .setTotal(output.items.size)
                        .setTruncated(end < output.items.size)
                        .setNextCursor(if (end < output.items.size) end.toString() else "")
                        .build())
                    .setRequestCount(output.requestCount)
                    .setUniqueLengths(output.uniqueLengths)
                    .setVerdict(output.verdict)
                    .setSubstitutionCount(output.substitutionCount)
                    .setRequestFingerprint(output.requestFingerprint)
            }
            is TaskJobOutput -> builder.setRequestCount(output.requestCount).setErrorCount(output.errorCount)
            is AuditJobOutput -> builder
                .setRequestCount(output.requestCount)
                .setErrorCount(output.errorCount)
                .setIssueCount(output.issueCount)
                .setScanType(output.scanType)
                .setStateless(output.stateless)
                .setStatusMessage(output.statusMessage)
            null -> Unit
        }
        return builder.build()
    }

    private fun io.github.nguyenthdat.burpmcp.PendingIntercept.toProto(): InterceptedMessage =
        InterceptedMessage.newBuilder()
            .setId(id)
            .setDirection(direction.name.lowercase())
            .setUrl(url)
            .setMethod(method)
            .setStatus(status)
            .setIsInScope(isInScope)
            .setRequest(com.google.protobuf.ByteString.copyFrom(request))
            .setResponse(com.google.protobuf.ByteString.copyFrom(response))
            .setPhase(phase.name.lowercase())
            .build()

    private fun io.github.nguyenthdat.burpmcp.PendingWebSocketIntercept.toProto(): InterceptedWebSocketMessage =
        InterceptedWebSocketMessage.newBuilder()
            .setId(id)
            .setWebSocketId(webSocketId)
            .setUpgradeUrl(upgradeUrl)
            .setDirection(direction)
            .setMessageType(messageType.name.lowercase())
            .setPhase(phase.name.lowercase())
            .setPayload(com.google.protobuf.ByteString.copyFrom(payload))
            .build()


    override fun loggerHistory(
        request: LoggerHistoryRequest,
        responseObserver: StreamObserver<LoggerHistoryResponse>,
    ) = responseObserver.respond {
        val limit = if (!request.hasPage() || request.page.limit == 0) 100 else request.page.limit.toInt().coerceAtMost(GRPC_MAX_PAGE_SIZE)
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "logger cursor")
        val page = resources.logger.history(
            LoggerHistoryQuery(
                limit = limit,
                offset = offset,
                afterId = if (request.hasAfterId()) request.afterId.toLong() else null,
                sourceFilter = request.sourceFilter.takeIf(String::isNotBlank),
                urlFilter = request.urlFilter.takeIf(String::isNotBlank),
                methodFilter = request.methodFilter.takeIf(String::isNotBlank),
                statusFilter = if (request.hasStatusFilter()) request.statusFilter else null,
                hasNotes = request.hasNotes,
                colorFilter = request.color.takeIf(String::isNotBlank),
            )
        )
        val nextOffset = page.offset + page.items.size
        LoggerHistoryResponse.newBuilder()
            .addAllItems(page.items.map { item ->
                LoggerEntry.newBuilder()
                    .setIndex(item.index)
                    .setId(item.id.toInt())
                    .setSource(item.source)
                    .setMethod(item.method)
                    .setUrl(item.url)
                    .also { entry -> item.status?.let(entry::setStatus) }
                    .also { entry -> item.length?.let(entry::setLength) }
                    .setHasResponse(item.hasResponse)
                    .setRequest(com.google.protobuf.ByteString.copyFrom(item.request))
                    .also { entry -> item.response?.let { entry.setResponse(com.google.protobuf.ByteString.copyFrom(it)) } }
                    .setNotes(item.notes.orEmpty())
                    .setHighlight(item.highlight.orEmpty())
                    .setTime(item.time)
                    .setContentType(item.contentType)
                    .build()
            })
            .setPage(
                PageInfo.newBuilder()
                    .setTotal(page.total)
                    .setTruncated(nextOffset < page.total)
                    .setNextCursor(if (nextOffset < page.total) nextOffset.toString() else "")
                    .build()
            )
            .build()
    }

    override fun loggerDetail(
        request: LoggerDetailRequest,
        responseObserver: StreamObserver<LoggerDetailResponse>,
    ) = responseObserver.respond {
        val detail = resources.logger.detail(request.index)
            ?: error("logger index out of range: ${request.index}")
        LoggerDetailResponse.newBuilder()
            .setIndex(detail.index)
            .setSource(detail.source)
            .setRequest(com.google.protobuf.ByteString.copyFrom(detail.request))
            .also { entry -> detail.response?.let { entry.setResponse(com.google.protobuf.ByteString.copyFrom(it)) } }
            .setNotes(detail.notes.orEmpty())
            .setHighlight(detail.highlight.orEmpty())
            .build()
    }

    override fun clearLogger(
        request: ClearLoggerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        resources.logger.clear()
        ActionResponse.newBuilder().setSuccess(true).setMessage("logger cleared").build()
    }

    override fun sendToOrganizer(
        request: SendToOrganizerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) = responseObserver.respond {
        resources.organizer.sendToOrganizer(
            request = request.request.toByteArray(),
            response = if (request.response.isEmpty) null else request.response.toByteArray(),
            host = request.host,
            port = request.port,
            https = request.https,
            notes = request.notes.takeIf(String::isNotBlank),
            highlight = request.highlight.takeIf(String::isNotBlank),
        )
        ActionResponse.newBuilder().setSuccess(true).setMessage("sent to organizer").build()
    }

    override fun organizerList(
        request: OrganizerListRequest,
        responseObserver: StreamObserver<OrganizerListResponse>,
    ) = responseObserver.respond {
        val limit = if (!request.hasPage() || request.page.limit == 0) 100 else request.page.limit.toInt().coerceAtMost(GRPC_MAX_PAGE_SIZE)
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "organizer cursor")
        val page = resources.organizer.list(
            OrganizerQuery(
                limit = limit,
                offset = offset,
                statusFilter = request.statusFilter.takeIf(String::isNotBlank),
                urlFilter = request.urlFilter.takeIf(String::isNotBlank),
            )
        )
        val nextOffset = page.offset + page.items.size
        OrganizerListResponse.newBuilder()
            .addAllItems(page.items.map { item ->
                OrganizerEntry.newBuilder()
                    .setId(item.id)
                    .setIndex(item.index)
                    .setUrl(item.url)
                    .setMethod(item.method)
                    .setStatusCode(item.statusCode)
                    .setStatus(item.status)
                    .setNotes(item.notes)
                    .setHighlight(item.highlight)
                    .setHasResponse(item.hasResponse)
                    .setContentType(item.contentType)
                    .build()
            })
            .setPage(
                PageInfo.newBuilder()
                    .setTotal(page.total)
                    .setTruncated(nextOffset < page.total)
                    .setNextCursor(if (nextOffset < page.total) nextOffset.toString() else "")
                    .build()
            )
            .build()
    }

    override fun testBCheck(
        request: TestBCheckRequest,
        responseObserver: StreamObserver<TestBCheckResponse>,
    ) = responseObserver.respond {
        val result = resources.scripts.testBCheck(
            script = request.script,
            request = request.request.toByteArray(),
            response = request.response.toByteArray(),
            host = request.host,
            port = request.port.toInt(),
            https = request.https,
        )
        TestBCheckResponse.newBuilder()
            .setValid(result.valid)
            .setMatched(result.matched)
            .setStatus(result.status)
            .addAllErrors(result.errors)
            .addAllFindings(result.findings)
            .build()
    }

    override fun updateScanIssueStatus(
        request: UpdateScanIssueStatusRequest,
        responseObserver: StreamObserver<UpdateScanIssueStatusResponse>,
    ) = responseObserver.respond {
        val updated = resources.scanner.updateIssueStatus(
            index = request.index,
            status = request.status,
            severity = if (request.hasSeverity()) request.severity else null,
            confidence = if (request.hasConfidence()) request.confidence else null,
            notes = if (request.hasNotes()) request.notes else null,
        )
        UpdateScanIssueStatusResponse.newBuilder()
            .setSuccess(updated)
            .setMessage("Issue status updated to ${request.status}")
            .build()
    }

    private fun UnifiedEditorSnapshot.toProto(): EditorSnapshot =
        EditorSnapshot.newBuilder()
            .setToken(token)
            .setKind(kind.name.lowercase())
            .setToolSource(toolSource)
            .setTabName(tabName)
            .setHost(host)
            .setPort(port)
            .setHttps(https)
            .setText(text)
            .setPayload(com.google.protobuf.ByteString.copyFrom(payload))
            .setIsJson(isJson)
            .setEditable(editable)
            .setSha256(sha256)
            .setCaretPosition(caretPosition)
            .setSelectionStart(selectionStart)
            .setSelectionEnd(selectionEnd)
            .setSelectedText(selectedText)
            .setExpiresAtMillis(expiresAtMillis)
            .build()
}
