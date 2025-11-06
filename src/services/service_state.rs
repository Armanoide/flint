use crate::{error::Result, launchd_config::LaunchdConfig};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf, process::Command};

/// Represents the running status of a service.
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Stale,
}

/// Stores information about a service instance, including its PID and status.
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct ServiceStateData {
    pids: Vec<i32>,
    status: ServiceStatus,
}

impl ServiceStateData {
    // used in tests
    #[allow(dead_code)]
    pub fn new(pids: Vec<i32>, status: ServiceStatus) -> Self {
        Self { pids, status }
    }

    pub fn status(&self) -> &ServiceStatus {
        &self.status
    }

    pub fn pids(&self) -> &[i32] {
        &self.pids
    }

    pub fn to_string(&self) -> String {
        match &self.status {
            ServiceStatus::Running => String::from("Running"),
            ServiceStatus::Stopped => String::from("Stopped"),
            ServiceStatus::Stale => String::from("Stale"),
        }
    }
}

/// Manages reading and writing of a service’s runtime state to disk.
pub struct ServiceState {
    formula: String,
    binary_name: String,
}

impl ServiceState {
    /// Creates a new `ServiceState` associated with the given formula.
    pub fn new(formula: impl Into<String>, launchd_config: &LaunchdConfig) -> Self {
        Self {
            formula: formula.into(),
            binary_name: launchd_config.binary_name(),
        }
    }

    pub fn search_pids(&self) -> Result<Vec<i32>> {
        let output = Command::new("pgrep")
            .args([self.binary_name.as_str()])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let pids = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.trim().parse::<i32>().ok())
            .collect();
        Ok(pids)
    }

    /// Returns the path to the JSON file that stores this service's state.
    fn state_file_path(&self) -> Result<PathBuf> {
        let dir = PathBuf::from("/tmp/flint");
        fs::create_dir_all(&dir)?;
        Ok(dir.join(format!("{}.state.json", self.formula)))
    }

    /// Writes the given PID and marks the service as running.
    pub fn mark_running(&self, pids: Vec<i32>) -> Result<()> {
        let data = ServiceStateData {
            pids,
            status: ServiceStatus::Running,
        };

        let path = self.state_file_path()?;
        let mut file = fs::File::create(path)?;
        serde_json::to_writer_pretty(&mut file, &data)?;
        file.flush()?;
        Ok(())
    }

    /// Reads the service’s state data from disk.
    pub fn read_state(&self) -> Result<ServiceStateData> {
        let data = match fs::read_to_string(self.state_file_path()?) {
            Ok(contents) => serde_json::from_str(&contents)?,
            Err(_) => ServiceStateData {
                pids: Vec::new(),
                status: ServiceStatus::Stale,
            },
        };
        Ok(data)
    }

    /// Marks the service as stopped and updates the state file.
    pub fn mark_stopped(&self) -> Result<()> {
        let mut data = self.read_state()?;
        data.status = ServiceStatus::Stopped;
        let path = self.state_file_path()?;
        let mut file = fs::File::create(path)?;
        serde_json::to_writer_pretty(&mut file, &data)?;
        file.flush()?;
        Ok(())
    }

    // used in tests
    #[allow(dead_code)]
    pub fn formula(&self) -> &str {
        &self.formula
    }

    /// Checks if the service is managed by launchctl (launchd)
    pub fn is_managed_by_launchctl(&self) -> Result<bool> {
        let output = Command::new("launchctl").arg("list").output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Homebrew launchd labels usually look like: homebrew.mxcl.<formula>
        Ok(stdout.contains(&format!("homebrew.mxcl.{}", self.formula)))
    }
}
