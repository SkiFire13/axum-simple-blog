mod form;
mod home;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    response::Redirect,
    routing::{get, post},
    Router,
};
use minijinja::Environment;
use reqwest::Client;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
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
const IMAGES_DIR_PATH: &str = "data/images/";
const DB_PATH: &str = "data/db.sqlite";

#[tokio::main]
async fn main() {
    env_logger::init();

    let ip = std::env::var("BLOG_IP").unwrap_or_else(|_| IP.to_string());
    let port = std::env::var("BLOG_PORT").unwrap_or_else(|_| PORT.to_string());
    let images_dir_path =
        std::env::var("BLOG_IMAGES_DIR").unwrap_or_else(|_| IMAGES_DIR_PATH.to_string());
    let db_path = std::env::var("BLOG_DB_NAME").unwrap_or_else(|_| DB_PATH.to_string());

    log::info!("ip = {ip}");
    log::info!("port = {port}");
    log::info!("images_dir_path = {images_dir_path}");
    log::info!("db_path = {db_path}");

    let template_env = Arc::new(setup_template_env());
    let db_pool = setup_sqlite_database(Path::new(&db_path)).await;
    setup_images_directory(Path::new(&images_dir_path)).await;

    let state = AppState {
        db_pool,
        template_env,
        client: Client::new(),
        images_dir: PathBuf::from(images_dir_path),
    };

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/home") }))
        .route("/home", get(home))
        .route("/form", post(form))
        .nest_service("/images", ServeDir::new(&state.images_dir))
        .with_state(state);

    log::info!("Starting network listener");
    let port = port.parse::<u16>().expect("failed to parse PORT");
    let listener = tokio::net::TcpListener::bind((&*ip, port))
        .await
        .expect("failed to bind network listener");

    log::info!("Starting service");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to serve");
    log::info!("Stopped");
}

fn setup_template_env() -> Environment<'static> {
    log::info!("Initializing template environment");
    let mut template_env = Environment::new();
    template_env.add_filter("dateformat", minijinja_contrib::filters::dateformat);
    template_env
        .add_template("home", include_str!("../templates/home.jinja"))
        .expect("embedded template is invalid");

    template_env
}

async fn setup_sqlite_database(db_path: &Path) -> SqlitePool {
    log::info!("Initializing SQLite database");
    if let Some(db_dir) = db_path.parent() {
        tokio::fs::create_dir_all(db_dir)
            .await
            .expect("failed to create db dir");
    }
    let db_conn_opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);
    let db_pool = SqlitePool::connect_with(db_conn_opts)
        .await
        .expect("failed to connect to the database");
    log::info!("Migrating SQLite database");
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("failed to apply migrations to the database");
    db_pool
}

async fn setup_images_directory(images_dir_path: &Path) {
    // Ensure the images directory exists
    log::info!("Initializing images directory");
    tokio::fs::create_dir_all(&images_dir_path)
        .await
        .expect("failed to create images dir");
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
