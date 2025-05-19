//! This module contains background workers for some tasks.

use std::time::Duration;

use tokio::time::sleep;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error};

use crate::db::DynDB;

/// Launches all background workers.
pub(crate) fn run(db: DynDB, tracker: &TaskTracker, cancellation_token: CancellationToken) {
    // Jobs archiver
    tracker.spawn(async move {
        archiver(db, cancellation_token).await;
    });
}

/// Worker that archives expired jobs periodically.
pub(crate) async fn archiver(db: DynDB, cancellation_token: CancellationToken) {
    // Random sleep to avoid multiple workers running at the same time
    tokio::select! {
        () = sleep(Duration::from_secs(rand::random_range(60..300))) => {},
        () = cancellation_token.cancelled() => return,
    }

    loop {
        // Archive expired jobs
        debug!("archiving expired jobs");
        if let Err(err) = db.archive_expired_jobs().await {
            error!("error archiving expired jobs: {err}");
        }

        // Pause for a while before the next iteration
        tokio::select! {
            () = sleep(Duration::from_secs(60*60)) => {},
            () = cancellation_token.cancelled() => break,
        }
    }
}
