mod form;
mod home;

use std::{path::{PathBuf, Path}, sync::Arc};

use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use minijinja::Environment;
use reqwest::Client;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tower_http::services::ServeDir;

use form::form;
use home::home;

#[derive(Clone)]
struct AppState {
    /// Connection pool to the SQLite database
    db_pool: SqlitePool,
    /// Jinja environment for templates
    template_env: Arc<Environment<'static>>,
    /// Reqwest client to resolve uploaded links
    client: Client,
    /// Path in which to store images
    images_dir: PathBuf,
}

// Default values for the environment variables
const IP: &str = "0.0.0.0";
const PORT: u16 = 80;
const DATA_DIR: &str = "data/";
const IMAGES_DIR: &str = "images/";
const DB_NAME: &str = "db.sqlite";

#[tokio::main]
async fn main() {
    let ip = std::env::var("BLOG_IP").unwrap_or_else(|_| IP.to_string());
    let port = std::env::var("BLOG_PORT").unwrap_or_else(|_| PORT.to_string());
    let data_dir = std::env::var("BLOG_DATA_DIR").unwrap_or_else(|_| DATA_DIR.to_string());
    let images_dir = std::env::var("BLOG_IMAGES_DIR").unwrap_or_else(|_| IMAGES_DIR.to_string());
    let db_name = std::env::var("BLOG_DB_NAME").unwrap_or_else(|_| DB_NAME.to_string());

    // TODO: Setup logging

    // Ensure the data directory exists
    tokio::fs::create_dir_all(&data_dir)
        .await
        .expect("failed to create data dir");

    // Setup the template environment
    let mut template_env = Environment::new();
    template_env.add_filter("dateformat", minijinja_contrib::filters::dateformat);
    template_env.add_template("home", include_str!("../templates/home.jinja"))
        .expect("embedded template is invalid");

    // Setup the database connection and migrations
    let db_conn_opts = SqliteConnectOptions::new()
        .filename(Path::new(&data_dir).join(db_name))
        .create_if_missing(true);
    let db_pool = SqlitePool::connect_with(db_conn_opts)
        .await
        .expect("failed to connect to the database");
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("failed to apply migrations to the database");

    let state = AppState {
        db_pool,
        template_env: Arc::new(template_env),
        client: Client::new(),
        images_dir: Path::new(&data_dir).join(images_dir),
    };

    // Ensure the images directory exists
    tokio::fs::create_dir_all(&state.images_dir)
        .await
        .expect("failed to create images dir");

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/home") }))
        .route("/home", get(home))
        .route("/form", post(form))
        .nest_service("/images", ServeDir::new(&state.images_dir))
        .with_state(state);

    let port = port.parse::<u16>().expect("failed to parse PORT");
    let listener = tokio::net::TcpListener::bind((&*ip, port)).await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
