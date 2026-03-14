use askama::Template;
use axum::{extract::State, response::IntoResponse};
use tower_sessions::Session;

use crate::{html::HtmlTemplate, models::Deployment, routes::auth::require_auth, AppState};

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    base: String,
    user_name: String,
    deployments: Vec<Deployment>,
}

pub async fn index(State(state): State<AppState>, session: Session) -> impl IntoResponse {
    let user_id = match require_auth(&session).await {
        Some(id) => id,
        None => return axum::response::Redirect::to(&format!("{}/login", state.base_path)).into_response(),
    };

    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&state.db)
        .await
        .expect("user moet bestaan");

    let deployments = sqlx::query_as::<_, Deployment>(
        "SELECT * FROM deployments WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_else(|e| {
        tracing::error!("Dashboard deployments query fout: {e}");
        vec![]
    });

    HtmlTemplate(DashboardTemplate {
        base: state.base_path,
        user_name: user.full_name,
        deployments,
    })
    .into_response()
}
