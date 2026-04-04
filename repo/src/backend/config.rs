#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub static_dir: String,
    pub encryption_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .expect("PORT must be a number"),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://app.db".into()),
            static_dir: std::env::var("STATIC_DIR")
                .unwrap_or_else(|_| "static".into()),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "dev-key-placeholder".into()),
        }
    }
}
