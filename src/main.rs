mod form;
mod home;

use std::{path::PathBuf, sync::Arc};

use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use minijinja::Environment;
use reqwest::Client;
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use form::form;
use home::home;

const IP: &str = "0.0.0.0";
const PORT: u16 = 3000;
const IMAGES_DIR: &str = "data/images/";
const DB_PATH: &str = "data/db.sqlite";

#[derive(Clone)]
struct AppState {
    /// Connection pool to the SQLite database
    db_pool: SqlitePool,
    /// Jinja environment for templates
    env: Arc<Environment<'static>>,
    /// Reqwest client to resolve uploaded links
    client: Client,
    /// Path in which to store images
    image_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    // TODO: Setup logging

    let mut env = Environment::new();
    env.add_template("home", include_str!("../templates/home.jinja"))
        .expect("Embedded template is invalid");

    let db_pool = SqlitePool::connect(DB_PATH)
        .await
        .expect("Couldn't connect to the database");

    let state = AppState {
        db_pool,
        env: Arc::new(env),
        client: Client::new(),
        image_dir: PathBuf::from(IMAGES_DIR),
    };

    if !tokio::fs::try_exists(&state.image_dir)
        .await
        .expect("Couldn't determine if the image data dir exists")
    {
        tokio::fs::create_dir_all(&state.image_dir)
            .await
            .expect("Couldn't create image data dir");
    }

    // TODO: Setup routing for images
    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/home") }))
        .route("/home", get(home))
        .route("/form", post(form))
        .nest_service("/images", ServeDir::new(&state.image_dir))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((IP, PORT)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
