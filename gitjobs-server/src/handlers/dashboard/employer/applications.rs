//! This module defines the HTTP handlers for the applications page.

use anyhow::Result;
use axum::{
    extract::{RawQuery, State},
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    db::{DynDB, dashboard::employer::ApplicationsSearchOutput},
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired},
    templates::{
        dashboard::employer::applications::{ApplicationsPage, Filters},
        pagination::NavigationLinks,
    },
};

// Pages handlers.

/// Handler that returns the applications list page.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    State(serde_qs_de): State<serde_qs::Config>,
    RawQuery(raw_query): RawQuery,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and applications that match the query
    let filters = Filters::new(&serde_qs_de, &raw_query.unwrap_or_default())?;
    let (filters_options, ApplicationsSearchOutput { applications, total }) = tokio::try_join!(
        db.get_applications_filters_options(&employer_id),
        db.search_applications(&employer_id, &filters)
    )?;

    // Prepare template
    let navigation_links = NavigationLinks::from_filters(&filters, total)?;
    let template = ApplicationsPage {
        applications,
        filters,
        filters_options,
        navigation_links,
    };

    Ok(Html(template.render()?))
}
