use axum::{extract::State, response::Html};
use minijinja::context;
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct Post {
    text: String,
    date: String,
    image: Option<String>,
    user: String,
    avatar: Option<String>,
}

pub async fn home(State(state): State<AppState>) -> Html<String> {
    // TODO: Actually get the posts
    let posts = Vec::<Post>::new();

    // TODO: Handle errors
    let rendered = state
        .env
        .get_template("home")
        .unwrap()
        .render(context!(posts => posts))
        .unwrap();

    Html(rendered)
}
