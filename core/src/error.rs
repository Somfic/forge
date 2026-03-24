pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("config error: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("http client error: {0}")]
    HttpClientError(#[from] reqwest::Error),
    #[error("failed to read config '{path}': {source}")]
    ConfigReadError {
        path: String,
        source: std::io::Error,
    },
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("address parse error: {0}")]
    AddressParseError(#[from] std::net::AddrParseError),
    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),
}
