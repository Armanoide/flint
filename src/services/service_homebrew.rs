use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct ServiceHomebrew {
    formula: String,
    formula_plist_path: String,
}

impl ServiceHomebrew {
    pub fn new(formula: String) -> Result<Self> {
        let formula_path = format!("/opt/homebrew/opt/{}", formula);
        let formula_path = Path::new(&formula_path);
        if !formula_path.exists() {
            return Err(Error::FormulaNotFound { formula });
        }
        let formula_plist_path = formula_path.join(format!("homebrew.mxcl.{}.plist", formula));
        if !formula_plist_path.exists() {
            return Err(Error::PlistNotFound { formula });
        }

        Ok(ServiceHomebrew {
            formula,
            formula_plist_path: formula_plist_path.display().to_string(),
        })
    }

    pub fn formulas() -> Result<Vec<String>> {
        let formulas: Vec<_> = std::fs::read_dir("/opt/homebrew/opt/")?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .join(format!(
                        "homebrew.mxcl.{}.plist",
                        e.file_name().to_string_lossy()
                    ))
                    .exists()
            })
            .map(|e| e.file_name())
            .map(|f| f.display().to_string())
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
