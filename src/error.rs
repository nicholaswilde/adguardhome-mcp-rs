use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Environment variable {0} is missing")]
    MissingEnvironmentVariable(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
