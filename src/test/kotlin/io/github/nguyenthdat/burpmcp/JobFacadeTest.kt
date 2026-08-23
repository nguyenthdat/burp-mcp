package io.github.nguyenthdat.burpmcp

import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class JobFacadeTest {
    @Test
    fun `runs a bounded job and retains typed result`() {
        JobFacade().use { jobs ->
            val started = jobs.start("test") { TaskJobOutput(3, 1) }
            val finished = awaitTerminal(jobs, started.id)

            assertEquals(JobState.COMPLETED, finished.state)
            assertEquals(TaskJobOutput(3, 1), finished.result)
        }
    }
    @Test
    fun `starting a new job retains completed results`() {
        JobFacade().use { jobs ->
            val first = jobs.start("first") { TaskJobOutput(1, 0) }
            val completed = awaitTerminal(jobs, first.id)

            jobs.start("second") { TaskJobOutput(1, 0) }

            assertEquals(completed, jobs.result(first.id))
        }
    }


    @Test
    fun `cancels a running job and ignores late completion`() {
        JobFacade().use { jobs ->
            val entered = CountDownLatch(1)
            val release = CountDownLatch(1)
            val started =
                jobs.start("test") {
                    entered.countDown()
                    release.await()
                    TaskJobOutput(1, 0)
                }
            assertTrue(entered.await(2, TimeUnit.SECONDS))

            val cancelled = assertNotNull(jobs.cancel(started.id))
            release.countDown()

            assertEquals(JobState.CANCELLED, cancelled.state)
            assertEquals(JobState.CANCELLED, awaitTerminal(jobs, started.id).state)
        }
    }

    @Test
    fun `scanner jobs fail instead of completing with zero requests`() {
        JobFacade().use { jobs ->
            val started = jobs.start("crawl") {
                awaitTaskCompletion(
                    operation = "crawl",
                    snapshot = { TaskJobOutput(0, 0) },
                    status = { "" },
                    timeoutMillis = 10,
                    stableMillis = 0,
                    pollMillis = 1,
                )
            }

            val finished = awaitTerminal(jobs, started.id)

            assertEquals(JobState.FAILED, finished.state)
            assertEquals("crawl issued no requests before timeout", finished.error)
        }
    }

    @Test
    fun `scanner jobs preserve unsupported errors`() {
        JobFacade().use { jobs ->
            val started = jobs.start("scanner_audit") {
                awaitAuditCompletion(
                    snapshot = { AuditJobOutput(0, 0, 0) },
                    status = { "Currently unsupported." },
                    timeoutMillis = 100,
                    stableMillis = 0,
                    pollMillis = 1,
                )
            }

            val finished = awaitTerminal(jobs, started.id)

            assertEquals(JobState.FAILED, finished.state)
            assertEquals("Currently unsupported.", finished.error)
        }
    }

    @Test
    fun `scanner jobs return settled nonzero results`() {
        val startedAt = System.nanoTime()
        val result =
            awaitTaskCompletion(
                operation = "crawl",
                snapshot = {
                    if (System.nanoTime() - startedAt > TimeUnit.MILLISECONDS.toNanos(5)) TaskJobOutput(3, 0) else TaskJobOutput(0, 0)
                },
                status = { "running" },
                timeoutMillis = 100,
                stableMillis = 5,
                pollMillis = 1,
            )

        assertEquals(TaskJobOutput(3, 0), result)
    }
    @Test
    fun `crawl falls back to observed scanner traffic when Montoya count remains zero`() {
        val startedAt = System.nanoTime()
        val result =
            awaitTaskCompletion(
                operation = "crawl",
                snapshot = { TaskJobOutput(0, 0) },
                observedRequestCount = {
                    if (System.nanoTime() - startedAt > TimeUnit.MILLISECONDS.toNanos(5)) 1 else 0
                },
                status = { null },
                timeoutMillis = 100,
                stableMillis = 5,
                pollMillis = 1,
            )

        assertEquals(TaskJobOutput(1, 0), result)
    }
    @Test
    fun `HTTP batch request count equals executed items`() {
        val output =
            HttpBatchJobOutput(
                items = listOf(HttpJobItem("one", 200, 1), HttpJobItem("two", 500, 2)),
                uniqueLengths = 2,
                verdict = "completed",
            )

        assertEquals(2, output.requestCount)
    }

    @Test
    fun `scanner audits use observed traffic when Montoya accessors fail`() {
        val startedAt = System.nanoTime()
        val result =
            awaitAuditCompletion(
                snapshot = { AuditJobOutput(0, 0, 0) },
                observedRequestCount = {
                    if (System.nanoTime() - startedAt > TimeUnit.MILLISECONDS.toNanos(5)) 2 else 0
                },
                status = { null },
                timeoutMillis = 100,
                stableMillis = 5,
                pollMillis = 1,
            )

        assertEquals(AuditJobOutput(2, 0, 0), result)
    }



    private fun awaitTerminal(jobs: JobFacade, id: String): JobSnapshot {
        repeat(100) {
            val snapshot = assertNotNull(jobs.status(id))
            if (snapshot.state in setOf(JobState.COMPLETED, JobState.FAILED, JobState.CANCELLED)) return snapshot
            Thread.sleep(10)
        }
        error("job did not reach a terminal state")
    }
}
