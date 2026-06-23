use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpeedControlError {
    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Configuration validation failed: {0}")]
    ConfigValidation(String),

    #[error("Invalid player input: {0}")]
    InvalidInput(String),

    #[error("Target player offline or not found: {0}")]
    TargetOffline(String),
}
