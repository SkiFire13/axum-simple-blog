mod form;
mod home;

use std::sync::Arc;

use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use minijinja::Environment;
use reqwest::Client;

use form::form;
use home::home;

const IP: &str = "0.0.0.0";
const PORT: u16 = 3000;

#[derive(Clone)]
struct AppState {
    /// Jinja environment for templates
    env: Arc<Environment<'static>>,
    /// Reqwest client to resolve uploaded links
    client: Client,
}

#[tokio::main]
async fn main() {
    // TODO: Setup logging

    let mut env = Environment::new();
    env.add_template("home", include_str!("../templates/home.jinja"))
        .expect("Embedded template is invalid");

    let state = AppState {
        env: Arc::new(env),
        client: Client::new(),
    };

    // TODO: Setup routing for images
    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/home") }))
        .route("/home", get(home))
        .route("/form", post(form))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((IP, PORT)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
