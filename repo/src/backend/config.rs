#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub static_dir: String,
    pub encryption_key: String,
    /// Optional path to a file containing the current hex key. When set, this
    /// file is authoritative and key rotation overwrites it atomically.
    pub encryption_key_file: Option<String>,
    /// Directory where uploaded evidence chunks + diagnostic ZIPs land.
    pub storage_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        let encryption_key_file = std::env::var("ENCRYPTION_KEY_FILE").ok();
        // If a key file exists and is readable, prefer it over the env var.
        let encryption_key = if let Some(ref path) = encryption_key_file {
            match std::fs::read_to_string(path) {
                Ok(s) => {
                    let trimmed = s.trim().to_string();
                    if trimmed.is_empty() {
                        std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| "dev-key-placeholder".into())
                    } else {
                        trimmed
                    }
                }
                Err(_) => std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| "dev-key-placeholder".into()),
            }
        } else {
            std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| "dev-key-placeholder".into())
        };

        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .expect("PORT must be a number"),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://app.db".into()),
            static_dir: std::env::var("STATIC_DIR")
                .unwrap_or_else(|_| "static".into()),
            encryption_key,
            encryption_key_file,
            storage_dir: std::env::var("STORAGE_DIR")
                .unwrap_or_else(|_| "/app/storage".into()),
        }
    }
}
