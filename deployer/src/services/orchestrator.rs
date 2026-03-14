use anyhow::{Context, Result};
use serde_json::json;
use sqlx::MySqlPool;

use crate::models::{Deployment, DeploymentStatus};
use crate::services::semaphore::{
    CreateEnvironment, CreateInventory, RunTaskBody, SemaphoreClient,
};

/// Orkestratielaag: koppelt de webapp aan Semaphore.
///
/// Workflow per deployment:
///   1. Maak/update Terraform environment in Semaphore met SP + tfvars
///   2. Start "Infrastructuur aanmaken (apply)" template → wacht tot klaar
///   3. Lees Terraform outputs (via Semaphore task output parsing)
///   4. Maak/update Ansible environment + inventory met outputs
///   5. Start "Volledige stack deployen" template → wacht tot klaar
pub struct Orchestrator {
    pub db: MySqlPool,
    pub sem: SemaphoreClient,
}

impl Orchestrator {
    pub fn new(db: MySqlPool, sem: SemaphoreClient) -> Self {
        Self { db, sem }
    }

    /// Update de status van een deployment in de database.
    async fn set_status(&self, id: &str, status: DeploymentStatus) -> Result<()> {
        let status_str = match status {
            DeploymentStatus::Draft => "draft",
            DeploymentStatus::Provisioning => "provisioning",
            DeploymentStatus::Provisioned => "provisioned",
            DeploymentStatus::Configuring => "configuring",
            DeploymentStatus::Ready => "ready",
            DeploymentStatus::Failed => "failed",
            DeploymentStatus::Destroying => "destroying",
            DeploymentStatus::Destroyed => "destroyed",
        };
        sqlx::query("UPDATE deployments SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Sla Semaphore environment IDs op bij de deployment.
    async fn save_sem_refs(
        &self,
        id: &str,
        tf_env_id: Option<i64>,
        env_id: Option<i64>,
        inv_id: Option<i64>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE deployments SET sem_tf_environment_id = ?, sem_environment_id = ?, sem_inventory_id = ? WHERE id = ?",
        )
        .bind(tf_env_id.map(|v| v as i32))
        .bind(env_id.map(|v| v as i32))
        .bind(inv_id.map(|v| v as i32))
        .bind(id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    /// Sla de laatste task ID op.
    async fn save_last_task(&self, id: &str, task_id: i64) -> Result<()> {
        sqlx::query("UPDATE deployments SET sem_last_task_id = ? WHERE id = ?")
            .bind(task_id as i32)
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Sla Terraform outputs op.
    async fn save_tf_outputs(
        &self,
        id: &str,
        ip: &str,
        fqdn: &str,
        mysql_fqdn: &str,
        admin_user: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE deployments SET tf_public_ip = ?, tf_public_fqdn = ?, tf_mysql_fqdn = ?, tf_admin_username = ? WHERE id = ?",
        )
        .bind(ip)
        .bind(fqdn)
        .bind(mysql_fqdn)
        .bind(admin_user)
        .bind(id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    // ── Terraform environment ─────────────────────────────────────

    /// Bouw de Terraform environment vars voor deze deployment.
    fn build_tf_env_vars(d: &Deployment) -> serde_json::Value {
        json!({
            "ARM_USE_MSI": "true",
            "ARM_SUBSCRIPTION_ID": d.subscription_id,
            "ARM_TENANT_ID": std::env::var("ARM_TENANT_ID").unwrap_or_default(),
            "TF_VAR_subscription_id": d.subscription_id,
            "TF_VAR_admin_public_key": d.admin_public_key,
            "TF_VAR_mysql_admin_password": d.mysql_admin_password_ref,
            "TF_VAR_resource_group_name": d.resource_group_name,
            "TF_VAR_public_ip_dns_label": d.public_ip_dns_label,
            "TF_VAR_mysql_server_name": d.mysql_server_name,
            "TF_VAR_mysql_admin_login": d.mysql_admin_login,
            "TF_IN_AUTOMATION": "1"
        })
    }

    /// Maak of update de Terraform environment in Semaphore.
    async fn ensure_tf_environment(&self, d: &Deployment) -> Result<i64> {
        let existing_id = d.sem_tf_environment_id.map(|v| v as i64);
        let env = CreateEnvironment {
            id: existing_id,
            name: format!("TF – {}", d.name),
            project_id: self.sem.project_id,
            password: String::new(),
            json: "{}".to_string(),
            env: Self::build_tf_env_vars(d).to_string(),
        };

        if let Some(existing_id) = existing_id {
            self.sem
                .update_environment(existing_id, env)
                .await?;
            Ok(existing_id)
        } else {
            let resp = self.sem.create_environment(env).await?;
            Ok(resp.id)
        }
    }

    // ── Ansible environment + inventory ───────────────────────────

    /// Bouw de Ansible environment JSON voor deze deployment.
    fn build_ansible_env(d: &Deployment) -> serde_json::Value {
        // Gebruik het IP-adres als ansible_host, maar val terug op de FQDN als
        // het IP onbekend is (bv. wanneer TF outputs niet geparsed konden worden).
        let ansible_host = if d.tf_public_ip.is_empty() {
            &d.tf_public_fqdn
        } else {
            &d.tf_public_ip
        };
        json!({
            "tf_public_fqdn": d.tf_public_fqdn,
            "tf_mysql_fqdn": d.tf_mysql_fqdn,
            "tf_mysql_admin_login": d.mysql_admin_login,
            "db_admin_password": d.mysql_admin_password_ref,
            "ansible_host": ansible_host,
            "db_wp_password": d.db_wp_password_ref,
            "wp_admin_password": d.wp_admin_password_ref,
            "ansible_become_password": d.ansible_become_password_ref,
            "wp_path": "/var/www/wordpress",
            "wp_db_name": "wordpress",
            "wp_db_user": "wpuser",
            "wp_db_port": 3306,
            "wp_db_ssl": true,
            "wp_admin_user": "osboxes",
            "wp_admin_email": format!("{}@student.hogent.be", d.name),
            "wp_title": format!("SELab Opdracht 4 – {}", d.name),
            "wp_locale": "nl_BE",
            "skip_common": false,
            "certbot_staging": true,
            "enable_vaultwarden": false,
            "vaultwarden_admin_token": "",
            "enable_tech_snake": false,
            "enable_semaphore": false,
            "enable_deployer": false,
            "semaphore_admin_user": "admin",
            "semaphore_admin_password": "",
            "ssh_host_alias": "azosboxes",
            "ssh_key": "~/.ssh/id_ed25519_hogent"
        })
    }

    /// Maak of update de Ansible environment in Semaphore.
    async fn ensure_ansible_environment(&self, d: &Deployment) -> Result<i64> {
        let existing_id = d.sem_environment_id.map(|v| v as i64);
        let env = CreateEnvironment {
            id: existing_id,
            name: format!("Ansible – {}", d.name),
            project_id: self.sem.project_id,
            password: String::new(),
            json: Self::build_ansible_env(d).to_string(),
            env: "{}".to_string(),
        };

        if let Some(existing_id) = existing_id {
            self.sem
                .update_environment(existing_id, env)
                .await?;
            Ok(existing_id)
        } else {
            let resp = self.sem.create_environment(env).await?;
            Ok(resp.id)
        }
    }

    /// Maak of update de Ansible inventory in Semaphore.
    async fn ensure_ansible_inventory(&self, d: &Deployment) -> Result<i64> {
        // Zoek SSH key ID op
        let ssh_key_id = self
            .sem
            .find_key_id("VM SSH sleutel")
            .await?
            .context("SSH key 'VM SSH sleutel' niet gevonden in Semaphore")?;
        let none_key_id = self
            .sem
            .find_key_id("Geen")
            .await?
            .context("Key 'Geen' niet gevonden in Semaphore")?;

        let existing_id = d.sem_inventory_id.map(|v| v as i64);
        let ansible_host = if d.tf_public_ip.is_empty() {
            &d.tf_public_fqdn
        } else {
            &d.tf_public_ip
        };
        let inventory_content = format!(
            "[webservers]\n{fqdn} ansible_host={host} ansible_become_password={become_pw}",
            fqdn = d.tf_public_fqdn,
            host = ansible_host,
            become_pw = d.ansible_become_password_ref,
        );

        let inv = CreateInventory {
            id: existing_id,
            name: format!("VM – {}", d.name),
            project_id: self.sem.project_id,
            inventory: inventory_content,
            ssh_key_id,
            become_key_id: none_key_id,
            inv_type: "static".to_string(),
        };

        if let Some(existing_id) = existing_id {
            self.sem
                .update_inventory(existing_id, inv)
                .await?;
            Ok(existing_id)
        } else {
            let resp = self.sem.create_inventory(inv).await?;
            Ok(resp.id)
        }
    }

    // ── Provisioning flow ─────────────────────────────────────────

    /// Start het volledige provisioning proces (Terraform apply).
    pub async fn start_provision(&self, deployment_id: &str) -> Result<()> {
        let d = self.get_deployment(deployment_id).await?;

        // 1. Status → provisioning
        self.set_status(&d.id, DeploymentStatus::Provisioning)
            .await?;

        // 2. Maak/update Terraform environment
        let tf_env_id = self.ensure_tf_environment(&d).await?;
        self.save_sem_refs(&d.id, Some(tf_env_id), d.sem_environment_id.map(|v| v as i64), d.sem_inventory_id.map(|v| v as i64))
            .await?;

        // 3. Zoek de "apply" template
        let template_id = self
            .sem
            .find_template_id("Infrastructuur aanmaken (apply)")
            .await?
            .context("Terraform apply template niet gevonden")?;

        // 4. Wijs de template naar onze per-deployment TF environment
        self.sem
            .update_template_environment(template_id, tf_env_id)
            .await?;

        // 5. Start de task met auto_approve
        let task = self
            .sem
            .run_task(RunTaskBody {
                template_id,
                environment: None,
                cli_args: None,
                plan: None,
                auto_approve: Some(true),
                destroy: None,
            })
            .await?;

        self.save_last_task(&d.id, task.id).await?;

        // 5. Poll in achtergrond (spawned task)
        let orch = Self::new(self.db.clone(), self.sem.clone());
        let deploy_id = d.id.clone();
        tokio::spawn(async move {
            if let Err(e) = orch.poll_provision_completion(&deploy_id, task.id).await {
                tracing::error!("Provisioning poll fout voor {deploy_id}: {e}");
                orch.set_status(&deploy_id, DeploymentStatus::Failed)
                    .await
                    .ok();
            }
        });

        Ok(())
    }

    /// Poll tot de Terraform task klaar is, parse dan outputs.
    async fn poll_provision_completion(&self, deploy_id: &str, task_id: i64) -> Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let task = self.sem.get_task(task_id).await?;
            let status = task["status"].as_str().unwrap_or("");

            match status {
                "success" => break,
                "error" | "stopped" => {
                    self.set_status(deploy_id, DeploymentStatus::Failed).await?;
                    return Ok(());
                }
                _ => continue, // "waiting", "running", etc.
            }
        }

        // Parse Terraform outputs uit de task output
        let outputs = self.sem.get_task_output(task_id).await?;
        let full_output: String = outputs.iter().map(|o| o.output.as_str()).collect();

        // Terraform outputs worden gelogd als "public_ip_address = ..." etc.
        let ip = Self::extract_tf_output(&full_output, "public_ip_address");
        let fqdn = Self::extract_tf_output(&full_output, "public_fqdn");
        let mysql_fqdn = Self::extract_tf_output(&full_output, "mysql_fqdn");
        let admin_user = Self::extract_tf_output(&full_output, "admin_username")
            .unwrap_or_else(|| "osboxes".to_string());

        // Fallback: bereken deterministische outputs uit formuliervelden
        // (bv. bij "No changes" herhaalde apply worden outputs niet geprint)
        let (final_ip, final_fqdn, final_mysql) = if fqdn.is_some() && mysql_fqdn.is_some() {
            (
                ip.unwrap_or_default(),
                fqdn.unwrap(),
                mysql_fqdn.unwrap(),
            )
        } else {
            let d = self.get_deployment(deploy_id).await?;
            let fb_fqdn = format!(
                "{}.francecentral.cloudapp.azure.com",
                d.public_ip_dns_label
            );
            let fb_mysql = format!(
                "{}.mysql.database.azure.com",
                d.mysql_server_name
            );
            tracing::warn!(
                "Kon Terraform outputs niet parsen voor {deploy_id}, gebruik fallback: fqdn={fb_fqdn}, mysql={fb_mysql}"
            );
            (
                ip.unwrap_or_default(),
                fqdn.unwrap_or(fb_fqdn),
                mysql_fqdn.unwrap_or(fb_mysql),
            )
        };

        self.save_tf_outputs(deploy_id, &final_ip, &final_fqdn, &final_mysql, &admin_user)
            .await?;
        self.set_status(deploy_id, DeploymentStatus::Provisioned)
            .await?;

        Ok(())
    }

    /// Probeer een Terraform output waarde te parsen uit task output.
    fn extract_tf_output(output: &str, key: &str) -> Option<String> {
        // Terraform output format: `key = "value"` of `key = value`
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with(key) {
                if let Some((_k, v)) = trimmed.split_once('=') {
                    let val = v.trim().trim_matches('"');
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
        None
    }

    // ── Configure flow ────────────────────────────────────────────

    /// Start Ansible configuratie (na provisioning).
    pub async fn start_configure(&self, deployment_id: &str) -> Result<()> {
        // Herlees deployment met TF outputs
        let d = self.get_deployment(deployment_id).await?;

        self.set_status(&d.id, DeploymentStatus::Configuring).await?;

        // 1. Maak/update Ansible environment
        let env_id = self.ensure_ansible_environment(&d).await?;

        // 2. Maak/update Ansible inventory
        let inv_id = self.ensure_ansible_inventory(&d).await?;

        // 3. Sla refs op
        self.save_sem_refs(
            &d.id,
            d.sem_tf_environment_id.map(|v| v as i64),
            Some(env_id),
            Some(inv_id),
        )
        .await?;

        // 4. Zoek de "deploy" template
        let template_id = self
            .sem
            .find_template_id("Volledige stack deployen")
            .await?
            .context("Ansible deploy template niet gevonden")?;

        // 5. Wijs de template naar onze per-deployment Ansible environment + inventory
        self.sem
            .update_template_env_and_inventory(template_id, env_id, inv_id)
            .await?;

        // 6. Start task
        let task = self
            .sem
            .run_task(RunTaskBody {
                template_id,
                environment: None,
                cli_args: None,
                plan: None,
                auto_approve: None,
                destroy: None,
            })
            .await?;

        self.save_last_task(&d.id, task.id).await?;

        // 6. Poll in achtergrond
        let orch = Self::new(self.db.clone(), self.sem.clone());
        let deploy_id = d.id.clone();
        tokio::spawn(async move {
            if let Err(e) = orch.poll_configure_completion(&deploy_id, task.id).await {
                tracing::error!("Configure poll fout voor {deploy_id}: {e}");
                orch.set_status(&deploy_id, DeploymentStatus::Failed)
                    .await
                    .ok();
            }
        });

        Ok(())
    }

    /// Poll tot Ansible klaar is.
    async fn poll_configure_completion(&self, deploy_id: &str, task_id: i64) -> Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let task = self.sem.get_task(task_id).await?;
            let status = task["status"].as_str().unwrap_or("");

            match status {
                "success" => {
                    self.set_status(deploy_id, DeploymentStatus::Ready).await?;
                    return Ok(());
                }
                "error" | "stopped" => {
                    self.set_status(deploy_id, DeploymentStatus::Failed).await?;
                    return Ok(());
                }
                _ => continue,
            }
        }
    }

    // ── Destroy flow ──────────────────────────────────────────────

    /// Start Terraform destroy.
    pub async fn start_destroy(&self, deployment_id: &str) -> Result<()> {
        let d = self.get_deployment(deployment_id).await?;

        self.set_status(&d.id, DeploymentStatus::Destroying).await?;

        // Zorg dat TF environment up-to-date is
        let tf_env_id = self.ensure_tf_environment(&d).await?;
        self.save_sem_refs(
            &d.id,
            Some(tf_env_id),
            d.sem_environment_id.map(|v| v as i64),
            d.sem_inventory_id.map(|v| v as i64),
        )
        .await?;

        let template_id = self
            .sem
            .find_template_id("Infrastructuur vernietigen (destroy)")
            .await?
            .context("Terraform destroy template niet gevonden")?;

        // Wijs de template naar onze per-deployment TF environment
        self.sem
            .update_template_environment(template_id, tf_env_id)
            .await?;

        let task = self
            .sem
            .run_task(RunTaskBody {
                template_id,
                environment: None,
                cli_args: None,
                plan: None,
                auto_approve: Some(true),
                destroy: Some(true),
            })
            .await?;

        self.save_last_task(&d.id, task.id).await?;

        let orch = Self::new(self.db.clone(), self.sem.clone());
        let deploy_id = d.id.clone();
        tokio::spawn(async move {
            if let Err(e) = orch.poll_destroy_completion(&deploy_id, task.id).await {
                tracing::error!("Destroy poll fout voor {deploy_id}: {e}");
                orch.set_status(&deploy_id, DeploymentStatus::Failed)
                    .await
                    .ok();
            }
        });

        Ok(())
    }

    /// Poll tot destroy klaar is.
    async fn poll_destroy_completion(&self, deploy_id: &str, task_id: i64) -> Result<()> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let task = self.sem.get_task(task_id).await?;
            let status = task["status"].as_str().unwrap_or("");

            match status {
                "success" => {
                    // Wis TF outputs
                    self.save_tf_outputs(deploy_id, "", "", "", "").await?;
                    self.set_status(deploy_id, DeploymentStatus::Destroyed)
                        .await?;
                    return Ok(());
                }
                "error" | "stopped" => {
                    self.set_status(deploy_id, DeploymentStatus::Failed).await?;
                    return Ok(());
                }
                _ => continue,
            }
        }
    }

    // ── Helpers ───────────────────────────────────────────────────

    async fn get_deployment(&self, id: &str) -> Result<Deployment> {
        sqlx::query_as::<_, Deployment>("SELECT * FROM deployments WHERE id = ?")
            .bind(id)
            .fetch_one(&self.db)
            .await
            .context("Deployment niet gevonden")
    }

    /// Haal de task output op voor weergave.
    pub async fn get_task_output_text(&self, task_id: i64) -> Result<String> {
        let outputs = self.sem.get_task_output(task_id).await?;
        Ok(outputs
            .iter()
            .map(|o| o.output.as_str())
            .collect::<Vec<_>>()
            .join(""))
    }
}
