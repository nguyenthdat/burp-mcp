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

    private fun awaitTerminal(jobs: JobFacade, id: String): JobSnapshot {
        repeat(100) {
            val snapshot = assertNotNull(jobs.status(id))
            if (snapshot.state in setOf(JobState.COMPLETED, JobState.FAILED, JobState.CANCELLED)) return snapshot
            Thread.sleep(10)
        }
        error("job did not reach a terminal state")
    }
}
