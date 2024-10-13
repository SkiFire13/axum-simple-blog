use axum::{extract::State, response::Html};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct Post {
    text: String,
    data: String,
    image: Option<String>,
    user: String,
    user_image: Option<String>,
}

pub async fn home(State(state): State<AppState>) -> Html<String> {
    // TODO: Actually get the posts
    let posts = Vec::<Post>::new();

    // TODO: Handle errors
    let rendered = state
        .env
        .get_template("home")
        .unwrap()
        .render(posts)
        .unwrap();

    Html(rendered)
}
