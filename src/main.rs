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

#[tokio::main]
async fn main() {
    let ip = "0.0.0.0";
    let port = 80;
    let images_dir_path = Path::new("data/images/");
    let db_path = Path::new("data/db.sqlite");

    env_logger::init();

    let template_env = Arc::new(setup_template_env());
    let db_pool = setup_sqlite_database(db_path).await;
    setup_images_directory(images_dir_path).await;

    let state = AppState {
        db_pool,
        template_env,
        client: Client::new(),
        images_dir: images_dir_path.to_owned(),
    };

    let app = Router::new()
        .route("/", get(|| async { Redirect::permanent("/home") }))
        .route("/home", get(home))
        .route("/home", post(form))
        .nest_service("/images", ServeDir::new(&state.images_dir))
        .with_state(state);

    log::info!("Starting network listener");
    let listener = tokio::net::TcpListener::bind((ip, port))
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
