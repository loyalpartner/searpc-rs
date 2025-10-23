use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Device ID management
pub struct DeviceIdManager {
    conf_dir: PathBuf,
    datadir: PathBuf,
}

impl DeviceIdManager {
    pub fn new(conf_dir: &Path, datadir: &Path) -> Self {
        Self {
            conf_dir: conf_dir.to_path_buf(),
            datadir: datadir.to_path_buf(),
        }
    }

    /// Get or create device ID
    pub fn get_device_id(&self) -> Result<String> {
        let id_file = self.datadir.join("id");

        // Try to read existing ID
        if id_file.exists() {
            return fs::read_to_string(&id_file)
                .context("Failed to read device ID")
                .map(|s| s.trim().to_string());
        }

        // Try to migrate from ccnet.conf
        let ccnet_conf = self.conf_dir.join("ccnet.conf");
        if ccnet_conf.exists() {
            debug!("Found ccnet.conf, attempting to migrate device ID");
            let content = fs::read_to_string(&ccnet_conf)?;
            for line in content.lines() {
                if let Some(id) = line.strip_prefix("ID = ") {
                    let device_id = id.trim().to_string();
                    info!("Migrating device ID from ccnet.conf");
                    write_secure_file(&id_file, &device_id)?;
                    return Ok(device_id);
                }
            }
        }

        // Create new device ID
        let device_id = generate_random_id(40);
        info!("Created new device ID: {}", &device_id[..8]); // Log first 8 chars only
        write_secure_file(&id_file, &device_id)?;
        Ok(device_id)
    }
}

/// Write file with secure permissions (0o600)
///
/// This ensures only the owner can read/write sensitive files.
/// On Unix: Creates file with 0o600 permissions
/// On non-Unix: Falls back to standard file write
fn write_secure_file(path: &Path, content: &str) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600) // Only owner can read/write
            .open(path)?;
        file.write_all(content.as_bytes())?;
    }

    #[cfg(not(unix))]
    {
        fs::write(path, content)?;
    }

    Ok(())
}

/// Generate random hexadecimal ID
///
/// Uses cryptographically secure random bytes and efficient hex encoding.
/// Performance: O(1) allocations instead of O(n).
fn generate_random_id(size: usize) -> String {
    use rand::RngCore;

    let mut rng = rand::thread_rng();
    let bytes_needed = size.div_ceil(2); // Each byte produces 2 hex chars
    let mut bytes = vec![0u8; bytes_needed];
    rng.fill_bytes(&mut bytes); // Fill all bytes at once

    // Convert to hex string and truncate to exact size
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()[..size]
        .to_string()
}

/// User configuration management (~/.seafile.conf)
pub struct UserConfig {
    pub server: Option<String>,
    pub user: Option<String>,
    pub token: Option<String>,
}

impl UserConfig {
    /// Load user config from file
    pub fn load(config_file: Option<&Path>) -> Result<Self> {
        let path = match config_file {
            Some(p) => p.to_path_buf(),
            None => {
                let home = std::env::var("HOME")?;
                PathBuf::from(home).join(".seafile.conf")
            }
        };

        if !path.exists() {
            return Ok(Self {
                server: None,
                user: None,
                token: None,
            });
        }

        let content = fs::read_to_string(&path)?;
        let mut server = None;
        let mut user = None;
        let mut token = None;

        let mut in_account_section = false;
        for line in content.lines() {
            let line = line.trim();
            if line == "[account]" {
                in_account_section = true;
                continue;
            }
            if line.starts_with('[') {
                in_account_section = false;
                continue;
            }
            if in_account_section {
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        "server" => server = Some(value.to_string()),
                        "user" => user = Some(value.to_string()),
                        "token" => token = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            server,
            user,
            token,
        })
    }
}

/// Initialize seafile configuration
pub fn init_config(conf_dir: &Path, parent_dir: &Path) -> Result<()> {
    if conf_dir.exists() {
        anyhow::bail!("{} already exists", conf_dir.display());
    }

    if !parent_dir.exists() {
        anyhow::bail!("{} does not exist", parent_dir.display());
    }

    // Create config directory
    fs::create_dir(conf_dir)?;

    // Create logs directory
    let logs_dir = conf_dir.join("logs");
    fs::create_dir(&logs_dir)?;

    // Create seafile.ini
    let seafile_ini = conf_dir.join("seafile.ini");
    let seafile_data = parent_dir.join("seafile-data");
    fs::write(&seafile_ini, seafile_data.to_string_lossy().as_bytes())?;

    // Create seafile-data directory
    if !seafile_data.exists() {
        fs::create_dir(&seafile_data)?;
    }

    info!(
        "Initialized seafile data directory: {} (config: {})",
        seafile_data.display(),
        seafile_ini.display()
    );

    Ok(())
}

/// Check if daemon is running
pub fn check_daemon_running(datadir: &Path) -> Result<()> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    let pidfile = datadir.join("seaf-daemon.pid");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600) // Only owner can read/write
        .open(&pidfile)?;

    // Try to get exclusive lock
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();

    unsafe {
        let ret = libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB);
        if ret == 0 {
            // Got lock, unlock it
            libc::flock(fd, libc::LOCK_UN);
            Ok(())
        } else {
            anyhow::bail!(
                "The seafile data directory {} is already used by another Seafile client instance",
                datadir.display()
            );
        }
    }
}
