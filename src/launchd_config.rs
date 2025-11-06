use std::path::PathBuf;

use derive_builder::Builder;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Builder)]
#[builder(build_fn(error = "crate::error::Error"))]
pub struct LaunchdConfig {
    #[serde(alias = "Program")]
    program: Option<String>,

    #[serde(alias = "ProgramArguments", default)]
    args: Vec<String>,

    #[serde(alias = "WorkingDirectory", default = "default_dir")]
    working_directory: String,

    #[serde(alias = "StandardOutPath", default)]
    stdout_path: Option<String>,

    #[serde(alias = "StandardErrorPath", default)]
    stderr_path: Option<String>,
}

impl LaunchdConfig {
    pub fn binary_name(&self) -> String {
        let path_prog = PathBuf::from(self.program());
        path_prog
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn program(&self) -> &str {
        if let Some(prog) = &self.program {
            prog
        } else {
            &self.args[0]
        }
    }

    pub fn is_program_exist(&self) -> bool {
        std::path::Path::new(self.program()).exists()
    }

    pub fn args(&self) -> &[String] {
        if self.program.is_some() {
            &self.args
        } else {
            &self.args[1..]
        }
    }

    pub fn working_directory(&self) -> &str {
        &self.working_directory
    }

    pub fn stdout_path(&self) -> Option<&str> {
        self.stdout_path.as_deref()
    }

    pub fn stderr_path(&self) -> Option<&str> {
        self.stderr_path.as_deref()
    }
}

fn default_dir() -> String {
    ".".to_string()
}
