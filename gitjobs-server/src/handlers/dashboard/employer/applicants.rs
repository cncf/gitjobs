//! This module defines the HTTP handlers for the applicants page.

use anyhow::Result;
use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    db::{DynDB, dashboard::employer::ApplicantsSearchOutput},
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired},
    templates::dashboard::employer::applicants::{ApplicantsPage, Filters},
};

// Pages handlers.

/// Handler that returns the applicants list page.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    State(serde_qs_de): State<serde_qs::Config>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and applicants that match the query
    let filters = Filters::new(&serde_qs_de, &raw_query.unwrap_or_default())?;
    let (filters_options, ApplicantsSearchOutput { applicants, total: _ }) = tokio::try_join!(
        db.get_applicants_filters_options(&employer_id),
        db.search_applicants(&employer_id, &filters)
    )?;

    // Prepare template
    let template = ApplicantsPage {
        applicants,
        filters,
        filters_options,
    };

    Ok(Html(template.render()?))
}
