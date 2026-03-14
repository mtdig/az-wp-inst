use askama::Template;
use axum::{
    Form,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;
use uuid::Uuid;

use crate::{
    html::HtmlTemplate,
    models::{Deployment, DeploymentStatus},
    routes::auth::require_auth,
    services::orchestrator::Orchestrator,
    services::semaphore::SemaphoreClient,
    AppState,
};

//  Templates 

#[derive(Template)]
#[template(path = "deploy_new.html")]
struct NewDeployTemplate {
    base: String,
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "deploy_detail.html")]
struct DeployDetailTemplate {
    base: String,
    deployment: Deployment,
}

#[derive(Template)]
#[template(path = "partials/status_badge.html")]
struct StatusBadgeTemplate {
    base: String,
    deployment: Deployment,
}

//  Forms 

#[derive(serde::Deserialize)]
pub struct NewDeployForm {
    name: String,
    subscription_id: String,
    resource_group_name: String,
    public_ip_dns_label: String,
    mysql_server_name: String,
    mysql_admin_login: String,
    mysql_admin_password: String,
    ansible_become_password: String,
    wp_admin_password: String,
    db_wp_password: String,
}

//  Helpers 

/// Maak een ingelogde SemaphoreClient aan.
async fn sem_client(state: &AppState) -> Result<SemaphoreClient, StatusCode> {
    let mut sem = SemaphoreClient::new(&state.semaphore_url, state.semaphore_project_id);
    sem.login(&state.semaphore_user, &state.semaphore_password)
        .await
        .map_err(|e| {
            tracing::error!("Semaphore login fout: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(sem)
}

/// Haal de deployment op, met auth check.
async fn get_deployment_for_user(
    state: &AppState,
    session: &Session,
    deploy_id: &str,
) -> Result<Deployment, Response> {
    let user_id = require_auth(session)
        .await
        .ok_or_else(|| Redirect::to(&format!("{}/login", state.base_path)).into_response())?;

    sqlx::query_as::<_, Deployment>(
        "SELECT * FROM deployments WHERE id = ? AND user_id = ?",
    )
    .bind(deploy_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("DB query fout voor deployment {deploy_id}: {e}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })?
    .ok_or_else(|| StatusCode::NOT_FOUND.into_response())
}

//  Handlers 

pub async fn new_page(State(state): State<AppState>, session: Session) -> Response {
    match require_auth(&session).await {
        Some(_) => HtmlTemplate(NewDeployTemplate { error: None, base: state.base_path }).into_response(),
        None => Redirect::to(&format!("{}/login", state.base_path)).into_response(),
    }
}

pub async fn create(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<NewDeployForm>,
) -> Response {
    let user_id = match require_auth(&session).await {
        Some(id) => id,
        None => return Redirect::to(&format!("{}/login", state.base_path)).into_response(),
    };

    let id = Uuid::new_v4().to_string();

    // TODO: secrets opslaan in Vaultwarden en alleen refs bewaren
    let result = sqlx::query(
        r#"INSERT INTO deployments
           (id, user_id, name, status,
            subscription_id, resource_group_name, public_ip_dns_label,
            mysql_server_name, mysql_admin_login, mysql_admin_password_ref,
            ansible_become_password_ref, wp_admin_password_ref, db_wp_password_ref)
           VALUES (?, ?, ?, 'draft', ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&form.name)
    .bind(&form.subscription_id)
    .bind(&form.resource_group_name)
    .bind(&form.public_ip_dns_label)
    .bind(&form.mysql_server_name)
    .bind(&form.mysql_admin_login)
    .bind(&form.mysql_admin_password)
    .bind(&form.ansible_become_password)
    .bind(&form.wp_admin_password)
    .bind(&form.db_wp_password)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Redirect::to(&format!("{}/deploy/{id}", state.base_path)).into_response(),
        Err(e) => HtmlTemplate(NewDeployTemplate {
            error: Some(format!("Fout bij opslaan: {e}")),
            base: state.base_path,
        })
        .into_response(),
    }
}

pub async fn detail(
    State(state): State<AppState>,
    session: Session,
    Path(deploy_id): Path<String>,
) -> Response {
    match get_deployment_for_user(&state, &session, &deploy_id).await {
        Ok(d) => HtmlTemplate(DeployDetailTemplate { base: state.base_path, deployment: d }).into_response(),
        Err(r) => r,
    }
}

/// htmx endpoint: start Terraform apply via Semaphore.
pub async fn start_provision(
    State(state): State<AppState>,
    session: Session,
    Path(deploy_id): Path<String>,
) -> Response {
    let d = match get_deployment_for_user(&state, &session, &deploy_id).await {
        Ok(d) => d,
        Err(r) => return r,
    };

    let sem = match sem_client(&state).await {
        Ok(s) => s,
        Err(sc) => return sc.into_response(),
    };

    let orch = Orchestrator::new(state.db.clone(), sem);
    if let Err(e) = orch.start_provision(&d.id).await {
        tracing::error!("Provisioning fout: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Herlees de deployment na statuswijziging
    let d = sqlx::query_as::<_, Deployment>("SELECT * FROM deployments WHERE id = ?")
        .bind(&deploy_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(d);

    HtmlTemplate(StatusBadgeTemplate { base: state.base_path, deployment: d }).into_response()
}

/// htmx endpoint: start Ansible configuratie via Semaphore.
pub async fn start_configure(
    State(state): State<AppState>,
    session: Session,
    Path(deploy_id): Path<String>,
) -> Response {
    let d = match get_deployment_for_user(&state, &session, &deploy_id).await {
        Ok(d) => d,
        Err(r) => return r,
    };

    let sem = match sem_client(&state).await {
        Ok(s) => s,
        Err(sc) => return sc.into_response(),
    };

    let orch = Orchestrator::new(state.db.clone(), sem);
    if let Err(e) = orch.start_configure(&d.id).await {
        tracing::error!("Configure fout: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let d = sqlx::query_as::<_, Deployment>("SELECT * FROM deployments WHERE id = ?")
        .bind(&deploy_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(d);

    HtmlTemplate(StatusBadgeTemplate { base: state.base_path, deployment: d }).into_response()
}

/// htmx endpoint: start Terraform destroy via Semaphore.
pub async fn start_destroy(
    State(state): State<AppState>,
    session: Session,
    Path(deploy_id): Path<String>,
) -> Response {
    let d = match get_deployment_for_user(&state, &session, &deploy_id).await {
        Ok(d) => d,
        Err(r) => return r,
    };

    let sem = match sem_client(&state).await {
        Ok(s) => s,
        Err(sc) => return sc.into_response(),
    };

    let orch = Orchestrator::new(state.db.clone(), sem);
    if let Err(e) = orch.start_destroy(&d.id).await {
        tracing::error!("Destroy fout: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let d = sqlx::query_as::<_, Deployment>("SELECT * FROM deployments WHERE id = ?")
        .bind(&deploy_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(d);

    HtmlTemplate(StatusBadgeTemplate { base: state.base_path, deployment: d }).into_response()
}

/// htmx polling endpoint: haal huidige status op.
pub async fn poll_status(
    State(state): State<AppState>,
    session: Session,
    Path(deploy_id): Path<String>,
) -> Response {
    let user_id = match require_auth(&session).await {
        Some(id) => id,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let deployment = sqlx::query_as::<_, Deployment>(
        "SELECT * FROM deployments WHERE id = ? AND user_id = ?",
    )
    .bind(&deploy_id)
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await;

    match deployment {
        Ok(Some(d)) => HtmlTemplate(StatusBadgeTemplate { base: state.base_path, deployment: d }).into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// htmx endpoint: haal de task output op als plain text.
pub async fn get_output(
    State(state): State<AppState>,
    session: Session,
    Path(deploy_id): Path<String>,
) -> Response {
    let d = match get_deployment_for_user(&state, &session, &deploy_id).await {
        Ok(d) => d,
        Err(r) => return r,
    };

    let task_id = match d.sem_last_task_id {
        Some(id) => id as i64,
        None => return (StatusCode::OK, "Geen taak gestart.").into_response(),
    };

    let sem = match sem_client(&state).await {
        Ok(s) => s,
        Err(sc) => return sc.into_response(),
    };

    let orch = Orchestrator::new(state.db.clone(), sem);
    match orch.get_task_output_text(task_id).await {
        Ok(text) => (StatusCode::OK, text).into_response(),
        Err(e) => {
            tracing::error!("Task output fout: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Kon output niet laden.").into_response()
        }
    }
}
