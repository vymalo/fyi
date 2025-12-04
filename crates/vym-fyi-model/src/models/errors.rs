#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("CLI error: {0}")]
    CliError(#[from] clap::Error),

    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),

    #[error("SetGlobalDefaultError error: {source}")]
    SetGlobalDefaultError {
        #[from]
        source: tracing::dispatcher::SetGlobalDefaultError,
    },

    #[error("Server error: {0}")]
    Server(String),

    // CRL / OpenSSL FFI
    #[error("CRL/OpenSSL FFI error in {func}")]
    CrlFfi { func: &'static str },

    #[error("Missing environment variable: {name}")]
    MissingEnvVar { name: String },

    #[error("Configuration error: {0}")]
    Config(String),
}

// Convenience alias used where appropriate
pub type AppResult<T> = Result<T, AppError>;
