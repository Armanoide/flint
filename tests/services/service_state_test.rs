#[cfg(test)]
use flint::error::Result;
use flint::launchd_config::{LaunchdConfig, LaunchdConfigBuilder};
use flint::services::service_state::{ServiceState, ServiceStateData, ServiceStatus};
use std::fs;
use tempfile::TempDir;

fn foo_launchd_config() -> Result<LaunchdConfig> {
    Ok(LaunchdConfigBuilder::default()
        .program(Some("/usr/bin/foo".to_string()))
        .args(vec!["--option".to_string(), "value".to_string()])
        .stdout_path(Some("/var/log/foo_stdout.log".to_string()))
        .stderr_path(Some("/var/log/foo_stderr.log".to_string()))
        .working_directory("/usr/local/foo".to_string())
        .build()?)
}

#[test]
fn test_write_and_read_state() {
    let tmp = TempDir::new().unwrap();
    let formula = "demo_service";

    // Temporarily change /tmp/flint to our temp directory
    let dir = tmp.path().join("flint");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}.state.json", formula));

    // Manually override path for testing
    let data = ServiceStateData::new(vec![1234], ServiceStatus::Running);
    let data_str = serde_json::to_string_pretty(&data).unwrap();
    fs::write(&path, &data_str).unwrap();

    let read: ServiceStateData = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(read.pids(), vec![1234]);
    assert_eq!(*read.status(), ServiceStatus::Running);
}

#[test]
fn test_mark_stopped() {
    let formula = "stoppable_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());

    // Write initial running state
    state.mark_running(vec![5678]).unwrap();

    // Mark stopped
    state.mark_stopped().unwrap();
    let updated = state.read_state().unwrap();

    assert_eq!(*updated.status(), ServiceStatus::Stopped);
}

#[test]
fn test_mark_running_writes_correct_state() {
    let formula = "mark_running_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());

    state.mark_running(vec![4242]).unwrap();
    let read = state.read_state().unwrap();

    assert_eq!(read.pids(), vec![4242]);
    assert_eq!(*read.status(), ServiceStatus::Running);
}

#[test]
fn test_read_state_returns_stale_when_missing() {
    let formula = "missing_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());

    // No file written
    let data = state.read_state().unwrap();

    assert_eq!(data.pids(), Vec::<i32>::new());
    assert_eq!(*data.status(), ServiceStatus::Stale);
}

#[test]
fn test_get_pid_returns_correct_value() {
    let formula = "pid_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());

    state.mark_running(vec![7777]).unwrap();
    let data = state.read_state().unwrap();

    assert_eq!(data.pids(), [7777]);
}

#[test]
fn test_state_can_toggle_between_running_and_stopped() {
    let formula = "toggle_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());

    state.mark_running(vec![1000]).unwrap();
    let running = state.read_state().unwrap();
    assert_eq!(*running.status(), ServiceStatus::Running);

    state.mark_stopped().unwrap();
    let stopped = state.read_state().unwrap();
    assert_eq!(*stopped.status(), ServiceStatus::Stopped);

    state.mark_running(vec![2000]).unwrap();
    let running_again = state.read_state().unwrap();
    assert_eq!(*running_again.status(), ServiceStatus::Running);
    assert_eq!(running_again.pids(), vec![2000]);
}

#[test]
fn test_formula_returns_correct_value() {
    let formula = "test_formula_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());
    assert_eq!(state.formula(), formula);
}

#[test]
fn test_search_pids_returns_empty_on_no_process() {
    let formula = "dummy_service";
    let state = ServiceState::new(formula.to_string(), &foo_launchd_config().unwrap());

    // Most likely there is no process with a random binary name
    let pids = state.search_pids().unwrap();
    assert!(pids.is_empty());
}
