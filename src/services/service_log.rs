use std::{fs, path::Path};

use crate::{error::Result, launchd_config::LaunchdConfig};

pub struct ServiceLog {
    formula: String,
    stdout_path: String,
    stderr_path: String,
}

impl ServiceLog {
    /// Create and initialize a new `ServiceLog` instance.
    pub fn new(formula: String, launchd_service: &LaunchdConfig) -> Result<Self> {
        let mut service_log = ServiceLog {
            formula,
            stdout_path: String::new(),
            stderr_path: String::new(),
        };

        service_log
            .resolve_paths((launchd_service.stdout_path(), launchd_service.stderr_path()))?;
        Ok(service_log)
    }

    /// Initialize log paths based on user config, plist, or default fallback.
    fn resolve_paths(&mut self, launchd_output_path: (Option<&str>, Option<&str>)) -> Result<()> {
        let home = std::env::var("HOME")?;
        let config_path = Path::new(&home)
            .join(".config")
            .join("flint")
            .join(format!("{}.json", self.formula));

        // 1️⃣ Prefer custom JSON config if available
        if config_path.exists() {
            #[derive(serde::Deserialize)]
            struct LogConfig {
                standard_out_path: Option<String>,
                standard_error_path: Option<String>,
            }

            let config_data = fs::read_to_string(&config_path)?;
            let config: LogConfig = serde_json::from_str(&config_data)?;

            if let Some(stdout) = config.standard_out_path {
                self.stdout_path = stdout;
            }
            if let Some(stderr) = config.standard_error_path {
                self.stderr_path = stderr;
            }

            return Ok(());
        }

        // 2️⃣ Use plist-provided paths if present
        let stdout_plist = launchd_output_path.0;
        let stderr_plist = launchd_output_path.1;

        if stdout_plist.is_some() || stderr_plist.is_some() {
            if let Some(stdout) = stdout_plist {
                self.stdout_path = stdout.to_string();
            }
            if let Some(stderr) = stderr_plist {
                self.stderr_path = stderr.to_string();
            }

            return Ok(());
        }

        // 3️⃣ Fallback to defaults

        self.stdout_path = format!("{}/Library/Logs/Flint/{}.log", home, self.formula);
        self.stderr_path = format!("{}/Library/Logs/Flint/{}_error.log", home, self.formula);
        Ok(())
    }

    pub fn create_log_dirs(&self) -> Result<()> {
        if let Some(parent) = Path::new(&self.stdout_path).parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = Path::new(&self.stderr_path).parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Return current stdout path.
    pub fn stdout_path(&self) -> &str {
        &self.stdout_path
    }

    /// Return current stderr path.
    pub fn stderr_path(&self) -> &str {
        &self.stderr_path
    }
}
