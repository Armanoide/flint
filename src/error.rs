use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("ParseInt error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("SerdeJSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Var error: {0}")]
    Var(#[from] std::env::VarError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Plist error: {0}")]
    Plist(#[from] plist::Error),
    #[error("No available formula with the name '{formula}'")]
    FormulaNotFound { formula: String },
    #[error("No plist found for formula '{formula}'")]
    PlistNotFound { formula: String },
    #[error("Service '{formula}' failed to start with exit code {code}")]
    ServiceFailedToStart { formula: String, code: i32 },
    #[error("Service '{formula}' with PID {pid} failed to stop: {reason}")]
    ServiceFailedToStop {
        formula: String,
        pid: i32,
        reason: String,
    },
    #[error("DeriveBuilder missing required field: {0}")]
    MissingField(String),
    #[error("Program '{program}' not found for formula '{formula}'")]
    ProgramNotFound { formula: String, program: String },
}

impl From<derive_builder::UninitializedFieldError> for Error {
    fn from(err: derive_builder::UninitializedFieldError) -> Self {
        Error::MissingField(err.field_name().to_string())
    }
}
