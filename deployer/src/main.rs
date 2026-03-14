mod html;
mod models;
mod routes;
mod services;

use axum::{Router, routing::get, routing::post};
use sqlx::mysql::MySqlPoolOptions;
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
    /// Base path waaronder de app draait, bv. "/app" (zonder trailing slash).
    /// Leeg als de app op "/" draait.
    pub base_path: String,
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
    // Base path voor reverse proxy, bv. "/app" (zonder trailing slash)
    let base_path = std::env::var("BASE_PATH")
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string();

    // Database
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("Database verbonden");

    // Migraties uitvoeren
    let migrations: &[(&str, &str)] = &[
        ("001_initial", include_str!("../migrations/001_initial.sql")),
        ("002_drop_sp_columns", include_str!("../migrations/002_drop_sp_columns.sql")),
        ("003_add_admin_public_key", include_str!("../migrations/003_add_admin_public_key.sql")),
    ];
    for (name, sql) in migrations {
        // Strip lijn-commentaar (--) eerst, dan splitsen op ;
        let cleaned: String = sql
            .lines()
            .filter(|line| !line.trim_start().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");
        for statement in cleaned
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            if let Err(e) = sqlx::query(statement).execute(&pool).await {
                // Duplicate column / table already exists → verwacht bij herstart
                let msg = e.to_string();
                if msg.contains("Duplicate") || msg.contains("already exists") {
                    tracing::debug!("Migratie {name}: overgeslagen (al toegepast)");
                } else {
                    tracing::warn!("Migratie {name} fout: {e}");
                }
            }
        }
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
        base_path: base_path.clone(),
    };

    // Routes
    let inner = Router::new()
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
        .with_state(state.clone());

    // Nest onder BASE_PATH als die gezet is
    let app = if base_path.is_empty() {
        inner.layer(session_layer)
    } else {
        Router::new()
            .nest(&base_path, inner)
            // nest() matcht /app maar niet /app/ — voeg expliciete route toe
            // zodat Apache's ProxyPass /app/ ook werkt (geen redirect loop)
            .route(
                &format!("{}/", base_path),
                get(routes::dashboard::index).with_state(state),
            )
            .layer(session_layer)
    };

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("Deployer luistert op http://{listen_addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
