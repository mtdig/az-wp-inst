use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::{AppState, html::HtmlTemplate};

const SESSION_USER_KEY: &str = "user_id";

//  Templates 

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    base: String,
    error: Option<String>,
}

//  Handlers 

pub async fn login_page(State(state): State<AppState>) -> impl IntoResponse {
    HtmlTemplate(LoginTemplate { base: state.base_path, error: None })
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login_submit(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Response {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE username = ?",
    )
    .bind(&form.username)
    .fetch_optional(&state.db)
    .await;

    let authenticated = match &user {
        Ok(Some(u)) => verify_password(&form.password, &u.password),
        _ => false,
    };

    if authenticated {
        let u = user.unwrap().unwrap();
        session.insert(SESSION_USER_KEY, &u.id).await.ok();
        Redirect::to(&format!("{}/", state.base_path)).into_response()
    } else {
        HtmlTemplate(LoginTemplate {
            base: state.base_path,
            error: Some("Ongeldige gebruikersnaam of wachtwoord.".into()),
        })
        .into_response()
    }
}

/// Verifieer een wachtwoord tegen een argon2 hash.
/// Valt terug op plaintext vergelijking voor backwards compatibiliteit.
fn verify_password(password: &str, stored: &str) -> bool {
    // Probeer eerst als argon2 hash
    if stored.starts_with("$argon2") {
        if let Ok(parsed_hash) = PasswordHash::new(stored) {
            return Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok();
        }
    }
    // Fallback: plaintext vergelijking (voor migratie)
    password == stored
}

pub async fn logout(State(state): State<AppState>, session: Session) -> Redirect {
    session.delete().await.ok();
    Redirect::to(&format!("{}/login", state.base_path))
}

/// Middleware-helper: haal user_id uit session, redirect naar login als niet ingelogd.
pub async fn require_auth(session: &Session) -> Option<String> {
    session.get::<String>(SESSION_USER_KEY).await.ok().flatten()
}
