package io.github.nguyenthdat.burpmcp.rpc

internal object RpcLimits {
    const val MAX_MESSAGE_BYTES: Int = 16 * 1024 * 1024
    const val MAX_PAGE_SIZE: Int = 500
    const val MAX_METADATA_BYTES: Int = 8 * 1024
    const val MAX_CONCURRENT_CALLS_PER_CONNECTION: Int = 32
    const val MAX_RPC_TIMEOUT_SECONDS: Long = 30
    const val MAX_RESPONSE_BYTES: Int = 16 * 1024 * 1024
    const val DEFAULT_PAGE_SIZE: Int = 100
    const val RESPONSE_OVERHEAD_BYTES: Int = 64 * 1024
    const val SHUTDOWN_SECONDS: Long = 5
}

internal const val GRPC_MAX_METADATA_BYTES: Int = RpcLimits.MAX_METADATA_BYTES
internal const val GRPC_MAX_MESSAGE_BYTES: Int = RpcLimits.MAX_MESSAGE_BYTES
internal const val GRPC_MAX_PAGE_SIZE: Int = RpcLimits.MAX_PAGE_SIZE
internal const val GRPC_MAX_CONCURRENT_CALLS_PER_CONNECTION: Int = RpcLimits.MAX_CONCURRENT_CALLS_PER_CONNECTION
internal const val GRPC_MAX_RPC_TIMEOUT_SECONDS: Long = RpcLimits.MAX_RPC_TIMEOUT_SECONDS
internal const val GRPC_MAX_RESPONSE_BYTES: Int = RpcLimits.MAX_RESPONSE_BYTES
