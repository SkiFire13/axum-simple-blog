use axum::{
    extract::State,
    response::{Html, Result},
};
use chrono::{DateTime, Utc};
use minijinja::context;
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::AppState;

#[derive(Serialize)]
struct Blogpost {
    text: String,
    date: DateTime<Utc>,
    image: Option<String>,
    user: String,
    avatar: Option<String>,
}

pub async fn home(State(state): State<AppState>) -> Result<Html<String>> {
    log::info!("Received /home request");

    let blogposts = load_blogposts(&state.db_pool)
        .await
        .inspect_err(|e| log::error!("failed to load blogposts: {e}"))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rendered = state
        .template_env
        .get_template("home")
        .inspect_err(|e| log::error!("failed to get template: {e}"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .render(context!(blogposts => blogposts))
        .inspect_err(|e| log::error!("failed to render template: {e}"))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(rendered))
}

async fn load_blogposts(db_pool: &SqlitePool) -> Result<Vec<Blogpost>, sqlx::Error> {
    sqlx::query_as!(
        Blogpost,
        r#"
            SELECT date AS "date: _", text, image, user, avatar
            FROM blogposts
            ORDER BY date DESC
        "#
    )
    .fetch_all(db_pool)
    .await
}
