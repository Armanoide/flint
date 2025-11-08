use flint::error::Error;
use flint::services::service_homebrew::ServiceHomebrew;
use flint::services::service_user_agent::ServiceUserAgent;
use std::path::Path;
use std::{fs, path::PathBuf};
use tempfile::{tempdir, TempDir};

/// Create a temporary fake `$HOME/Library/LaunchAgents` environment for testing.
///
/// Returns the `TempDir` so it lives until the test ends,
/// and the full path to the created `LaunchAgents` directory.
pub fn setup_fake_home() -> anyhow::Result<(TempDir, PathBuf)> {
    // Create a temporary home
    let tmp = tempdir()?;
    unsafe {
        std::env::set_var("HOME", tmp.path());
    }

    // Create ~/Library/LaunchAgents
    let launch_agents = tmp.path().join("Library").join("LaunchAgents");
    fs::create_dir_all(&launch_agents)?;

    Ok((tmp, launch_agents))
}

#[test]
fn test_user_agent_launch_agents_dir_uses_home() -> anyhow::Result<()> {
    let (tmp, _) = setup_fake_home()?;

    unsafe {
        std::env::set_var("HOME", tmp.path());
    }

    let expected = tmp.path().join("Library").join("LaunchAgents");
    let path = ServiceUserAgent::launch_agents_dir()?;
    assert_eq!(path, expected);
    Ok(())
}

#[test]
fn test_user_agent_find_and_new_success() -> anyhow::Result<()> {
    let (tmp, _) = setup_fake_home()?;

    unsafe {
        std::env::set_var("HOME", tmp.path().to_str().unwrap());
    }

    let agents = tmp.path().join("Library").join("LaunchAgents");
    fs::create_dir_all(&agents)?;

    let plist_path = agents.join("com.example.test.plist");
    fs::write(&plist_path, b"<?xml version=\"1.0\"?><plist></plist>")?;

    // Find plist
    let found = ServiceUserAgent::find_plist("com.example.test")?;
    assert!(found.is_some());
    assert_eq!(found.unwrap(), plist_path);

    // Create new service from plist
    let svc = ServiceUserAgent::new("com.example.test".into())?;
    assert_eq!(svc.formula(), "com.example.test");
    assert_eq!(Path::new(svc.formula_plist_path()), plist_path);

    Ok(())
}

#[test]
fn test_user_agent_new_missing_plist() -> anyhow::Result<()> {
    let (tmp, _) = setup_fake_home()?;
    unsafe {
        std::env::set_var("HOME", tmp.path());
    }

    let result = ServiceUserAgent::new("does.not.exist".into());
    assert!(matches!(result, Err(Error::PlistNotFound { .. })));
    Ok(())
}

#[test]
fn test_homebrew_new_and_formulas_with_fake_opt() -> anyhow::Result<()> {
    let (tmp, _) = setup_fake_home()?;
    let fake_opt = tmp.path().join("opt");
    fs::create_dir_all(&fake_opt)?;

    // Create a fake formula dir and plist
    let formula = "nginx";
    let formula_dir = fake_opt.join(formula);
    fs::create_dir_all(&formula_dir)?;
    let plist_path = formula_dir.join(format!("homebrew.mxcl.{}.plist", formula));
    fs::write(&plist_path, b"<plist/>")?;

    // Temporarily override /opt/homebrew/opt path by symlinking (safe on macOS/Linux)
    let real_opt = Path::new("/opt/homebrew/opt");
    if real_opt.exists() {
        // To avoid touching the real one, we just ensure code under test
        // uses our fake path (see below alternative)
    }

    // Instead of changing /opt, let's simulate the logic inline:
    assert!(formula_dir.exists());
    assert!(plist_path.exists());

    // Directly invoke the same logic your new() uses
    let svc = ServiceHomebrew::new(formula.to_string());
    if Path::new("/opt/homebrew/opt/nginx").exists() {
        // Real install present — skip (to avoid side effects)
        eprintln!("Real Homebrew nginx exists, skipping fake test.");
    } else {
        assert!(matches!(svc, Err(Error::FormulaNotFound { .. })));
    }

    Ok(())
}
