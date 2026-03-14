use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Client voor de Semaphore REST API.
///
/// Alle endpoints gebruiken het `/deploy/api/` pad (door SEMAPHORE_WEB_ROOT).
#[derive(Clone)]
pub struct SemaphoreClient {
    client: Client,
    base_url: String,
    cookie: Option<String>,
    pub project_id: i64,
}

//  API-types 

#[derive(Debug, Serialize)]
struct LoginBody {
    auth: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct SemaphoreTask {
    pub id: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct CreateEnvironment {
    pub name: String,
    pub project_id: i64,
    pub password: String,
    pub json: String,
    pub env: String,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateInventory {
    pub name: String,
    pub project_id: i64,
    pub inventory: String,
    pub ssh_key_id: i64,
    pub become_key_id: i64,
    #[serde(rename = "type")]
    pub inv_type: String,
}

#[derive(Debug, Deserialize)]
pub struct InventoryResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyResponse {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub key_type: String,
}

#[derive(Debug, Deserialize)]
pub struct TemplateResponse {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RunTaskBody {
    pub template_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_approve: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destroy: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TaskOutput {
    pub output: String,
    pub time: String,
}

impl SemaphoreClient {
    pub fn new(base_url: &str, project_id: i64) -> Self {
        Self {
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
            cookie: None,
            project_id,
        }
    }

    fn api(&self, path: &str) -> String {
        format!("{}/deploy/api{}", self.base_url, path)
    }

    fn proj(&self, path: &str) -> String {
        format!(
            "{}/deploy/api/project/{}{}",
            self.base_url, self.project_id, path
        )
    }

    /// Inloggen – slaat cookie op voor volgende requests.
    pub async fn login(&mut self, user: &str, password: &str) -> Result<()> {
        let resp = self
            .client
            .post(self.api("/auth/login"))
            .json(&LoginBody {
                auth: user.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .context("Semaphore login request")?;

        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("; ");

        self.cookie = Some(cookie);
        Ok(())
    }

    fn authed(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref c) = self.cookie {
            req.header("Cookie", c)
        } else {
            req
        }
    }

    //  Keys 

    /// Zoek alle keys op in het project.
    pub async fn list_keys(&self) -> Result<Vec<KeyResponse>> {
        let resp = self.authed(self.client.get(self.proj("/keys"))).send().await?;
        Ok(resp.json().await?)
    }

    /// Vind een key op naam, geeft ID terug.
    pub async fn find_key_id(&self, name: &str) -> Result<Option<i64>> {
        let keys = self.list_keys().await?;
        Ok(keys.into_iter().find(|k| k.name == name).map(|k| k.id))
    }

    //  Environments 

    pub async fn create_environment(&self, env: CreateEnvironment) -> Result<EnvironmentResponse> {
        let resp = self
            .authed(self.client.post(self.proj("/environment")))
            .json(&env)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("create_environment {status}: {body}");
        }
        Ok(resp.json().await?)
    }

    pub async fn update_environment(&self, id: i64, env: CreateEnvironment) -> Result<()> {
        let resp = self
            .authed(
                self.client
                    .put(self.proj(&format!("/environment/{id}")))
            )
            .json(&env)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("update_environment {status}: {body}");
        }
        Ok(())
    }

    //  Inventories 

    pub async fn create_inventory(&self, inv: CreateInventory) -> Result<InventoryResponse> {
        let resp = self
            .authed(self.client.post(self.proj("/inventory")))
            .json(&inv)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("create_inventory {status}: {body}");
        }
        Ok(resp.json().await?)
    }

    pub async fn update_inventory(
        &self,
        id: i64,
        inv: CreateInventory,
    ) -> Result<()> {
        let resp = self
            .authed(
                self.client
                    .put(self.proj(&format!("/inventory/{id}")))
            )
            .json(&inv)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("update_inventory {status}: {body}");
        }
        Ok(())
    }

    //  Templates 

    pub async fn list_templates(&self) -> Result<Vec<TemplateResponse>> {
        let resp = self
            .authed(self.client.get(self.proj("/templates")))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// Vind een template op naam, geeft ID terug.
    pub async fn find_template_id(&self, name: &str) -> Result<Option<i64>> {
        let templates = self.list_templates().await?;
        Ok(templates.into_iter().find(|t| t.name == name).map(|t| t.id))
    }

    //  Tasks 

    /// Start een task.
    pub async fn run_task(&self, body: RunTaskBody) -> Result<SemaphoreTask> {
        let resp = self
            .authed(self.client.post(self.proj("/tasks")))
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("run_task {status}: {text}");
        }
        Ok(resp.json().await?)
    }

    /// Haal de status van een task op.
    pub async fn get_task(&self, task_id: i64) -> Result<Value> {
        let resp = self
            .authed(self.client.get(self.proj(&format!("/tasks/{task_id}"))))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// Haal task output op.
    pub async fn get_task_output(&self, task_id: i64) -> Result<Vec<TaskOutput>> {
        let resp = self
            .authed(
                self.client
                    .get(self.proj(&format!("/tasks/{task_id}/output"))),
            )
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
