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
import io.github.nguyenthdat.burpmcp.ProxyInterceptRuleConfig
import io.github.nguyenthdat.burpmcp.ScannerFacade
import io.github.nguyenthdat.burpmcp.ScanIssueQuery
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
import io.github.nguyenthdat.burpmcp.grpc.v1.InterceptStateResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyInterceptConfigRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyInterceptConfigResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyInterceptRule
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyWebSocketEntry
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyWebSocketHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ProxyWebSocketHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ManagedWebSocketHistoryRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ManagedWebSocketHistoryResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.ManagedWebSocketMessageEntry
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
import io.github.nguyenthdat.burpmcp.grpc.v1.ClearProxyRulesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.RegisterProxyRuleRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.CreateSessionRuleRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListSessionRulesRequest
import io.github.nguyenthdat.burpmcp.grpc.v1.ListSessionRulesResponse
import io.github.nguyenthdat.burpmcp.grpc.v1.RemoveSessionRulesRequest
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
    private val port: Int,
    private val clock: Clock = Clock.systemUTC(),
    private val serverFactory:
        (InetSocketAddress, BurpServiceGrpc.BurpServiceImplBase, ExecutorService) -> Server =
        { address, service, executor ->
            NettyServerBuilder
                .forAddress(address)
                .executor(executor)
                .addService(ServerInterceptors.intercept(service, RpcDeadlineInterceptor))
                .maxInboundMessageSize(GRPC_MAX_MESSAGE_BYTES)
                .maxInboundMetadataSize(GRPC_MAX_METADATA_BYTES)
                .maxConcurrentCallsPerConnection(GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION)
                .permitKeepAliveWithoutCalls(false)
                .build()
        },
) : AutoCloseable {
    private val running = AtomicBoolean(false)
    private var server: Server? = null
    private var executor: ExecutorService? = null
    private var service: BurpRpcService? = null

    fun start() {
        check(running.compareAndSet(false, true)) { "gRPC server is already running" }
        val address = loopbackAddress(port)
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
            server = serverFactory(address, currentService, workerPool).start()
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
        fun loopbackAddress(port: Int): InetSocketAddress {
            require(port in 1..65535) { "gRPC port must be between 1 and 65535" }
            val address = InetAddress.getByName("127.0.0.1")
            require(address.isLoopbackAddress) { "gRPC server address must be loopback" }
            return InetSocketAddress(address, port)
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
private fun structuredStatus(
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
private inline fun <T> StreamObserver<T>.respond(block: () -> T) {
    try {
        onNext(block())
        onCompleted()
    } catch (exception: IllegalArgumentException) {
        onError(
            structuredStatus(
                Status.INVALID_ARGUMENT,
                ErrorCode.ERROR_CODE_INVALID_ARGUMENT,
                exception.message ?: "invalid argument",
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
        systemGrpcService.ping(request, responseObserver)

    override fun echoBytes(request: EchoBytesRequest, responseObserver: StreamObserver<EchoBytesResponse>) =
        systemGrpcService.echoBytes(request, responseObserver)

    override fun serverInfo(request: ServerInfoRequest, responseObserver: StreamObserver<ServerInfoResponse>) =
        systemGrpcService.serverInfo(request, responseObserver)

    override fun proxyHistory(
        request: ProxyHistoryRequest,
        responseObserver: StreamObserver<ProxyHistoryResponse>,
    ) {
        try {
            val limit =
                when {
                    !request.hasPage() || request.page.limit == 0 -> 100
                    request.page.limit > GRPC_MAX_PAGE_SIZE -> {
                        responseObserver.onError(
                            Status.INVALID_ARGUMENT
                                .withDescription("page limit must be at most $GRPC_MAX_PAGE_SIZE")
                                .asRuntimeException(),
                        )
                        return
                    }
                    else -> request.page.limit.toInt()
                }
            val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "proxy history cursor")
            val page =
                proxyFacade.history(
                    ProxyHistoryQuery(
                        limit = limit,
                        offset = offset,
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
                if (Context.current().isCancelled) {
                    responseObserver.onError(Status.CANCELLED.asRuntimeException())
                    return
                }
                val protoItem =
                    ProxyHistoryEntry
                        .newBuilder()
                        .setIndex(item.index)
                        .setMethod(item.method)
                        .setUrl(item.url)
                        .setStatus(item.status ?: 0)
                        .setLength(item.length?.toLong() ?: 0)
                        .setHasResponse(item.hasResponse)
                        .setNotes(item.notes ?: "")
                        .setHighlight(item.highlight ?: "")
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
            responseObserver.onNext(builder.build())
            responseObserver.onCompleted()
        } catch (exception: Exception) {
            responseObserver.onError(Status.INTERNAL.withDescription("unable to read proxy history").withCause(exception).asRuntimeException())
        }
    }

    override fun proxyDetail(
        request: ProxyDetailRequest,
        responseObserver: StreamObserver<ProxyDetailResponse>,
    ) {
        val detail = proxyFacade.detail(request.index)
        if (detail == null) {
            responseObserver.onError(
                structuredStatus(Status.NOT_FOUND, ErrorCode.ERROR_CODE_NOT_FOUND, "proxy history index ${request.index} was not found"),
            )
            return
        }
        val response =
            ProxyDetailResponse
                .newBuilder()
                .setIndex(detail.index)
                .setRequest(com.google.protobuf.ByteString.copyFromUtf8(detail.request))
                .setNotes(detail.notes ?: "")
                .setHighlight(detail.highlight ?: "")
        detail.response?.let { response.setResponse(com.google.protobuf.ByteString.copyFromUtf8(it)) }
        responseObserver.onNext(response.build())
        responseObserver.onCompleted()
    }

    override fun sitemapSnapshot(
        request: SitemapSnapshotRequest,
        responseObserver: StreamObserver<SitemapSnapshotResponse>,
    ) {
        val limit =
            when {
                !request.hasPage() || request.page.limit == 0 -> 200
                request.page.limit > GRPC_MAX_PAGE_SIZE -> {
                    responseObserver.onError(
                        structuredStatus(Status.INVALID_ARGUMENT, ErrorCode.ERROR_CODE_INVALID_ARGUMENT, "page limit must be at most $GRPC_MAX_PAGE_SIZE"),
                    )
                    return
                }
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
        responseObserver.onNext(response.build())
        responseObserver.onCompleted()
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
    ) {
        val limit = if (request.limit == 0) 500 else request.limit.toInt().coerceAtMost(500)
        val info = targetFacade.info(request.urlPrefix, limit)
        responseObserver.onNext(
            TargetInfoResponse
                .newBuilder()
                .addAllHosts(info.hosts)
                .addAllTechnologies(info.technologies)
                .setRequestsSampled(info.requestsSampled)
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun scopeCheck(
        request: ScopeCheckRequest,
        responseObserver: StreamObserver<ScopeCheckResponse>,
    ) {
        val scope = targetFacade.scope(request.url)
        responseObserver.onNext(
            ScopeCheckResponse
                .newBuilder()
                .setUrl(scope.url)
                .setInScope(scope.inScope)
                .build(),
        )
        responseObserver.onCompleted()
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
    ) {
        val item = scannerFacade.issueDetail(request.index.toInt())
        responseObserver.onNext(
            ScanIssueEntry.newBuilder()
                .setIndex(item.index)
                .setName(item.name)
                .setSeverity(item.severity)
                .setConfidence(item.confidence)
                .setUrl(item.url)
                .setDetail(item.detail)
                .build(),
        )
        responseObserver.onCompleted()
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
    ) {
        val report =
            scannerFacade.generateReport(
                format = request.format,
                path = request.path,
                issueIndexes = request.issueIndexesList.map { it.toInt() },
            )
        responseObserver.onNext(
            GenerateScannerReportResponse.newBuilder()
                .setPath(report.path)
                .setFormat(report.format)
                .setIssueCount(report.issueCount)
                .setSizeBytes(report.sizeBytes)
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun cookieJar(
        request: CookieJarRequest,
        responseObserver: StreamObserver<CookieJarResponse>,
    ) {
        val limit = if (request.limit == 0) 100 else request.limit.toInt()
        if (limit > GRPC_MAX_PAGE_SIZE) {
            responseObserver.onError(
                structuredStatus(
                    Status.INVALID_ARGUMENT,
                    ErrorCode.ERROR_CODE_INVALID_ARGUMENT,
                    "cookie limit must be at most $GRPC_MAX_PAGE_SIZE",
                ),
            )
            return
        }
        val cookies = cookieFacade.cookies(CookieQuery(request.domain.takeIf(String::isNotEmpty), limit))
        responseObserver.onNext(
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
                ).build(),
        )
        responseObserver.onCompleted()
    }

    override fun setCookie(
        request: SetCookieRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        cookieFacade.setCookie(
            request.name,
            request.value,
            request.domain,
            request.path.ifBlank { "/" },
            request.expiration.ifBlank { null },
        )
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("cookie updated").build())
        responseObserver.onCompleted()
    }

    override fun interceptState(
        request: InterceptStateRequest,
        responseObserver: StreamObserver<InterceptStateResponse>,
    ) {
        val enabled = proxyFacade.interceptState(if (request.hasEnabled()) request.enabled else null)
        responseObserver.onNext(InterceptStateResponse.newBuilder().setEnabled(enabled).build())
        responseObserver.onCompleted()
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
        ProxyInterceptConfigResponse.newBuilder()
            .setMasterInterceptEnabled(config.masterInterceptEnabled)
            .setRequestDoIntercept(config.requestDoIntercept)
            .setRequestAutoContentLength(config.requestAutoContentLength)
            .setRequestFixMissingNewLines(config.requestFixMissingNewLines)
            .setResponseDoIntercept(config.responseDoIntercept)
            .setResponseAutoContentLength(config.responseAutoContentLength)
            .setWebsocketClientToServer(config.websocketClientToServer)
            .setWebsocketServerToClient(config.websocketServerToClient)
            .setWebsocketInScopeOnly(config.websocketInScopeOnly)
            .addAllRequestRules(config.requestRules.map { it.toProto() })
            .addAllResponseRules(config.responseRules.map { it.toProto() })
            .setResponseUnhideHiddenFields(config.responseUnhideHiddenFields)
            .setResponseEnableDisabledFields(config.responseEnableDisabledFields)
            .setResponseRemoveInputLengthLimits(config.responseRemoveInputLengthLimits)
            .setResponseRemoveJavascriptValidation(config.responseRemoveJavaScriptValidation)
            .setResponseRemoveAllJavascript(config.responseRemoveAllJavaScript)
            .build()
    }

    override fun proxyWebSocketHistory(
        request: ProxyWebSocketHistoryRequest,
        responseObserver: StreamObserver<ProxyWebSocketHistoryResponse>,
    ) = responseObserver.respond {
        val limit = if (!request.hasPage() || request.page.limit == 0) 50 else request.page.limit.coerceAtMost(GRPC_MAX_PAGE_SIZE)
        val offset = parseCursor(if (request.hasPage()) request.page.cursor else "", "WebSocket cursor")
        val page = proxyFacade.webSocketHistory(limit, offset)
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
    ) {
        capabilityFacade.sendToIntruder(
            request.request.toByteArray(),
            request.host,
            request.port.toInt(),
            request.https,
            request.tabName.ifBlank { null },
        )
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("request opened in Intruder").build())
        responseObserver.onCompleted()
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
    ) {
        responseObserver.onNext(
            ListPayloadProcessorsResponse.newBuilder().addAllItems(
                intruderPayloadFacade.listProcessors().map { item ->
                    PayloadProcessorEntry.newBuilder()
                        .setId(item.id)
                        .setDisplayName(item.displayName)
                        .setOperation(item.operation)
                        .setRegistered(item.registered)
                        .build()
                },
            ).build(),
        )
        responseObserver.onCompleted()
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
    ) {
        responseObserver.onNext(
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
            ).build(),
        )
        responseObserver.onCompleted()
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
    ) {
        val info = capabilityFacade.extensionInfo()
        responseObserver.onNext(
            ExtensionInfoResponse.newBuilder()
                .setFilename(info.filename)
                .setIsBapp(info.isBapp)
                .addAllCommandLineArguments(info.commandLineArguments)
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun createPayloadList(request: CreatePayloadListRequest, responseObserver: StreamObserver<PayloadListEntry>) = responseObserver.respond {
        payloadListFacade.create(request.id, request.displayName, request.payloadsList).toProto()
    }

    override fun importPayloadList(request: ImportPayloadListRequest, responseObserver: StreamObserver<PayloadListEntry>) = responseObserver.respond {
        payloadListFacade.import(request.id, request.displayName, request.content, request.format, request.keepEmpty).toProto()
    }

    override fun listPayloadLists(request: ListPayloadListsRequest, responseObserver: StreamObserver<ListPayloadListsResponse>) {
        responseObserver.onNext(ListPayloadListsResponse.newBuilder().addAllItems(payloadListFacade.list().map { it.toProto() }).build())
        responseObserver.onCompleted()
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
    ) {
        val exchange = httpFacade.send(request.toSpec())
        responseObserver.onNext(exchange.toProto())
        responseObserver.onCompleted()
    }

    override fun sendRequests(
        request: SendRequestsRequest,
        responseObserver: StreamObserver<SendRequestsResponse>,
    ) {
        if (request.requestsCount > 32) {
            responseObserver.onError(
                structuredStatus(Status.INVALID_ARGUMENT, ErrorCode.ERROR_CODE_INVALID_ARGUMENT, "at most 32 requests may be sent in one batch"),
            )
            return
        }
        responseObserver.onNext(
            SendRequestsResponse
                .newBuilder()
                .addAllResponses(httpFacade.sendParallel(request.requestsList.map { it.toSpec() }).map { it.toProto() })
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun sendToRepeater(
        request: SendToRepeaterRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        val port = if (request.port == 0) if (request.https) 443 else 80 else request.port.toInt()
        httpFacade.sendToRepeater(
            request.request.toStringUtf8(),
            request.host,
            port,
            request.https,
            request.tabName.takeIf(String::isNotEmpty),
        )
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("request opened in Repeater").build())
        responseObserver.onCompleted()
    }

    override fun setHighlight(
        request: SetHighlightRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        val color = annotationFacade.highlight(request.index, request.color.takeIf(String::isNotEmpty))
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage(color).build())
        responseObserver.onCompleted()
    }

    override fun setNote(
        request: SetNoteRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        annotationFacade.annotate(request.index, request.note)
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("note updated").build())
        responseObserver.onCompleted()
    }

    override fun inspectConfig(
        request: ExportConfigRequest,
        responseObserver: StreamObserver<InspectConfigResponse>,
    ) {
        val inspection = configFacade.inspect(request.pathsList)
        responseObserver.onNext(
            InspectConfigResponse.newBuilder()
                .setConfig(inspection.config)
                .addAllPaths(inspection.paths)
                .setSizeBytes(inspection.sizeBytes)
                .build(),
        )
        responseObserver.onCompleted()
    }

    override fun mutateScope(
        request: MutateScopeRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        if (request.include) targetFacade.include(request.url) else targetFacade.exclude(request.url)
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage(if (request.include) "included" else "excluded").build())
        responseObserver.onCompleted()
    }

    override fun exportConfig(
        request: ExportConfigRequest,
        responseObserver: StreamObserver<ConfigResponse>,
    ) {
        responseObserver.onNext(ConfigResponse.newBuilder().setConfig(configFacade.export(request.pathsList)).build())
        responseObserver.onCompleted()
    }

    override fun importConfig(
        request: ImportConfigRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        configFacade.import(request.config)
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("configuration imported").build())
        responseObserver.onCompleted()
    }

    override fun registerHttpHandler(
        request: RegisterHttpHandlerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        httpHandlerFacade.register(
            HttpHandlerRule(
                request.headerName.takeIf(String::isNotEmpty),
                request.headerValue.takeIf(String::isNotEmpty),
                request.match.takeIf(String::isNotEmpty),
                request.replacement.takeIf(String::isNotEmpty),
            ),
        )
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("HTTP handler registered").build())
        responseObserver.onCompleted()
    }

    override fun clearHttpHandler(
        @Suppress("UNUSED_PARAMETER") request: ClearHttpHandlerRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        httpHandlerFacade.clear()
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("HTTP handlers cleared").build())
        responseObserver.onCompleted()
    }

    override fun registerProxyRule(
        request: RegisterProxyRuleRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
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
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("proxy rule registered").build())
        responseObserver.onCompleted()
    }

    override fun listProxyRules(
        @Suppress("UNUSED_PARAMETER") request: ListProxyRulesRequest,
        responseObserver: StreamObserver<ListProxyRulesResponse>,
    ) {
        responseObserver.onNext(
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
                ).build(),
        )
        responseObserver.onCompleted()
    }

    override fun clearProxyRules(
        request: ClearProxyRulesRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        proxyRuleFacade.clear(request.id.takeIf(String::isNotEmpty))
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("proxy rules cleared").build())
        responseObserver.onCompleted()
    }

    override fun createSessionRule(
        request: CreateSessionRuleRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        sessionRuleFacade.create(
            SessionRule(
                description = request.description.ifBlank { "Burp MCP session rule" },
                actionType = request.actionType.ifBlank { "replace_text" },
                find = request.find,
                replacement = request.replacement,
                headerName = request.headerName,
                parameterName = request.parameterName,
                macroDescription = request.macroDescription,
                urlContains = request.urlContains,
                tools = request.toolsList.map(String::lowercase).toSet(),
                enabled = request.enabled,
            ),
        )
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("session rule created").build())
        responseObserver.onCompleted()
    }

    override fun listSessionRules(
        @Suppress("UNUSED_PARAMETER") request: ListSessionRulesRequest,
        responseObserver: StreamObserver<ListSessionRulesResponse>,
    ) {
        val response =
            ListSessionRulesResponse
                .newBuilder()
                .addAllItems(
                    sessionRuleFacade.list().map { rule ->
                        SessionRuleEntry.newBuilder()
                            .setFind(rule.find)
                            .setReplacement(rule.replacement)
                            .setDescription(rule.description)
                            .setActionType(rule.actionType)
                            .setHeaderName(rule.headerName)
                            .setParameterName(rule.parameterName)
                            .setMacroDescription(rule.macroDescription)
                            .setUrlContains(rule.urlContains)
                            .addAllTools(rule.tools)
                            .setEnabled(rule.enabled)
                            .build()
                    },
                ).build()
        responseObserver.onNext(response)
        responseObserver.onCompleted()
    }

    override fun removeSessionRules(
        @Suppress("UNUSED_PARAMETER") request: RemoveSessionRulesRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        sessionRuleFacade.remove()
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("session rules removed").build())
        responseObserver.onCompleted()
    }

    override fun createMacro(
        request: CreateMacroRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        val macro = macroFacade.create(request.macro.toDomain())
        responseObserver.onNext(
            ActionResponse.newBuilder().setSuccess(true).setMessage(macro.serialNumber.toString()).build(),
        )
        responseObserver.onCompleted()
    }

    override fun listMacros(
        @Suppress("UNUSED_PARAMETER") request: ListMacrosRequest,
        responseObserver: StreamObserver<ListMacrosResponse>,
    ) {
        responseObserver.onNext(
            ListMacrosResponse.newBuilder().addAllMacros(macroFacade.list().map { it.toProto() }).build(),
        )
        responseObserver.onCompleted()
    }

    override fun runMacro(
        request: RunMacroRequest,
        responseObserver: StreamObserver<RunMacroResponse>,
    ) {
        responseObserver.onNext(
            RunMacroResponse.newBuilder().addAllItems(
                macroFacade.run(request.description).map { exchange ->
                    RunMacroItem.newBuilder()
                        .setRequest(exchange.request)
                        .setResponse(exchange.response.orEmpty())
                        .setStatusCode(exchange.status ?: 0)
                        .setHasResponse(exchange.response != null)
                        .build()
                },
            ).build(),
        )
        responseObserver.onCompleted()
    }

    override fun removeMacro(
        request: RemoveMacroRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        val removed = macroFacade.remove(request.description)
        responseObserver.onNext(
            ActionResponse.newBuilder().setSuccess(removed).setMessage(if (removed) "macro removed" else "macro not found").build(),
        )
        responseObserver.onCompleted()
    }

    override fun startConcurrentRequestCheck(
        request: StartConcurrentRequestCheckRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) {
        val port = request.port.toInt().takeIf { it > 0 } ?: if (request.https) 443 else 80
        val snapshot =
            longOperationFacade.startRace(
                request.request.toStringUtf8(),
                request.host,
                port,
                request.https,
                request.count.toInt().takeIf { it > 0 } ?: 10,
            )
        responseObserver.onNext(snapshot.toStatusProto())
        responseObserver.onCompleted()
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
        longOperationFacade.startInlineFuzzer(
            request.template.toStringUtf8(),
            request.host,
            port,
            request.https,
            request.marker.ifEmpty { "FUZZ" },
            inputs,
        ).toStatusProto()
    }
    override fun startCrawl(
        request: StartCrawlRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) {
        responseObserver.onNext(longOperationFacade.startCrawl(request.url).toStatusProto())
        responseObserver.onCompleted()
    }

    override fun startAudit(
        request: StartAuditRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) = responseObserver.respond {
        longOperationFacade.startAudit(request.url, request.active).toStatusProto()
    }

    override fun getJobStatus(
        request: GetJobStatusRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) {
        val snapshot = jobFacade.status(request.id)
        if (snapshot == null) {
            responseObserver.onError(Status.NOT_FOUND.withDescription("job not found").asRuntimeException())
            return
        }
        responseObserver.onNext(snapshot.toStatusProto())
        responseObserver.onCompleted()
    }

    override fun cancelJob(
        request: CancelJobRequest,
        responseObserver: StreamObserver<JobStatusResponse>,
    ) {
        val snapshot = jobFacade.cancel(request.id)
        if (snapshot == null) {
            responseObserver.onError(Status.NOT_FOUND.withDescription("job not found").asRuntimeException())
            return
        }
        responseObserver.onNext(snapshot.toStatusProto())
        responseObserver.onCompleted()
    }

    override fun getJobResult(
        request: GetJobResultRequest,
        responseObserver: StreamObserver<JobResultResponse>,
    ) {
        val snapshot = jobFacade.result(request.id)
        if (snapshot == null) {
            responseObserver.onError(Status.NOT_FOUND.withDescription("job not found").asRuntimeException())
            return
        }
        val limit = request.page.limit.toInt().takeIf { it > 0 }?.coerceAtMost(GRPC_MAX_PAGE_SIZE) ?: GRPC_DEFAULT_PAGE_SIZE
        val offset = parseCursor(request.page.cursor, "job result cursor")
        responseObserver.onNext(snapshot.toResultProto(offset, limit))
        responseObserver.onCompleted()
    }

    override fun generateCollaboratorPayloads(
        request: GenerateCollaboratorPayloadsRequest,
        responseObserver: StreamObserver<GenerateCollaboratorPayloadsResponse>,
    ) {
        val payloads = collaboratorFacade.generate(request.count.toInt().takeIf { it > 0 } ?: 1)
        responseObserver.onNext(GenerateCollaboratorPayloadsResponse.newBuilder().addAllPayloads(payloads).build())
        responseObserver.onCompleted()
    }

    override fun pollCollaboratorInteractions(
        request: PollCollaboratorInteractionsRequest,
        responseObserver: StreamObserver<PollCollaboratorInteractionsResponse>,
    ) {
        val items = collaboratorFacade.interactions()
        val limit = request.page.limit.toInt().takeIf { it > 0 }?.coerceAtMost(GRPC_MAX_PAGE_SIZE) ?: GRPC_DEFAULT_PAGE_SIZE
        val offset = parseCursor(request.page.cursor, "Collaborator cursor")
        val end = minOf(offset + limit, items.size)
        val pageItems = if (offset >= items.size) emptyList() else items.subList(offset, end)
        responseObserver.onNext(
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
                            .build()
                    },
                ).setPage(
                    PageInfo.newBuilder()
                        .setTotal(items.size)
                        .setTruncated(end < items.size)
                        .setNextCursor(if (end < items.size) end.toString() else "")
                        .build(),
                ).build(),
        )
        responseObserver.onCompleted()
    }

    override fun createWebSocket(
        request: CreateWebSocketRequest,
        responseObserver: StreamObserver<CreateWebSocketResponse>,
    ) {
        val port = request.port.toInt().takeIf { it > 0 } ?: if (request.https) 443 else 80
        val creation = webSocketFacade.create(request.host, port, request.https, request.path.ifEmpty { "/" })
        responseObserver.onNext(CreateWebSocketResponse.newBuilder().setId(creation.id.orEmpty()).setStatus(creation.status).build())
        responseObserver.onCompleted()
    }

    override fun sendWebSocketText(
        request: SendWebSocketTextRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        webSocketFacade.sendText(request.id, request.text)
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("message sent").build())
        responseObserver.onCompleted()
    }

    override fun managedWebSocketHistory(
        request: ManagedWebSocketHistoryRequest,
        responseObserver: StreamObserver<ManagedWebSocketHistoryResponse>,
    ) {
        val limit = request.page.limit.toInt().takeIf { it > 0 }?.coerceAtMost(GRPC_MAX_PAGE_SIZE) ?: GRPC_DEFAULT_PAGE_SIZE
        val offset = parseCursor(request.page.cursor, "managed WebSocket cursor")
        val page = webSocketFacade.history(request.id.takeIf(String::isNotEmpty), offset, limit)
        val end = page.offset + page.items.size
        responseObserver.onNext(
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
                ).build(),
        )
        responseObserver.onCompleted()
    }

    override fun sendWebSocketBinary(
        request: SendWebSocketBinaryRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        webSocketFacade.sendBinary(request.id, request.data.toByteArray())
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("message sent").build())
        responseObserver.onCompleted()
    }

    override fun closeWebSocket(
        request: CloseWebSocketRequest,
        responseObserver: StreamObserver<ActionResponse>,
    ) {
        webSocketFacade.close(request.id)
        responseObserver.onNext(ActionResponse.newBuilder().setSuccess(true).setMessage("WebSocket closed").build())
        responseObserver.onCompleted()
    }
    override fun importBambda(
        request: ImportBambdaRequest,
        responseObserver: StreamObserver<ScriptImportResponse>,
    ) {
        val result = scriptImportFacade.importBambda(request.script)
        responseObserver.onNext(ScriptImportResponse.newBuilder().setSuccess(result.success).setStatus(result.status).addAllErrors(result.errors).build())
        responseObserver.onCompleted()
    }
    private fun PayloadListDefinition.toProto(): PayloadListEntry = PayloadListEntry.newBuilder()
        .setId(id).setDisplayName(displayName).setPayloadCount(payloads.size).setSizeBytes(sizeBytes).setFingerprint(fingerprint).build()


    override fun importBCheck(
        request: ImportBCheckRequest,
        responseObserver: StreamObserver<ScriptImportResponse>,
    ) {
        val result = scriptImportFacade.importBCheck(request.script, request.enabled)
        responseObserver.onNext(ScriptImportResponse.newBuilder().setSuccess(result.success).setStatus(result.status).addAllErrors(result.errors).build())
        responseObserver.onCompleted()
    }


    override fun listWebSockets(
        @Suppress("UNUSED_PARAMETER") request: ListWebSocketsRequest,
        responseObserver: StreamObserver<ListWebSocketsResponse>,
    ) {
        responseObserver.onNext(ListWebSocketsResponse.newBuilder().addAllIds(webSocketFacade.list()).build())
        responseObserver.onCompleted()
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

    private fun io.github.nguyenthdat.burpmcp.HttpExchange.toProto(): SendRequestResponse =
        SendRequestResponse
            .newBuilder()
            .setRequest(com.google.protobuf.ByteString.copyFromUtf8(request))
            .setResponse(com.google.protobuf.ByteString.copyFromUtf8(response ?: ""))
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

    private fun JobSnapshot.toStatusProto(): JobStatusResponse =
        JobStatusResponse
            .newBuilder()
            .setId(id)
            .setOperation(operation)
            .setState(state.name.lowercase())
            .setError(error.orEmpty())
            .build()

    private fun JobSnapshot.toResultProto(offset: Int, limit: Int): JobResultResponse {
        val builder =
            JobResultResponse
                .newBuilder()
                .setId(id)
                .setOperation(operation)
                .setState(state.name.lowercase())
                .setError(error.orEmpty())
        when (val output = result) {
            is HttpBatchJobOutput -> {
                val end = minOf(offset + limit, output.items.size)
                val pageItems = if (offset >= output.items.size) emptyList() else output.items.subList(offset, end)
                builder
                    .addAllItems(
                        pageItems.map { item ->
                            HttpJobResultItem
                                .newBuilder()
                                .setLabel(item.label)
                                .also { entry -> item.status?.let(entry::setStatus) }
                                .also { entry -> item.length?.let(entry::setLength) }
                                .setError(item.error.orEmpty())
                                .build()
                        },
                    ).setPage(
                        PageInfo
                            .newBuilder()
                            .setTotal(output.items.size)
                            .setTruncated(end < output.items.size)
                            .setNextCursor(if (end < output.items.size) end.toString() else "")
                            .build(),
                    ).setRequestCount(output.requestCount)
                    .setUniqueLengths(output.uniqueLengths)
                    .setVerdict(output.verdict)
                    .setSubstitutionCount(output.substitutionCount)
                    .setRequestFingerprint(output.requestFingerprint)
            }

            is TaskJobOutput -> builder.setRequestCount(output.requestCount).setErrorCount(output.errorCount)
            is AuditJobOutput ->
                builder
                    .setRequestCount(output.requestCount)
                    .setErrorCount(output.errorCount)
                    .setIssueCount(output.issueCount)
            null -> Unit
        }
        return builder.build()
    }

}
