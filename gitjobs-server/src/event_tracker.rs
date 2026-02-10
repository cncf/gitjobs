//! This module contains the types and functionality used to track various events.
//!
//! It provides an asynchronous, batched mechanism for tracking and persisting event
//! counts to the database. Events are aggregated in memory and flushed periodically or
//! on shutdown, minimizing database writes and improving performance.

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

use crate::db::event_tracker::DynDBEventTracker;

/// Format used to represent the date in the tracker.
/// The format is `[year]-[month]-[day]`, e.g., "2024-06-01".
static DATE_FORMAT: LazyLock<Vec<FormatItem<'static>>> =
    LazyLock::new(|| format_description::parse("[year]-[month]-[day]").expect("format to be valid"));

/// How often events will be written to the database.
/// In production, this is 5 minutes; in tests, 100ms.
#[cfg(not(test))]
const FLUSH_FREQUENCY: Duration = Duration::from_secs(300);
#[cfg(test)]
const FLUSH_FREQUENCY: Duration = Duration::from_millis(100);

/// Type alias for a thread-safe reference-counted `EventTracker` trait object.
pub(crate) type DynEventTracker = Arc<dyn EventTracker + Send + Sync>;

/// Type alias representing a job's unique identifier.
pub(crate) type JobId = Uuid;

/// Type alias representing a day in the format specified by `DATE_FORMAT`.
pub(crate) type Day = String;

/// Type alias representing the total number of events for a job on a given day.
pub(crate) type Total = u32;

/// Container for batches of aggregated events, separated by event type.
#[derive(Debug, Clone)]
struct Batches {
    /// Aggregated job view events.
    job_views: HashMap<(JobId, Day), Total>,
    /// Aggregated search appearance events.
    search_appearances: HashMap<(JobId, Day), Total>,
}

impl Batches {
    /// Creates a new empty Batches container.
    fn new() -> Self {
        Self {
            job_views: HashMap::new(),
            search_appearances: HashMap::new(),
        }
    }

    /// Returns true if both containers are empty.
    fn is_empty(&self) -> bool {
        self.job_views.is_empty() && self.search_appearances.is_empty()
    }

    /// Clears both containers.
    fn clear(&mut self) {
        self.job_views.clear();
        self.search_appearances.clear();
    }
}

/// Represents different types of events that can be tracked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    /// A single job view event.
    JobView { job_id: JobId },
    /// Multiple jobs appearing in search results.
    SearchAppearances { job_ids: Vec<JobId> },
}

/// Trait defining the interface for tracking events.
///
/// Implementations are responsible for asynchronously tracking events and ensuring they
/// are eventually persisted to the database.
#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait EventTracker {
    /// Track an event.
    async fn track(&self, event: Event) -> Result<()>;
}

/// Implementation of `EventTracker` backed by a `PostgreSQL` database.
///
/// Events are sent to an internal channel, aggregated, and periodically flushed to the
/// database in batches.
pub(crate) struct EventTrackerDB {
    /// Channel for sending events to the aggregator worker.
    events_tx: mpsc::Sender<Event>,
}

impl EventTrackerDB {
    /// Create a new `EventTrackerDB` instance.
    ///
    /// Spawns background workers for aggregating and flushing events.
    pub(crate) fn new(
        db: DynDBEventTracker,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
    ) -> Self {
        // Setup channels.
        let (events_tx, events_rx) = mpsc::channel(100);
        let (batches_tx, batches_rx) = mpsc::channel(5);

        // Setup workers.
        task_tracker.spawn(aggregator(events_rx, batches_tx, cancellation_token.clone()));
        task_tracker.spawn(flusher(db, batches_rx));

        Self { events_tx }
    }
}

#[async_trait]
impl EventTracker for EventTrackerDB {
    /// Track an event by sending it to the aggregator.
    async fn track(&self, event: Event) -> Result<()> {
        self.events_tx.send(event).await.map_err(Into::into)
    }
}

/// Aggregator worker that receives events, aggregates them in memory, and
/// periodically sends batches to the flusher.
///
/// Batches are flushed either on a fixed interval or when the system is shutting down.
async fn aggregator(
    mut events_rx: mpsc::Receiver<Event>,
    batches_tx: mpsc::Sender<Batches>,
    cancellation_token: CancellationToken,
) {
    let first_flush = Instant::now() + FLUSH_FREQUENCY;
    let mut flush_interval = tokio::time::interval_at(first_flush, FLUSH_FREQUENCY);
    flush_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut batches = Batches::new();
    loop {
        tokio::select! {
            biased;

            // Send batch to flusher every FLUSH_FREQUENCY.
            _ = flush_interval.tick() => {
                if !batches.is_empty() && batches_tx.send(batches.clone()).await.is_ok() {
                    batches.clear();
                }
            }

            // Pick next event from queue and aggregate it.
            Some(event) = events_rx.recv() => {
                let day = OffsetDateTime::now_utc()
                    .format(&DATE_FORMAT)
                    .expect("format to succeed");

                match event {
                    Event::JobView { job_id } => {
                        *batches.job_views.entry((job_id, day)).or_default() += 1;
                    }
                    Event::SearchAppearances { job_ids } => {
                        for job_id in job_ids {
                            *batches.search_appearances
                                .entry((job_id, day.clone()))
                                .or_default() += 1;
                        }
                    }
                }
            }

            // Exit if the aggregator has been asked to stop.
            () = cancellation_token.cancelled() => {
                if !batches.is_empty() {
                    _ = batches_tx.send(batches).await;
                }
                break
            }
        }
    }
}

/// Flusher worker that receives batches of aggregated events and writes them to the
/// database.
async fn flusher(db: DynDBEventTracker, mut batches_rx: mpsc::Receiver<Batches>) {
    while let Some(batches) = batches_rx.recv().await {
        // Process job views.
        if !batches.job_views.is_empty() {
            let job_views = prepare_batch_data(&batches.job_views);
            if let Err(err) = db.update_jobs_views(job_views).await {
                error!(?err, "error writing job views to database");
            }
        }

        // Process search appearances.
        if !batches.search_appearances.is_empty() {
            let search_appearances = prepare_batch_data(&batches.search_appearances);
            if let Err(err) = db.update_search_appearances(search_appearances).await {
                error!(?err, "error writing search appearances to database");
            }
        }
    }
}

/// Converts a `HashMap` of aggregated events into a sorted vector ready for database
/// insertion.
fn prepare_batch_data(data: &HashMap<(JobId, Day), Total>) -> Vec<(JobId, Day, Total)> {
    let mut db_ready_data: Vec<(JobId, Day, Total)> = data
        .iter()
        .map(|((job_id, day), total)| (*job_id, day.clone(), *total))
        .collect();
    db_ready_data.sort();
    db_ready_data
}

#[cfg(test)]
mod tests {
    //! Tests for the event tracking module.
    //!
    //! These tests verify that events are flushed both periodically and on shutdown, and
    //! that no flush occurs if no events are tracked.

    use mockall::predicate::eq;
    use tokio::time::{Duration, sleep};

    use crate::db::mock::MockDB;

    use super::*;

    /// Static job IDs used for testing.
    static JOB1_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap());
    static JOB2_ID: LazyLock<Uuid> =
        LazyLock::new(|| Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap());

    /// Test that job view events are flushed when the tracker is stopped.
    #[tokio::test]
    async fn flush_job_views_on_stop() {
        // Setup mock database.
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDB::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Ok(()));
        let mock_db = Arc::new(mock_db);

        // Setup tracker and track some job views.
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let t = EventTrackerDB::new(mock_db, &tracker, &cancellation_token);
        t.track(Event::JobView { job_id: *JOB1_ID }).await.unwrap();
        t.track(Event::JobView { job_id: *JOB1_ID }).await.unwrap();
        t.track(Event::JobView { job_id: *JOB2_ID }).await.unwrap();

        // Stop the tracker and wait for the workers to complete.
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }

    /// Test that job view events are flushed periodically.
    #[tokio::test]
    async fn flush_job_views_periodically() {
        // Setup mock database.
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDB::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Ok(()));
        let mock_db = Arc::new(mock_db);

        // Setup tracker and track some job views.
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let t = EventTrackerDB::new(mock_db, &tracker, &cancellation_token);
        t.track(Event::JobView { job_id: *JOB1_ID }).await.unwrap();
        t.track(Event::JobView { job_id: *JOB1_ID }).await.unwrap();
        t.track(Event::JobView { job_id: *JOB2_ID }).await.unwrap();

        // Wait for the periodic flush to complete.
        sleep(Duration::from_millis(500)).await;
    }

    /// Test that nothing is flushed if no events are tracked.
    #[tokio::test]
    async fn no_events_tracked_nothing_to_flush() {
        // Setup tracker (no events tracked).
        let mock_db = Arc::new(MockDB::new());
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let _ = EventTrackerDB::new(mock_db, &tracker, &cancellation_token);

        // Stop the tracker and wait for the workers to complete.
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }

    /// Test that search appearances are flushed correctly.
    #[tokio::test]
    async fn flush_search_appearances() {
        // Setup mock database.
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDB::new();
        mock_db
            .expect_update_search_appearances()
            .with(eq(vec![(*JOB1_ID, day.clone(), 1), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Ok(()));
        let mock_db = Arc::new(mock_db);

        // Setup tracker and track search appearances.
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let t = EventTrackerDB::new(mock_db, &tracker, &cancellation_token);
        t.track(Event::SearchAppearances {
            job_ids: vec![*JOB1_ID, *JOB2_ID],
        })
        .await
        .unwrap();

        // Stop the tracker and wait for the workers to complete.
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }

    /// Test that mixed events are flushed to their respective tables.
    #[tokio::test]
    async fn flush_mixed_events() {
        // Setup mock database.
        let day = OffsetDateTime::now_utc().format(&DATE_FORMAT).unwrap();
        let mut mock_db = MockDB::new();
        mock_db
            .expect_update_jobs_views()
            .with(eq(vec![(*JOB1_ID, day.clone(), 2)]))
            .times(1)
            .returning(|_| Ok(()));
        mock_db
            .expect_update_search_appearances()
            .with(eq(vec![(*JOB1_ID, day.clone(), 1), (*JOB2_ID, day, 1)]))
            .times(1)
            .returning(|_| Ok(()));
        let mock_db = Arc::new(mock_db);

        // Setup tracker and track mixed events.
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();
        let t = EventTrackerDB::new(mock_db, &tracker, &cancellation_token);
        t.track(Event::JobView { job_id: *JOB1_ID }).await.unwrap();
        t.track(Event::SearchAppearances {
            job_ids: vec![*JOB1_ID, *JOB2_ID],
        })
        .await
        .unwrap();
        t.track(Event::JobView { job_id: *JOB1_ID }).await.unwrap();

        // Stop the tracker and wait for the workers to complete.
        tracker.close();
        cancellation_token.cancel();
        tracker.wait().await;
    }
}
