use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Seafile HTTP API client
pub struct SeafileHttpClient {
    client: Client,
    server_url: String,
}

#[derive(Debug, Serialize)]
struct AuthRequest {
    username: String,
    password: String,
    platform: String,
    device_id: String,
    device_name: String,
    client_version: String,
    platform_version: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub encrypted: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RepoDownloadInfo {
    pub token: String,
    pub email: String,
    pub repo_name: String,
    pub encrypted: String,
    pub magic: String,
    pub enc_version: i32,
    pub random_key: String,
    pub repo_version: i32,
    pub salt: String,
    #[serde(default)]
    pub permission: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateRepoResponse {
    repo_id: String,
}

impl SeafileHttpClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            client: Client::new(),
            server_url: server_url.trim_end_matches('/').to_string(),
        }
    }

    /// Get authentication token
    pub fn get_token(
        &self,
        username: &str,
        password: &str,
        device_id: &str,
        tfa: Option<&str>,
    ) -> Result<String> {
        let hostname = hostname::get()?
            .to_string_lossy()
            .chars()
            .take(25)
            .collect::<String>();

        let auth_req = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
            platform: "linux".to_string(),
            device_id: device_id.to_string(),
            device_name: format!("terminal-{}", hostname),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            platform_version: String::new(),
        };

        let url = format!("{}/api2/auth-token/", self.server_url);
        let mut req = self.client.post(&url).form(&auth_req);

        if let Some(otp) = tfa {
            req = req.header("X-SEAFILE-OTP", otp);
        }

        let resp = req.send().context("Failed to send auth request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            anyhow::bail!("Authentication failed: {} - {}", status, text);
        }

        let auth_resp: AuthResponse = resp.json().context("Failed to parse auth response")?;
        Ok(auth_resp.token)
    }

    /// List remote repositories
    pub fn list_repos(&self, token: &str) -> Result<Vec<RepoInfo>> {
        let url = format!("{}/api2/repos/", self.server_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", token))
            .send()
            .context("Failed to list repos")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            anyhow::bail!("Failed to list repos: {} - {}", status, text);
        }

        let repos: Vec<RepoInfo> = resp.json().context("Failed to parse repo list")?;
        Ok(repos)
    }

    /// Get repository download information
    pub fn get_repo_download_info(&self, token: &str, repo_id: &str) -> Result<RepoDownloadInfo> {
        let url = format!("{}/api2/repos/{}/download-info/", self.server_url, repo_id);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", token))
            .send()
            .context("Failed to get download info")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            anyhow::bail!("Failed to get download info: {} - {}", status, text);
        }

        let info: RepoDownloadInfo = resp.json().context("Failed to parse download info")?;
        Ok(info)
    }

    /// Create a new repository
    pub fn create_repo(
        &self,
        token: &str,
        name: &str,
        desc: &str,
        password: Option<&str>,
    ) -> Result<String> {
        let url = format!("{}/api2/repos/", self.server_url);

        let mut data = HashMap::new();
        data.insert("name", name);
        data.insert("desc", desc);
        if let Some(pwd) = password {
            data.insert("passwd", pwd);
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", token))
            .form(&data)
            .send()
            .context("Failed to create repo")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            anyhow::bail!("Failed to create repo: {} - {}", status, text);
        }

        let resp: CreateRepoResponse = resp.json().context("Failed to parse create response")?;
        Ok(resp.repo_id)
    }

    /// Get base URL from server URL
    pub fn get_base_url(&self) -> &str {
        &self.server_url
    }
}
