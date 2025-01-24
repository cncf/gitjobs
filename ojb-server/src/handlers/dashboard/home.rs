//! This module defines the templates for the dashboard home page.

use std::collections::HashMap;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use rinja::Template;
use tracing::instrument;

use crate::{
    db::DynDB,
    handlers::{error::HandlerError, extractors::EmployerId},
    templates::dashboard::{
        employers,
        home::{self, Content, Tab},
        jobs,
    },
};

/// Handler that returns the dashboard home page.
#[instrument(skip_all, err)]
pub(crate) async fn page(
    State(db): State<DynDB>,
    Query(query): Query<HashMap<String, String>>,
    EmployerId(employer_id): EmployerId,
) -> Result<impl IntoResponse, HandlerError> {
    let tab: Tab = query.get("tab").into();
    let content = match tab {
        Tab::Jobs => {
            let jobs = db.list_employer_jobs(&employer_id).await?;
            Content::Jobs(jobs::ListPage { jobs })
        }
        Tab::Settings => {
            let employer_details = db.get_employer_details(&employer_id).await?;
            Content::Settings(employers::UpdatePage { employer_details })
        }
    };
    let template = home::Page { content };

    Ok(Html(template.render()?))
}
