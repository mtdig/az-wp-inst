use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum DeploymentStatus {
    Draft,
    Provisioning,
    Provisioned,
    Configuring,
    Ready,
    Failed,
    Destroying,
    Destroyed,
}

impl std::fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "Concept"),
            Self::Provisioning => write!(f, "Provisioning…"),
            Self::Provisioned => write!(f, "Geprovisioned"),
            Self::Configuring => write!(f, "Configureren…"),
            Self::Ready => write!(f, "Gereed ✓"),
            Self::Failed => write!(f, "Mislukt ✗"),
            Self::Destroying => write!(f, "Vernietigen…"),
            Self::Destroyed => write!(f, "Vernietigd"),
        }
    }
}

impl DeploymentStatus {
    /// CSS class voor de status badge.
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Draft => "badge-ghost",
            Self::Provisioning | Self::Configuring | Self::Destroying => "badge-warning",
            Self::Provisioned => "badge-info",
            Self::Ready => "badge-success",
            Self::Failed => "badge-error",
            Self::Destroyed => "badge-neutral",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Deployment {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub status: DeploymentStatus,

    // Azure / Terraform inputs
    pub subscription_id: String,
    pub resource_group_name: String,
    pub public_ip_dns_label: String,
    pub mysql_server_name: String,
    pub mysql_admin_login: String,
    pub mysql_admin_password_ref: String,

    // Terraform outputs
    pub tf_public_ip: String,
    pub tf_public_fqdn: String,
    pub tf_mysql_fqdn: String,
    pub tf_admin_username: String,

    // Ansible refs
    pub ansible_become_password_ref: String,
    pub wp_admin_password_ref: String,
    pub db_wp_password_ref: String,

    // Semaphore refs
    pub sem_environment_id: Option<i32>,
    pub sem_inventory_id: Option<i32>,
    pub sem_tf_environment_id: Option<i32>,
    pub sem_last_task_id: Option<i32>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
