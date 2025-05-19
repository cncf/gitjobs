//! This module contains the types and functionality used to track job views.
//!
//! It provides an asynchronous, batched mechanism for tracking and persisting job view
//! counts to the database. Views are aggregated in memory and flushed periodically or on
//! shutdown, minimizing database writes and improving performance.

use std::{collections::HashMap, sync::Arc, sync::LazyLock, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use time::{
    OffsetDateTime,
    format_description::{self, FormatItem},
};
use tokio::{
    sync::mpsc,
    time::{Instant, MissedTickBehavior},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;
use uuid::Uuid;

use crate::db::views::DynDBViews;

/// Format used to represent the date in the views tracker.
/// The format is `[year]-[month]-[day]`, e.g., "2024-06-01".
static DATE_FORMAT: LazyLock<Vec<FormatItem<'static>>> =
    LazyLock::new(|| format_description::parse("[year]-[month]-[day]").expect("format to be valid"));

/// How often job views will be written to the database.
/// In production, this is 5 minutes; in tests, 100ms.
#[cfg(not(test))]
const FLUSH_FREQUENCY: Duration = Duration::from_secs(300);
#[cfg(test)]
const FLUSH_FREQUENCY: Duration = Duration::from_millis(100);

/// Type alias for a thread-safe reference-counted `ViewsTracker` trait object.
pub(crate) type DynViewsTracker = Arc<dyn ViewsTracker + Send + Sync>;

/// Type alias representing a job's unique identifier.
pub(crate) type JobId = Uuid;

/// Type alias representing a day in the format specified by `DATE_FORMAT`.
pub(crate) type Day = String;

/// Type alias representing the total number of views for a job on a given day.
pub(crate) type Total = u32;

/// Type alias representing a batch of aggregated job views.
/// The key is a combination of job ID and day, and the value is the view count.
type Batch = HashMap<BatchKey, Total>;

/// Key for a batch entry, combining a job ID and a day.
/// Used to uniquely identify the view count for a specific job on a specific day.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BatchKey {
    /// The unique identifier of the job.
    job_id: JobId,
    /// The day for which views are being tracked, formatted as `DATE_FORMAT`.
    day: Day,
}

/// Trait defining the interface for tracking job views.
///
/// Implementations are responsible for asynchronously tracking views and ensuring they
/// are eventually persisted to the database.
#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait ViewsTracker {
    /// Track a view for the provided job ID.
    async fn track_view(&self, job_id: JobId) -> Result<()>;
}

/// Implementation of `ViewsTracker` backed by a `PostgreSQL` database.
///
/// Views are sent to an internal channel, aggregated, and periodically flushed to the
/// database in batches.
pub(crate) struct ViewsTrackerDB {
    /// Channel for sending job view events to the aggregator worker.
    views_tx: mpsc::Sender<JobId>,
}

impl ViewsTrackerDB {
    /// Create a new `ViewsTrackerDB` instance.
    ///
    /// Spawns background workers for aggregating and flushing views.
    pub(crate) fn new(db: DynDBViews, tracker: &TaskTracker, cancellation_token: &CancellationToken) -> Self {
        // Setup channels.
        let (views_tx, views_rx) = mpsc::channel(100);
        let (batches_tx, batches_rx) = mpsc::channel(5);

        // Setup workers.
        tracker.spawn(aggregator(views_rx, batches_tx, cancellation_token.clone()));
        tracker.spawn(flusher(db, batches_rx));

        Self { views_tx }
    }
}

#[async_trait]
impl ViewsTracker for ViewsTrackerDB {
    /// Track a view for the provided job ID by sending it to the aggregator.
    async fn track_view(&self, job_id: JobId) -> Result<()> {
        self.views_tx.send(job_id).await.map_err(Into::into)
    }
}

/// Aggregator worker that receives job view events, aggregates them in memory, and
/// periodically sends batches to the flusher.
///
/// Batches are flushed either on a fixed interval or when the system is shutting down.
async fn aggregator(
    mut views_rx: mpsc::Receiver<JobId>,
    batches_tx: mpsc::Sender<Batch>,
    cancellation_token: CancellationToken,
) {
    let first_flush = Instant::now() + FLUSH_FREQUENCY;
    let mut flush_interval = tokio::time::interval_at(first_flush, FLUSH_FREQUENCY);
    flush_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut batch: Batch = HashMap::new();
    loop {
        tokio::select! {
            biased;

            // Send batch to flusher every FLUSH_FREQUENCY.
            _ = flush_interval.tick() => {
                if !batch.is_empty() && batches_tx.send(batch.clone()).await.is_ok() {
                    batch.clear();
                }
            }

            // Pick next view from queue and aggregate it.
            Some(job_id) = views_rx.recv() => {
                let key = BatchKey {
                    job_id,
                    day: OffsetDateTime::now_utc()
                        .format(&DATE_FORMAT)
                        .expect("format to succeed"),
                };
                *batch.entry(key).or_default() += 1;
            }

            // Exit if the aggregator has been asked to stop.
            () = cancellation_token.cancelled() => {
                if !batch.is_empty() {
                    _ = batches_tx.send(batch).await;
                }
                break
            }
        }
    }
}

/// Flusher worker that receives batches of aggregated job views and writes them to the
/// database.
async fn flusher(db: DynDBViews, mut batches_rx: mpsc::Receiver<Batch>) {
    while let Some(batch) = batches_rx.recv().await {
        // Prepare batch data for database update.
        let mut data: Vec<(JobId, Day, Total)> = batch
            .iter()
            .map(|(key, total)| (key.job_id, key.day.clone(), *total))
            .collect();
        data.sort();

        // Write data to database.
        if let Err(err) = db.update_jobs_views(data).await {
            error!(?err, "error writing jobs views to database");
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the views tracking module.
    //!
    //! These tests verify that views are flushed both periodically and on shutdown, and
    //! that no flush occurs if no views are tracked.

    use futures::future;
    use mockall::predicate::eq;
    use tokio::time::{Duration, sleep};

    use crate::db::views::MockDBViews;

    use super::*;

    /// Static job IDs used for testing.
    static JOB1_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    static JOB2_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap());

    /// Test that views are flushed when the tracker is stopped.
    #[tokio::test]
    async fn flush_on_stop() {
        // Setup mock database.
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDBViews::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Box::pin(future::ready(Ok(()))));
        let mock_db = Arc::new(mock_db);

        // Setup views tracker and track some views.
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let vt = ViewsTrackerDB::new(mock_db, &tracker, &cancellation_token);
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB2_ID).await.unwrap();

        // Stop the tracker and wait for the workers to complete.
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }

    /// Test that views are flushed periodically.
    #[tokio::test]
    async fn flush_periodically() {
        // Setup mock database.
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDBViews::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Box::pin(future::ready(Ok(()))));
        let mock_db = Arc::new(mock_db);

        // Setup views tracker and track some views.
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let vt = ViewsTrackerDB::new(mock_db, &tracker, &cancellation_token);
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB2_ID).await.unwrap();

        // Wait for the periodic flush to complete.
        sleep(Duration::from_millis(500)).await;
    }

    /// Test that nothing is flushed if no views are tracked.
    #[tokio::test]
    async fn no_views_tracked_nothing_to_flush() {
        // Setup views tracker (no views tracked).
        let mock_db = Arc::new(MockDBViews::new());
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let _ = ViewsTrackerDB::new(mock_db, &tracker, &cancellation_token);

        // Stop the tracker and wait for the workers to complete.
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }
}
