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
    pub repo_id: String,
    pub repo_name: String,
    pub state: String,
    #[serde(default)]
    pub error: i32,
}

/// Sync task information
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncTask {
    pub repo_id: String,
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

    /// Shutdown the seafile daemon
    fn shutdown(&mut self) -> Result<i32>;
}
