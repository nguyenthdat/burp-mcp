package io.github.nguyenthdat.burpmcp

import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.Future
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.atomic.AtomicReference

internal enum class JobState {
    QUEUED,
    RUNNING,
    COMPLETED,
    FAILED,
    CANCELLED,
}

internal sealed interface JobOutput

internal data class HttpJobItem(
    val label: String,
    val status: Int?,
    val length: Int?,
    val error: String? = null,
)

internal data class HttpBatchJobOutput(
    val items: List<HttpJobItem>,
    val uniqueLengths: Int,
    val verdict: String,
) : JobOutput {
    val requestCount: Int get() = items.size
}

internal data class TaskJobOutput(
    val requestCount: Int,
    val errorCount: Int,
) : JobOutput
internal data class AuditJobOutput(
    val requestCount: Int,
    val errorCount: Int,
    val issueCount: Int,
) : JobOutput

internal data class JobSnapshot(
    val id: String,
    val operation: String,
    val state: JobState,
    val result: JobOutput? = null,
    val error: String? = null,
)

internal class JobFacade : AutoCloseable {
    private class Record(
        val id: String,
        val operation: String,
        val state: AtomicReference<JobState> = AtomicReference(JobState.QUEUED),
        @Volatile var result: JobOutput? = null,
        @Volatile var error: String? = null,
        @Volatile var future: Future<*>? = null,
    )

    private val ids = AtomicLong()
    private val records = ConcurrentHashMap<String, Record>()
    private val order = ConcurrentLinkedQueue<String>()
    private val executor: ExecutorService = Executors.newFixedThreadPool(8) { runnable ->
        Thread(runnable, "burp-mcp-job").apply { isDaemon = true }
    }

    @Synchronized
    fun start(operation: String, task: () -> JobOutput): JobSnapshot {
        require(operation.isNotBlank()) { "job operation must not be blank" }
        evictTerminalJobs()
        require(records.size < MAX_RETAINED_JOBS) { "too many retained jobs" }
        val record = Record("job-${ids.incrementAndGet()}", operation)
        records[record.id] = record
        order.add(record.id)
        record.future =
            executor.submit {
                if (!record.state.compareAndSet(JobState.QUEUED, JobState.RUNNING)) return@submit
                try {
                    val result = task()
                    if (record.state.compareAndSet(JobState.RUNNING, JobState.COMPLETED)) record.result = result
                } catch (exception: InterruptedException) {
                    Thread.currentThread().interrupt()
                    record.error = "job interrupted"
                    record.state.compareAndSet(JobState.RUNNING, JobState.CANCELLED)
                } catch (exception: Exception) {
                    record.error = exception.message ?: exception::class.simpleName ?: "job failed"
                    record.state.compareAndSet(JobState.RUNNING, JobState.FAILED)
                }
            }
        return snapshot(record)
    }

    fun status(id: String): JobSnapshot? = records[id]?.let(::snapshot)

    fun result(id: String): JobSnapshot? = records[id]?.let { record ->
        when (record.state.get()) {
            JobState.COMPLETED, JobState.FAILED, JobState.CANCELLED -> snapshot(record)
            JobState.QUEUED, JobState.RUNNING -> snapshot(record)
        }
    }

    fun cancel(id: String): JobSnapshot? = records[id]?.let { record ->
        if (record.state.compareAndSet(JobState.QUEUED, JobState.CANCELLED) ||
            record.state.compareAndSet(JobState.RUNNING, JobState.CANCELLED)
        ) {
            record.future?.cancel(true)
        }
        snapshot(record)
    }

    private fun snapshot(record: Record): JobSnapshot =
        JobSnapshot(record.id, record.operation, record.state.get(), record.result, record.error)
    private fun evictTerminalJobs() {
        repeat(order.size) {
            val id = order.poll() ?: return
            val record = records[id] ?: return@repeat
            if (record.state.get() in TERMINAL_STATES) {
                records.remove(id, record)
            } else {
                order.add(id)
            }
        }
    }

    private companion object {
        const val MAX_RETAINED_JOBS = 256
        val TERMINAL_STATES = setOf(JobState.COMPLETED, JobState.FAILED, JobState.CANCELLED)
    }

    override fun close() {
        records.values.forEach { record ->
            record.future?.cancel(true)
            record.state.set(JobState.CANCELLED)
        }
        executor.shutdownNow()
        records.clear()
    }
}
