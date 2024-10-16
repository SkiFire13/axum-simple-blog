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
struct Post {
    text: String,
    date: DateTime<Utc>,
    image: Option<String>,
    user: String,
    avatar: Option<String>,
}

pub async fn home(State(state): State<AppState>) -> Result<Html<String>> {
    let blogposts = load_blogposts(&state.db_pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let rendered = state
        .env
        .get_template("home")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .render(context!(blogposts => blogposts))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Html(rendered))
}

async fn load_blogposts(db_pool: &SqlitePool) -> Result<Vec<Post>, sqlx::Error> {
    sqlx::query_as!(
        Post,
        r#"
            SELECT date AS "date: _", text, image, user, avatar
            FROM blogposts
            ORDER BY date DESC
        "#
    )
    .fetch_all(db_pool)
    .await
}
