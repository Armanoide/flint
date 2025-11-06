use std::process::Command;
use std::thread;
use std::time::Duration;

use nix::libc::kill;

use crate::error::{Error, Result};
use crate::launchd_config::LaunchdConfig;
use crate::services::service_homebrew::ServiceHomebrew;
use crate::services::service_log::ServiceLog;
use crate::services::service_state::{ServiceState, ServiceStatus};
use crate::services::service_user_agent::ServiceUserAgent;

pub enum ServiceType {
    Homebrew(ServiceHomebrew),
    UserAgent(ServiceUserAgent),
}

impl ServiceType {
    pub fn formula(&self) -> &str {
        match self {
            ServiceType::Homebrew(svc) => svc.formula(),
            ServiceType::UserAgent(svc) => svc.formula(),
        }
    }

    fn formula_plist_path(&self) -> &str {
        match self {
            ServiceType::Homebrew(svc) => svc.formula_plist_path(),
            ServiceType::UserAgent(svc) => svc.formula_plist_path(),
        }
    }
}

pub struct ServiceManager {
    service: ServiceType,
    launchd: LaunchdConfig,
    log: ServiceLog,
    state: ServiceState,
}

impl ServiceManager {
    pub fn new(formula: String) -> Result<Self> {
        let service = if let Ok(homebrew) = ServiceHomebrew::new(formula.clone()) {
            ServiceType::Homebrew(homebrew)
        } else if let Ok(user_agent) = ServiceUserAgent::new(formula.clone()) {
            ServiceType::UserAgent(user_agent)
        } else {
            return Err(Error::FormulaNotFound { formula });
        };

        let data = std::fs::read(&service.formula_plist_path())?;
        let launchd_service = plist::from_bytes::<LaunchdConfig>(data.as_slice())?;
        let log = ServiceLog::new(formula.clone(), &launchd_service)?;
        let stats = ServiceState::new(formula.clone(), &launchd_service);

        return Ok(ServiceManager {
            service,
            launchd: launchd_service,
            log,
            state: stats,
        });
    }

    pub fn start(&self) -> Result<()> {
        let state = self.state.read_state()?;
        if state.status() == &ServiceStatus::Running {
            println!("Service '{}' is already running.", self.service.formula());
            return Ok(());
        }

        if !self.launchd.is_program_exist() {
            return Err(Error::ProgramNotFound {
                formula: self.service.formula().to_string(),
                program: self.launchd.program().to_string(),
            });
        }
        self.log.create_log_dirs()?;
        let file_stdout = std::fs::File::create(&self.log.stdout_path())?;
        let file_stderr = std::fs::File::create(&self.log.stderr_path())?;
        println!(
            "Logging into paths:\n{}\n{}",
            self.log.stdout_path(),
            self.log.stderr_path()
        );
        let mut child = Command::new(self.launchd.program())
            .args(self.launchd.args())
            .current_dir(self.launchd.working_directory())
            .stdout(file_stdout)
            .stderr(file_stderr)
            .spawn()?;

        thread::sleep(Duration::from_millis(500));

        match child.try_wait()? {
            Some(status) => {
                println!("Service exited early with status: {}", status);
                Err(Error::ServiceFailedToStart {
                    formula: self.service.formula().to_string(),
                    code: status.code().unwrap_or(-1),
                })
            }
            None => {
                let pids = self.state.search_pids()?;
                self.state.mark_running(pids.clone())?;
                println!("Service '{}' started", self.service.formula());
                Ok(())
            }
        }
    }

    pub fn stop(&self) -> Result<()> {
        if self.state.is_managed_by_launchctl()? {
            let _ = Command::new("launchctl")
                .arg("unload")
                .arg(self.service.formula_plist_path())
                .status();
        }

        for pid in self.state.read_state()?.pids() {
            if 0 != unsafe { kill(*pid, 0) } {
                // PID does not exist and may have already exited
                continue;
            }
            let ret = unsafe { kill(*pid, nix::libc::SIGTERM) };
            if ret != 0 {
                return Err(Error::ServiceFailedToStop {
                    formula: self.service.formula().to_string(),
                    pid: *pid,
                    reason: std::io::Error::last_os_error().to_string(),
                });
            }
        }

        thread::sleep(Duration::from_millis(500));
        self.state.mark_stopped()?;
        println!("Service '{}' stopped successfully.", self.service.formula());
        Ok(())
    }

    fn print_state(&self) -> Result<()> {
        println!(
            "{:<20} {}",
            self.service.formula(),
            self.state.read_state()?.to_string()
        );
        Ok(())
    }

    pub fn states() -> Result<()> {
        let print_section = |title: &str, formulas: Vec<_>| {
            println!("{:-<30}", "");
            println!("{title}");
            println!("{:-<30}", "");
            formulas
                .into_iter()
                .filter_map(|f| ServiceManager::new(f).ok())
                .for_each(|s| {
                    if let Err(e) = s.print_state() {
                        eprintln!("Error printing state: {:?}", e);
                    }
                });
        };

        print_section("Homebrew service", ServiceHomebrew::formulas()?);
        print_section("Service user agent", ServiceUserAgent::formulas()?);

        Ok(())
    }

    pub fn state(&self) -> Result<()> {
        let state_data = self.state.read_state()?;
        if *state_data.status() == ServiceStatus::Running {
            println!("Service '{}' is running.", self.service.formula());
        } else {
            println!("Service '{}' is not running.", self.service.formula());
        }
        Ok(())
    }

    // used in tests
    #[allow(dead_code)]
    pub fn service(&self) -> &ServiceType {
        &self.service
    }
}
