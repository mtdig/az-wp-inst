mod html;
mod models;
mod routes;
mod services;

use axum::{Router, routing::get, routing::post};
use sqlx::mysql::MySqlPoolOptions;
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions::MemoryStore;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::MySqlPool,
    pub semaphore_url: String,
    pub semaphore_user: String,
    pub semaphore_password: String,
    pub semaphore_project_id: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deployer=debug,tower_http=info".into()),
        )
        .init();

    // Env vars
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://deployer:deployer@127.0.0.1/deployer".into());
    let listen_addr =
        std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3001".into());
    let semaphore_url =
        std::env::var("SEMAPHORE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let semaphore_user =
        std::env::var("SEMAPHORE_USER").unwrap_or_else(|_| "admin".into());
    let semaphore_password =
        std::env::var("SEMAPHORE_PASSWORD").expect("SEMAPHORE_PASSWORD is vereist");
    let semaphore_project_id: i64 = std::env::var("SEMAPHORE_PROJECT_ID")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .expect("SEMAPHORE_PROJECT_ID moet een getal zijn");

    // Database
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Database verbonden");

    // Migraties uitvoeren
    for statement in include_str!("../migrations/001_initial.sql")
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
    {
        sqlx::query(statement).execute(&pool).await.ok();
    }
    tracing::info!("Migraties uitgevoerd");

    // Sessie store (in-memory, voldoende voor enkele VM)
    let session_store = MemoryStore::default();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // true bij HTTPS
        .with_http_only(true);

    let state = AppState {
        db: pool,
        semaphore_url,
        semaphore_user,
        semaphore_password,
        semaphore_project_id,
    };

    // Routes
    let app = Router::new()
        // Auth
        .route(
            "/login",
            get(routes::auth::login_page).post(routes::auth::login_submit),
        )
        .route("/logout", get(routes::auth::logout))
        // Dashboard
        .route("/", get(routes::dashboard::index))
        // Deploy
        .route("/deploy/new", get(routes::deploy::new_page))
        .route("/deploy", post(routes::deploy::create))
        .route("/deploy/{id}", get(routes::deploy::detail))
        .route(
            "/deploy/{id}/provision",
            post(routes::deploy::start_provision),
        )
        .route(
            "/deploy/{id}/configure",
            post(routes::deploy::start_configure),
        )
        .route(
            "/deploy/{id}/destroy",
            post(routes::deploy::start_destroy),
        )
        .route("/deploy/{id}/status", get(routes::deploy::poll_status))
        .route("/deploy/{id}/output", get(routes::deploy::get_output))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        // Middleware
        .layer(session_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("Deployer luistert op http://{listen_addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
