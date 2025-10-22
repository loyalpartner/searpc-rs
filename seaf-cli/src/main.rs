use anyhow::Result;
use clap::{Parser, Subcommand};
use searpc::{SearpcClient, UnixSocketTransport};
use std::path::PathBuf;

mod rpc_client;
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
    /// List local libraries
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Show syncing status
    Status,

    /// Configure seafile client
    Config {
        /// Configuration key
        #[arg(short = 'k', long)]
        key: String,

        /// Configuration value (if provided, set key to this value)
        #[arg(short = 'v', long)]
        value: Option<String>,
    },

    /// Stop seafile daemon
    Stop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine config directory
    let conf_dir = match cli.confdir {
        Some(dir) => dir,
        None => {
            let home = std::env::var("HOME")?;
            PathBuf::from(home).join(".ccnet")
        }
    };

    // Read seafile.ini to get socket path
    let seafile_ini = conf_dir.join("seafile.ini");
    let seafile_datadir = std::fs::read_to_string(&seafile_ini)?
        .trim()
        .to_string();

    let socket_path = PathBuf::from(&seafile_datadir).join("seafile.sock");

    // Create RPC client
    let transport = UnixSocketTransport::connect(&socket_path, "seafile-rpcserver")?;
    let mut client = SearpcClient::new(transport);

    // Execute command
    match cli.command {
        Commands::List { json } => {
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

        Commands::Status => {
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
                            println!("{:<50}\t{:<20}\t{:.1}%, {:.1}KB/s",
                                task.repo_name, "downloading", progress, rate);
                        }
                    }
                    "error" => {
                        let err = client.sync_error_id_to_str(task.error)?;
                        println!("{:<50}\t{:<20}\t{:<20}", task.repo_name, "error", err);
                    }
                    "done" => {
                        // Skip, will be shown in repo status
                    }
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
                    Ok(Some(task)) => {
                        match task.state.as_str() {
                            "uploading" | "downloading" => {
                                if let Ok(tx_task) = client.find_transfer_task(&repo.id) {
                                    let progress = if tx_task.block_total > 0 {
                                        (tx_task.block_done as f64 / tx_task.block_total as f64) * 100.0
                                    } else {
                                        0.0
                                    };
                                    let rate = tx_task.rate as f64 / 1024.0;
                                    println!("{:<50}\t{:<20}\t{:.1}%, {:.1}KB/s",
                                        repo.name, task.state, progress, rate);
                                }
                            }
                            "error" => {
                                let err = client.sync_error_id_to_str(task.error)?;
                                println!("{:<50}\t{:<20}\t{:<20}", repo.name, "error", err);
                            }
                            _ => {
                                println!("{:<50}\t{:<20}", repo.name, task.state);
                            }
                        }
                    }
                    Ok(None) => {
                        println!("{:<50}\t{:<20}", repo.name, "waiting for sync");
                    }
                    Err(_) => {
                        println!("{:<50}\t{:<20}", repo.name, "waiting for sync");
                    }
                }
            }
        }

        Commands::Config { key, value } => {
            if let Some(val) = value {
                client.set_config(&key, &val)?;
                println!("Set {} = {}", key, val);
            } else {
                let val = client.get_config(&key)?;
                println!("{} = {}", key, val);
            }
        }

        Commands::Stop => {
            match client.shutdown() {
                Ok(_) => println!("Seafile daemon stopped"),
                Err(_) => {
                    // Network error expected when daemon shuts down
                    println!("Seafile daemon stopping...");
                }
            }
        }
    }

    Ok(())
}
