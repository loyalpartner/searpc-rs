use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use searpc::{SearpcClient, UnixSocketTransport};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

mod config;
mod http_client;
mod rpc_client;

use config::{check_daemon_running, init_config, DeviceIdManager, UserConfig};
use http_client::SeafileHttpClient;
use rpc_client::SeafileRpc as _;

/// Seafile command-line client
#[derive(Parser)]
#[command(name = "seaf-cli")]
#[command(about = "Command line interface for Seafile client", long_about = None)]
struct Cli {
    /// Config directory (default: ~/.ccnet)
    #[arg(short = 'c', long = "confdir", global = true)]
    confdir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize config directory
    Init {
        /// Parent directory to put seafile-data
        #[arg(short = 'd', long)]
        dir: PathBuf,
    },

    /// Start seafile daemon
    Start,

    /// Stop seafile daemon
    Stop,

    /// List local libraries
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// List remote libraries
    ListRemote {
        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Seafile server URL
        #[arg(short = 's', long)]
        server: Option<String>,

        /// Username
        #[arg(short = 'u', long)]
        username: Option<String>,

        /// Password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Token
        #[arg(short = 'T', long)]
        token: Option<String>,

        /// Two-factor authentication code
        #[arg(short = 'a', long)]
        tfa: Option<String>,

        /// User config file
        #[arg(short = 'C')]
        user_config: Option<PathBuf>,
    },

    /// Show syncing status
    Status,

    /// Download a library from seafile server
    Download {
        /// Library ID
        #[arg(short = 'l', long)]
        library: String,

        /// Seafile server URL
        #[arg(short = 's', long)]
        server: Option<String>,

        /// Directory to put the library
        #[arg(short = 'd', long)]
        dir: Option<PathBuf>,

        /// Username
        #[arg(short = 'u', long)]
        username: Option<String>,

        /// Password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Token
        #[arg(short = 'T', long)]
        token: Option<String>,

        /// Two-factor authentication code
        #[arg(short = 'a', long)]
        tfa: Option<String>,

        /// Library password (for encrypted repos)
        #[arg(short = 'e', long)]
        libpasswd: Option<String>,

        /// User config file
        #[arg(short = 'C')]
        user_config: Option<PathBuf>,
    },

    /// Download a library by name from seafile server
    DownloadByName {
        /// Library name
        #[arg(short = 'L', long)]
        libraryname: String,

        /// Seafile server URL
        #[arg(short = 's', long)]
        server: Option<String>,

        /// Directory to put the library
        #[arg(short = 'd', long)]
        dir: Option<PathBuf>,

        /// Username
        #[arg(short = 'u', long)]
        username: Option<String>,

        /// Password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Token
        #[arg(short = 'T', long)]
        token: Option<String>,

        /// Two-factor authentication code
        #[arg(short = 'a', long)]
        tfa: Option<String>,

        /// Library password (for encrypted repos)
        #[arg(short = 'e', long)]
        libpasswd: Option<String>,

        /// User config file
        #[arg(short = 'C')]
        user_config: Option<PathBuf>,
    },

    /// Sync a library with an existing folder
    Sync {
        /// Library ID
        #[arg(short = 'l', long)]
        library: String,

        /// Seafile server URL
        #[arg(short = 's', long)]
        server: Option<String>,

        /// Existing local folder
        #[arg(short = 'd', long)]
        folder: PathBuf,

        /// Username
        #[arg(short = 'u', long)]
        username: Option<String>,

        /// Password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Token
        #[arg(short = 'T', long)]
        token: Option<String>,

        /// Two-factor authentication code
        #[arg(short = 'a', long)]
        tfa: Option<String>,

        /// Library password (for encrypted repos)
        #[arg(short = 'e', long)]
        libpasswd: Option<String>,

        /// User config file
        #[arg(short = 'C')]
        user_config: Option<PathBuf>,
    },

    /// Desynchronize a library from seafile server
    Desync {
        /// Local folder
        #[arg(short = 'd', long)]
        folder: PathBuf,
    },

    /// Create a new library
    Create {
        /// Library name
        #[arg(short = 'n', long)]
        name: String,

        /// Library description
        #[arg(short = 't', long)]
        desc: String,

        /// Library password (for encrypted repos)
        #[arg(short = 'e', long)]
        libpasswd: Option<String>,

        /// Seafile server URL
        #[arg(short = 's', long)]
        server: Option<String>,

        /// Username
        #[arg(short = 'u', long)]
        username: Option<String>,

        /// Password
        #[arg(short = 'p', long)]
        password: Option<String>,

        /// Token
        #[arg(short = 'T', long)]
        token: Option<String>,

        /// Two-factor authentication code
        #[arg(short = 'a', long)]
        tfa: Option<String>,

        /// User config file
        #[arg(short = 'C')]
        user_config: Option<PathBuf>,
    },

    /// Configure seafile client
    Config {
        /// Configuration key
        #[arg(short = 'k', long)]
        key: String,

        /// Configuration value (if provided, set key to this value)
        #[arg(short = 'v', long)]
        value: Option<String>,
    },
}

fn main() -> Result<()> {
    // Initialize tracing with env filter
    // Set RUST_LOG=debug to see debug logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();

    let cli = Cli::parse();
    debug!("Parsed command line arguments");

    if let Commands::Init { dir } = &cli.command {
        let conf_dir = if let Some(dir) = cli.confdir.clone() {
            dir
        } else {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE")) // Windows fallback
                .context("HOME or USERPROFILE environment variable not set")?;
            PathBuf::from(home).join(".ccnet")
        };
        init_config(&conf_dir, dir)?;
        return Ok(());
    }

    // Determine config directory
    let conf_dir = match cli.confdir {
        Some(dir) => dir,
        None => {
            let home = std::env::var("HOME")?;
            PathBuf::from(home).join(".ccnet")
        }
    };

    // For start command, we don't need to read config yet
    if matches!(cli.command, Commands::Start) {
        return handle_start(&conf_dir);
    }

    // Read seafile.ini to get socket path
    let seafile_ini = conf_dir.join("seafile.ini");
    let seafile_datadir = fs::read_to_string(&seafile_ini)
        .context("Failed to read seafile.ini")?
        .trim()
        .to_string();

    let datadir_path = PathBuf::from(&seafile_datadir);

    // Execute command
    match cli.command {
        Commands::Init { .. } => unreachable!(),
        Commands::Start => unreachable!(),

        Commands::List { json } => {
            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);
            let repos = client.get_repo_list(-1, -1)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&repos)?);
            } else {
                println!("Name\tID\tPath");
                for repo in repos {
                    println!("{}\t{}\t{}", repo.name, repo.id, repo.worktree);
                }
            }
        }

        Commands::ListRemote {
            json,
            server,
            username,
            password,
            token,
            tfa,
            user_config,
        } => {
            let user_cfg = UserConfig::load(user_config.as_deref())?;
            let server_url = server.or(user_cfg.server).context("Server URL required")?;
            let username = username.or(user_cfg.user).context("Username required")?;

            let token = if let Some(t) = token.or(user_cfg.token) {
                t
            } else {
                let password = if let Some(p) = password {
                    p
                } else {
                    rpassword::prompt_password(format!("Enter password for user {}: ", username))?
                };

                let device_mgr = DeviceIdManager::new(&conf_dir, &datadir_path);
                let device_id = device_mgr.get_device_id()?;
                let http_client = SeafileHttpClient::new(&server_url);
                http_client.get_token(&username, &password, &device_id, tfa.as_deref())?
            };

            let http_client = SeafileHttpClient::new(&server_url);
            let repos = http_client.list_repos(&token)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&repos)?);
            } else {
                println!("Name\tID");
                for repo in repos {
                    println!("{}\t{}", repo.name, repo.id);
                }
            }
        }

        Commands::Status => {
            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            // Get clone tasks
            let tasks = client.get_clone_tasks()?;
            println!("# {:<50}\t{:<20}\t{:<20}", "Name", "Status", "Progress");

            for task in tasks {
                match task.state.as_str() {
                    "fetch" => {
                        if let Ok(tx_task) = client.find_transfer_task(&task.repo_id) {
                            let progress = if tx_task.block_total > 0 {
                                (tx_task.block_done as f64 / tx_task.block_total as f64) * 100.0
                            } else {
                                0.0
                            };
                            let rate = tx_task.rate as f64 / 1024.0;
                            println!(
                                "{:<50}\t{:<20}\t{:.1}%, {:.1}KB/s",
                                task.repo_name, "downloading", progress, rate
                            );
                        }
                    }
                    "error" => {
                        let err = client.sync_error_id_to_str(task.error)?;
                        println!("{:<50}\t{:<20}\t{:<20}", task.repo_name, "error", err);
                    }
                    "done" => {}
                    _ => {
                        println!("{:<50}\t{:<20}", task.repo_name, task.state);
                    }
                }
            }

            // Get repo sync status
            let repos = client.get_repo_list(-1, -1)?;
            for repo in repos {
                let auto_sync = client.is_auto_sync_enabled()?;
                if !auto_sync || !repo.auto_sync {
                    println!("{:<50}\t{:<20}", repo.name, "auto sync disabled");
                    continue;
                }

                match client.get_repo_sync_task(&repo.id) {
                    Ok(Some(task)) => match task.state.as_str() {
                        "uploading" | "downloading" => {
                            if let Ok(tx_task) = client.find_transfer_task(&repo.id) {
                                let progress = if tx_task.block_total > 0 {
                                    (tx_task.block_done as f64 / tx_task.block_total as f64) * 100.0
                                } else {
                                    0.0
                                };
                                let rate = tx_task.rate as f64 / 1024.0;
                                println!(
                                    "{:<50}\t{:<20}\t{:.1}%, {:.1}KB/s",
                                    repo.name, task.state, progress, rate
                                );
                            }
                        }
                        "error" => {
                            let err = client.sync_error_id_to_str(task.error)?;
                            println!("{:<50}\t{:<20}\t{:<20}", repo.name, "error", err);
                        }
                        _ => {
                            println!("{:<50}\t{:<20}", repo.name, task.state);
                        }
                    },
                    Ok(None) | Err(_) => {
                        println!("{:<50}\t{:<20}", repo.name, "waiting for sync");
                    }
                }
            }
        }

        Commands::Download {
            library,
            server,
            dir,
            username,
            password,
            token,
            tfa,
            libpasswd,
            user_config,
        } => {
            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            handle_download(
                &mut client,
                &conf_dir,
                &datadir_path,
                &library,
                server.as_deref(),
                dir.as_deref(),
                username.as_deref(),
                password.as_deref(),
                token.as_deref(),
                tfa.as_deref(),
                libpasswd.as_deref(),
                user_config.as_deref(),
            )?;
        }

        Commands::DownloadByName {
            libraryname,
            server,
            dir,
            username,
            password,
            token,
            tfa,
            libpasswd,
            user_config,
        } => {
            let user_cfg = UserConfig::load(user_config.as_deref())?;
            let server_url = server.or(user_cfg.server).context("Server URL required")?;
            let username = username.or(user_cfg.user).context("Username required")?;

            let token = get_or_create_token(
                &server_url,
                &username,
                password.as_deref(),
                token.as_deref(),
                tfa.as_deref(),
                user_cfg.token.as_deref(),
                &conf_dir,
                &datadir_path,
            )?;

            let http_client = SeafileHttpClient::new(&server_url);
            let repos = http_client.list_repos(&token)?;

            let library_id = repos
                .iter()
                .find(|r| r.name == libraryname)
                .map(|r| r.id.clone())
                .context("Library not found")?;

            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            handle_download(
                &mut client,
                &conf_dir,
                &datadir_path,
                &library_id,
                Some(&server_url),
                dir.as_deref(),
                Some(&username),
                password.as_deref(),
                Some(&token),
                tfa.as_deref(),
                libpasswd.as_deref(),
                user_config.as_deref(),
            )?;
        }

        Commands::Sync {
            library,
            server,
            folder,
            username,
            password,
            token,
            tfa,
            libpasswd,
            user_config,
        } => {
            if !folder.exists() {
                anyhow::bail!("Local directory does not exist");
            }

            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            handle_sync(
                &mut client,
                &conf_dir,
                &datadir_path,
                &library,
                server.as_deref(),
                &folder,
                username.as_deref(),
                password.as_deref(),
                token.as_deref(),
                tfa.as_deref(),
                libpasswd.as_deref(),
                user_config.as_deref(),
            )?;
        }

        Commands::Desync { folder } => {
            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            let repo_path = folder.canonicalize()?;
            let repos = client.get_repo_list(-1, -1)?;

            let repo = repos
                .iter()
                .find(|r| PathBuf::from(&r.worktree) == repo_path)
                .context("Not a library")?;

            println!("Desynchronize {}", repo.name);
            client.remove_repo(&repo.id)?;
        }

        Commands::Create {
            name,
            desc,
            libpasswd,
            server,
            username,
            password,
            token,
            tfa,
            user_config,
        } => {
            let user_cfg = UserConfig::load(user_config.as_deref())?;
            let server_url = server.or(user_cfg.server).context("Server URL required")?;
            let username = username.or(user_cfg.user).context("Username required")?;

            let token = get_or_create_token(
                &server_url,
                &username,
                password.as_deref(),
                token.as_deref(),
                tfa.as_deref(),
                user_cfg.token.as_deref(),
                &conf_dir,
                &datadir_path,
            )?;

            let http_client = SeafileHttpClient::new(&server_url);
            let repo_id = http_client.create_repo(&token, &name, &desc, libpasswd.as_deref())?;
            println!("{}", repo_id);
        }

        Commands::Config { key, value } => {
            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            if let Some(val) = value {
                client.set_config(&key, &val)?;
                println!("Set {} = {}", key, val);
            } else {
                let val = client.get_config(&key)?;
                println!("{} = {}", key, val);
            }
        }

        Commands::Stop => {
            let socket_path = datadir_path.join("seafile.sock");
            let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
            let mut client = SearpcClient::new(transport);

            match client.shutdown() {
                Ok(_) => println!("Seafile daemon stopped"),
                Err(_) => println!("Seafile daemon stopping..."),
            }
        }
    }

    Ok(())
}

/// Handle start command
fn handle_start(conf_dir: &Path) -> Result<()> {
    debug!("Starting daemon with conf_dir: {}", conf_dir.display());

    // Read seafile.ini
    let seafile_ini = conf_dir.join("seafile.ini");
    let seafile_datadir = fs::read_to_string(&seafile_ini)
        .context("Failed to read seafile.ini")?
        .trim()
        .to_string();

    let datadir_path = PathBuf::from(&seafile_datadir);
    let seafile_worktree = datadir_path
        .parent()
        .ok_or_else(|| anyhow!("Invalid data dir path: {}", datadir_path.display()))?
        .join("seafile");
    debug!(
        "Data dir: {}, Worktree: {}",
        datadir_path.display(),
        seafile_worktree.display()
    );

    // Check if daemon is already running
    check_daemon_running(&datadir_path)?;
    debug!("No existing daemon detected");

    info!("Starting seafile daemon");

    // Start seaf-daemon
    let status = Command::new("seaf-daemon")
        .arg("--daemon")
        .arg("-c")
        .arg(conf_dir)
        .arg("-d")
        .arg(&datadir_path)
        .arg("-w")
        .arg(&seafile_worktree)
        .status()
        .context("Failed to start seaf-daemon")?;

    if !status.success() {
        anyhow::bail!("Failed to start seafile daemon");
    }
    debug!("seaf-daemon process started");

    // Wait for daemon to start and set delete_confirm_threshold
    let socket_path = datadir_path.join("seafile.sock");
    debug!("Waiting for socket: {}", socket_path.display());

    for i in 0..4 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        debug!("Connection attempt {} of 4", i + 1);

        if let Ok(transport) = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver") {
            debug!("Connected to RPC server");
            let mut client = SearpcClient::new(transport);
            if client
                .set_config_int("delete_confirm_threshold", 1000000)
                .is_ok()
            {
                debug!("Set delete_confirm_threshold successfully");
                break;
            }
        }
        if i == 3 {
            warn!("Could not set delete_confirm_threshold");
        }
    }

    info!("Seafile daemon started successfully");
    Ok(())
}

/// Get or create authentication token
#[allow(clippy::too_many_arguments)]
fn get_or_create_token(
    server_url: &str,
    username: &str,
    password: Option<&str>,
    token: Option<&str>,
    tfa: Option<&str>,
    config_token: Option<&str>,
    conf_dir: &Path,
    datadir_path: &Path,
) -> Result<String> {
    if let Some(t) = token.or(config_token) {
        debug!("Using existing token");
        return Ok(t.to_string());
    }

    debug!("Obtaining new token from server");
    let password = if let Some(p) = password {
        p.to_string()
    } else {
        rpassword::prompt_password(format!("Enter password for user {}: ", username))?
    };

    let device_mgr = DeviceIdManager::new(conf_dir, datadir_path);
    let device_id = device_mgr.get_device_id()?;
    debug!(
        "Authenticating as {} with device {}",
        username,
        &device_id[..8]
    );

    let http_client = SeafileHttpClient::new(server_url);
    let token = http_client.get_token(username, &password, &device_id, tfa)?;
    debug!("Authentication successful");
    Ok(token)
}

/// Handle download command
#[allow(clippy::too_many_arguments)]
fn handle_download<T: searpc::Transport>(
    client: &mut SearpcClient<T>,
    conf_dir: &Path,
    datadir_path: &Path,
    repo_id: &str,
    server: Option<&str>,
    dir: Option<&Path>,
    username: Option<&str>,
    password: Option<&str>,
    token: Option<&str>,
    tfa: Option<&str>,
    libpasswd: Option<&str>,
    user_config: Option<&Path>,
) -> Result<()> {
    let user_cfg = UserConfig::load(user_config)?;
    let server_url = server
        .or(user_cfg.server.as_deref())
        .context("Server URL required")?;
    let username = username
        .or(user_cfg.user.as_deref())
        .context("Username required")?;

    let token = get_or_create_token(
        server_url,
        username,
        password,
        token,
        tfa,
        user_cfg.token.as_deref(),
        conf_dir,
        datadir_path,
    )?;

    let http_client = SeafileHttpClient::new(server_url);
    let download_info = http_client.get_repo_download_info(&token, repo_id)?;
    debug!("Received download_info from API:");
    debug!("  repo_name: {}", download_info.repo_name);
    debug!("  repo_version: {}", download_info.repo_version);
    debug!("  encrypted: '{}'", download_info.encrypted);
    debug!("  enc_version: {}", download_info.enc_version);
    debug!("  magic: '{}'", download_info.magic);
    debug!("  salt: '{}'", download_info.salt);
    debug!("  random_key: '{}'", download_info.random_key);

    let download_dir = if let Some(d) = dir {
        d.to_path_buf()
    } else {
        datadir_path
            .parent()
            .ok_or_else(|| anyhow!("Invalid data dir path: {}", datadir_path.display()))?
            .join("seafile")
    };

    let is_encrypted = !download_info.encrypted.is_empty() && download_info.encrypted != "0";

    info!("Starting to download library {}", repo_id);
    info!("Download directory: {}", download_dir.display());
    debug!(
        "Repository version: {}, name: {}",
        download_info.repo_version, download_info.repo_name
    );
    debug!(
        "Encrypted: {}, enc_version from API: {}",
        is_encrypted, download_info.enc_version
    );
    let repo_passwd = if is_encrypted {
        if let Some(pwd) = libpasswd {
            Some(pwd.to_string())
        } else {
            Some(rpassword::prompt_password(
                "Enter password for the library: ",
            )?)
        }
    } else {
        None
    };

    // Build more_info JSON (without random_key - it's a separate parameter)
    let mut more_info = serde_json::json!({
        "server_url": http_client.get_base_url(),
        "is_readonly": if download_info.permission.as_deref() == Some("r") { 1 } else { 0 },
    });
    if !download_info.salt.is_empty() {
        more_info["repo_salt"] = serde_json::json!(&download_info.salt);
    }

    debug!("RPC download call parameters:");
    debug!("  repo_id: {}", repo_id);
    debug!("  repo_version: {}", download_info.repo_version);
    debug!("  repo_name: {}", download_info.repo_name);
    debug!("  worktree: {}", download_dir.display());
    debug!("  token: {}...", &download_info.token[..8]);
    debug!(
        "  passwd: {}",
        if repo_passwd.is_some() {
            "Some(<provided>)"
        } else {
            "None"
        }
    );
    if download_info.magic.is_empty() {
        debug!("  magic: None");
    } else {
        debug!("  magic: Some({})", download_info.magic);
    }
    debug!("  email: {}", download_info.email);
    if download_info.random_key.is_empty() {
        debug!("  random_key: None");
    } else {
        debug!("  random_key: Some({})", download_info.random_key);
    }
    debug!("  enc_version: {}", download_info.enc_version);
    debug!("  more_info: {}", more_info.to_string());

    let download_dir_str = download_dir
        .to_str()
        .ok_or_else(|| anyhow!("Path contains invalid UTF-8: {}", download_dir.display()))?;

    client.download(
        repo_id,
        download_info.repo_version,
        &download_info.repo_name,
        download_dir_str,
        &download_info.token,
        repo_passwd.as_deref(), // None for non-encrypted, Some for encrypted
        if download_info.magic.is_empty() {
            None
        } else {
            Some(&download_info.magic)
        },
        &download_info.email,
        if download_info.random_key.is_empty() {
            None
        } else {
            Some(&download_info.random_key)
        },
        download_info.enc_version,
        &more_info.to_string(),
    )?;

    Ok(())
}

/// Handle sync command
#[allow(clippy::too_many_arguments)]
fn handle_sync<T: searpc::Transport>(
    client: &mut SearpcClient<T>,
    conf_dir: &Path,
    datadir_path: &Path,
    repo_id: &str,
    server: Option<&str>,
    folder: &Path,
    username: Option<&str>,
    password: Option<&str>,
    token: Option<&str>,
    tfa: Option<&str>,
    libpasswd: Option<&str>,
    user_config: Option<&Path>,
) -> Result<()> {
    let user_cfg = UserConfig::load(user_config)?;
    let server_url = server
        .or(user_cfg.server.as_deref())
        .context("Server URL required")?;
    let username = username
        .or(user_cfg.user.as_deref())
        .context("Username required")?;

    let token = get_or_create_token(
        server_url,
        username,
        password,
        token,
        tfa,
        user_cfg.token.as_deref(),
        conf_dir,
        datadir_path,
    )?;

    let http_client = SeafileHttpClient::new(server_url);
    debug!("Getting download info for repo: {}", repo_id);
    let download_info = http_client.get_repo_download_info(&token, repo_id)?;

    let is_encrypted = !download_info.encrypted.is_empty() && download_info.encrypted != "0";

    info!("Syncing library {} to folder {}", repo_id, folder.display());
    debug!(
        "Repository: {}, encrypted: {}, enc_version: {}",
        download_info.repo_name, is_encrypted, download_info.enc_version
    );
    let repo_passwd = if is_encrypted {
        if let Some(pwd) = libpasswd {
            Some(pwd.to_string())
        } else {
            Some(rpassword::prompt_password(
                "Enter password for the library: ",
            )?)
        }
    } else {
        None
    };

    // Build more_info JSON (without random_key - it's a separate parameter)
    let mut more_info = serde_json::json!({
        "server_url": http_client.get_base_url(),
        "is_readonly": if download_info.permission.as_deref() == Some("r") { 1 } else { 0 },
    });
    if !download_info.salt.is_empty() {
        more_info["repo_salt"] = serde_json::json!(&download_info.salt);
    }

    let folder_str = folder
        .to_str()
        .ok_or_else(|| anyhow!("Path contains invalid UTF-8: {}", folder.display()))?;

    client.clone(
        repo_id,
        download_info.repo_version,
        &download_info.repo_name,
        folder_str,
        &download_info.token,
        repo_passwd.as_deref(), // None for non-encrypted, Some for encrypted
        if download_info.magic.is_empty() {
            None
        } else {
            Some(&download_info.magic)
        },
        &download_info.email,
        if download_info.random_key.is_empty() {
            None
        } else {
            Some(&download_info.random_key)
        },
        download_info.enc_version,
        &more_info.to_string(),
    )?;

    Ok(())
}
