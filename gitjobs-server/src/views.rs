//! This module contains the types and functionality used to track jobs views.

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
static DATE_FORMAT: LazyLock<Vec<FormatItem<'static>>> =
    LazyLock::new(|| format_description::parse("[year]-[month]-[day]").expect("format to be valid"));

/// How often jobs views will be written to the database.
#[cfg(not(test))]
const FLUSH_FREQUENCY: Duration = Duration::from_secs(300);
#[cfg(test)]
const FLUSH_FREQUENCY: Duration = Duration::from_millis(100);

/// Type alias to represent a `ViewsTracker` trait object.
pub(crate) type DynViewsTracker = Arc<dyn ViewsTracker + Send + Sync>;

/// Type alias to represent a job id.
pub(crate) type JobId = Uuid;

/// Type alias to represent a day in `DATE_FORMAT`.
pub(crate) type Day = String;

/// Type alias to represent a views counter.
pub(crate) type Total = u32;

/// Type alias to represent a views batch.
type Batch = HashMap<BatchKey, Total>;

/// Views batch key, which is a combination of the job id and the day.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BatchKey {
    job_id: JobId,
    day: Day,
}

/// Trait that defines some operations a ViewsTracker implementation must
/// support.
#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait ViewsTracker {
    /// Track a view for the job provided.
    async fn track_view(&self, job_id: JobId) -> Result<()>;
}

/// `ViewsTracker` backed by `PostgreSQL`.
pub(crate) struct ViewsTrackerDB {
    views_tx: mpsc::Sender<JobId>,
}

impl ViewsTrackerDB {
    /// Create a new `ViewsTrackerDB` instance.
    pub(crate) fn new(db: DynDBViews, tracker: &TaskTracker, cancellation_token: &CancellationToken) -> Self {
        // Setup channels
        let (views_tx, views_rx) = mpsc::channel(100);
        let (batches_tx, batches_rx) = mpsc::channel(5);

        // Setup workers
        tracker.spawn(aggregator(views_rx, batches_tx, cancellation_token.clone()));
        tracker.spawn(flusher(db, batches_rx));

        Self { views_tx }
    }
}

#[async_trait]
impl ViewsTracker for ViewsTrackerDB {
    async fn track_view(&self, job_id: JobId) -> Result<()> {
        self.views_tx.send(job_id).await.map_err(Into::into)
    }
}

/// Worker that aggregates the views received on the views channel, passing the
/// resulting batches to the flusher periodically.
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

            // Send batch to flusher every FLUSH_FREQUENCY
            _ = flush_interval.tick() => {
                if !batch.is_empty() && batches_tx.send(batch.clone()).await.is_ok() {
                    batch.clear();
                }
            }

            // Pick next view from queue and aggregate it
            Some(job_id) = views_rx.recv() => {
                let key = BatchKey {
                    job_id,
                    day: OffsetDateTime::now_utc().format(&DATE_FORMAT).expect("format to succeed"),
                };
                *batch.entry(key).or_default() += 1;
            }

            // Exit if the aggregator has been asked to stop
            () = cancellation_token.cancelled() => {
                if !batch.is_empty() {
                    _ = batches_tx.send(batch).await;
                }
                break
            }
        }
    }
}

/// Worker that stores the views batches received from the aggregator into
/// the database.
async fn flusher(db: DynDBViews, mut batches_rx: mpsc::Receiver<Batch>) {
    while let Some(batch) = batches_rx.recv().await {
        // Prepare batch data for database update
        let mut data: Vec<(JobId, Day, Total)> = batch
            .iter()
            .map(|(key, total)| (key.job_id, key.day.clone(), *total))
            .collect();
        data.sort();

        // Write data to database
        if let Err(err) = db.update_jobs_views(data).await {
            error!(?err, "error writing jobs views to database");
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::future;
    use mockall::predicate::eq;
    use tokio::time::{Duration, sleep};

    use crate::db::views::MockDBViews;

    use super::*;

    static JOB1_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    static JOB2_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap());

    #[tokio::test]
    async fn flush_on_stop() {
        // Setup mock database
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDBViews::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Box::pin(future::ready(Ok(()))));
        let mock_db = Arc::new(mock_db);

        // Setup views tracker and track some views
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let vt = ViewsTrackerDB::new(mock_db, &tracker, &cancellation_token);
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB2_ID).await.unwrap();

        // Stop the tracker and wait for the workers to complete
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }

    #[tokio::test]
    async fn flush_periodically() {
        // Setup mock database
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDBViews::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Box::pin(future::ready(Ok(()))));
        let mock_db = Arc::new(mock_db);

        // Setup views tracker and track some views
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let vt = ViewsTrackerDB::new(mock_db, &tracker, &cancellation_token);
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB1_ID).await.unwrap();
        vt.track_view(*JOB2_ID).await.unwrap();

        // Wait for the periodic flush to complete
        sleep(Duration::from_millis(500)).await;
    }

    #[tokio::test]
    async fn no_views_tracked_nothing_to_flush() {
        // Setup views tracker (no views tracked)
        let mock_db = Arc::new(MockDBViews::new());
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let _ = ViewsTrackerDB::new(mock_db, &tracker, &cancellation_token);

        // Stop the tracker and wait for the workers to complete
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }
}
