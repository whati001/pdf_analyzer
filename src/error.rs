use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to load PDF '{path}': {reason}")]
    PdfLoad { path: String, reason: String },

    #[error("Failed to render page {page}: {reason}")]
    RenderError { page: usize, reason: String },

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Analyzer '{analyzer}' failed on '{file}': {reason}")]
    AnalyzerError {
        analyzer: String,
        file: String,
        reason: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
