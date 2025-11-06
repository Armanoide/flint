use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct ServiceUserAgent {
    formula: String,
    formula_plist_path: String,
}

impl ServiceUserAgent {
    pub fn launch_agents_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")?;
        let formula_path = Path::new(&home).join("Library").join("LaunchAgents");
        Ok(formula_path)
    }

    pub fn find_plist(formula: &str) -> Result<Option<PathBuf>> {
        let formula_path = Self::launch_agents_dir()?;

        for entry in std::fs::read_dir(&formula_path)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.ends_with(&format!("{}.plist", formula)) {
                return Ok(Some(entry.path()));
            }
        }

        Ok(None)
    }

    pub fn new(formula: String) -> Result<Self> {
        let formula_plist_path = Self::find_plist(&formula)?.ok_or(Error::PlistNotFound {
            formula: formula.clone(),
        })?;
        if !formula_plist_path.exists() {
            return Err(Error::PlistNotFound { formula });
        }

        Ok(ServiceUserAgent {
            formula,
            formula_plist_path: formula_plist_path.display().to_string(),
        })
    }

    fn transform(name: &str) -> String {
        // Remove the .plist suffix if present
        let name = name.strip_suffix(".plist").unwrap_or(name);

        // Split by '.'
        let parts: Vec<&str> = name.split('.').collect();
        let n = parts.len();

        if n >= 3 {
            format!("{}.{}", parts[n - 2], parts[n - 1])
        } else if n >= 2 {
            parts[n - 1].to_string()
        } else {
            name.to_string()
        }
    }

    pub fn formulas() -> Result<Vec<String>> {
        let path = Self::launch_agents_dir()?;

        let formulas: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name())
            .map(|f| f.display().to_string())
            .filter_map(|plist| {
                let formula = Self::transform(&plist);
                if formula.starts_with("mxcl.") {
                    None
                } else {
                    Some(formula)
                }
            })
            .collect();
        Ok(formulas)
    }

    pub fn formula_plist_path(&self) -> &str {
        &self.formula_plist_path
    }

    pub fn formula(&self) -> &str {
        &self.formula
    }
}
