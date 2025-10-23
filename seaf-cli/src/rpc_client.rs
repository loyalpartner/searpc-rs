use searpc::Result;
use searpc_macro::rpc;
use serde::{Deserialize, Serialize};

/// Seafile repo information
#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    pub id: String,
    pub name: String,
    pub worktree: String,
    #[serde(default)]
    pub auto_sync: bool,
}

/// Clone task information
#[derive(Debug, Serialize, Deserialize)]
pub struct CloneTask {
    #[serde(default)]
    pub repo_id: String,
    #[serde(default)]
    pub repo_name: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub error: i32,
}

/// Sync task information
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncTask {
    #[serde(default)]
    pub repo_id: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub error: i32,
}

/// Transfer task information
#[derive(Debug, Serialize, Deserialize)]
pub struct TransferTask {
    pub repo_id: String,
    pub block_done: i64,
    pub block_total: i64,
    pub rate: i64,
    #[serde(default)]
    pub rt_state: String,
    #[serde(default)]
    pub fs_objects_done: i64,
    #[serde(default)]
    pub fs_objects_total: i64,
}

/// Seafile RPC interface
///
/// This trait defines all RPC methods available in Seafile daemon.
/// The #[rpc] macro automatically generates the implementation for any
/// SearpcClient<T> where T implements Transport.
///
/// # Example
///
/// ```rust,ignore
/// use searpc::UnixSocketTransport;
///
/// let transport = UnixSocketTransport::connect("/path/to/seafile.sock", "seafile-rpcserver")?;
/// let mut client = SearpcClient::new(transport);
///
/// // Now you can call any method from SeafileRpc trait
/// let repos = client.get_repo_list(-1, -1)?;
/// ```
#[rpc(prefix = "seafile")]
pub trait SeafileRpc {
    /// Get list of repositories
    ///
    /// # Arguments
    /// * `start` - Starting index (-1 for all)
    /// * `limit` - Maximum number of repos (-1 for all)
    fn get_repo_list(&mut self, start: i32, limit: i32) -> Result<Vec<Repo>>;

    /// Get clone tasks
    fn get_clone_tasks(&mut self) -> Result<Vec<CloneTask>>;

    /// Get sync task for a specific repository
    ///
    /// Returns None if no sync task exists for the repository
    fn get_repo_sync_task(&mut self, repo_id: &str) -> Result<Option<SyncTask>>;

    /// Find transfer task for a repository
    fn find_transfer_task(&mut self, repo_id: &str) -> Result<TransferTask>;

    /// Check if auto sync is enabled
    #[rpc(name = "seafile_is_auto_sync_enabled")]
    fn is_auto_sync_enabled(&mut self) -> Result<bool>;

    /// Convert sync error ID to human-readable string
    fn sync_error_id_to_str(&mut self, error_id: i32) -> Result<String>;

    /// Get configuration value
    fn get_config(&mut self, key: &str) -> Result<String>;

    /// Set configuration value
    fn set_config(&mut self, key: &str, value: &str) -> Result<i32>;

    /// Set configuration value as integer
    fn set_config_int(&mut self, key: &str, value: i32) -> Result<i32>;

    /// Remove a repository (destroy it)
    #[rpc(name = "seafile_destroy_repo")]
    fn remove_repo(&mut self, repo_id: &str) -> Result<i32>;

    /// Download a repository from server
    ///
    /// # Arguments
    /// * `repo_id` - Repository ID
    /// * `repo_version` - Repository version
    /// * `repo_name` - Repository name
    /// * `worktree` - Local directory path
    /// * `token` - Clone token from server
    /// * `passwd` - Repository password (None for non-encrypted repos)
    /// * `magic` - Encryption magic (None for non-encrypted repos)
    /// * `email` - User email
    /// * `random_key` - Random key for encrypted repos (None for non-encrypted)
    /// * `enc_version` - Encryption version
    /// * `more_info` - Additional info as JSON string
    #[allow(clippy::too_many_arguments)]
    fn download(
        &mut self,
        repo_id: &str,
        repo_version: i32,
        repo_name: &str,
        worktree: &str,
        token: &str,
        passwd: Option<&str>,
        magic: Option<&str>,
        email: &str,
        random_key: Option<&str>,
        enc_version: i32,
        more_info: &str,
    ) -> Result<Option<String>>;

    /// Clone a repository into existing folder
    ///
    /// Same arguments as download()
    #[allow(clippy::too_many_arguments)]
    fn clone(
        &mut self,
        repo_id: &str,
        repo_version: i32,
        repo_name: &str,
        worktree: &str,
        token: &str,
        passwd: Option<&str>,
        magic: Option<&str>,
        email: &str,
        random_key: Option<&str>,
        enc_version: i32,
        more_info: &str,
    ) -> Result<Option<String>>;

    /// Shutdown the seafile daemon
    fn shutdown(&mut self) -> Result<i32>;
}
