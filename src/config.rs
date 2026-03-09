use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub sessions: HashMap<String, Session>,
}

#[derive(Deserialize, Clone)]
pub struct Session {
    pub host: String,
    pub port: Option<u16>,
    pub tls: Option<bool>,
    pub ca_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub topics: Option<Vec<String>>,
}

impl Config {
    pub fn load() -> Option<Self> {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).ok(),
            Err(_) => Some(Self {
                sessions: HashMap::new(),
            }),
        }
    }
    pub fn get_sessions(&self, name: &str) -> Result<&Session, String> {
        self.sessions
            .get(name)
            .ok_or_else(|| format!("Session '{}' not found", name))
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("rmqtty")
        .join("config.toml")
}
