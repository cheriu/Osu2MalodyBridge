use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub malody: MalodyConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    /// Public hostname for download URLs. Required.
    /// Examples: `malody.example.com`, `192.168.1.100`,`::1`
    #[serde(default)]
    pub host: String,
    /// Path to TLS certificate (PEM). Enables HTTPS when both cert and key are set.
    #[serde(default)]
    pub tls_cert: Option<String>,
    /// Path to TLS private key (PEM).
    #[serde(default)]
    pub tls_key: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MalodyConfig {
    pub server: MalodyServerConfig,
    pub osu: OsuConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MalodyServerConfig {
    #[serde(default = "default_api_version")]
    pub api: i32,
    #[serde(default = "default_api_version")]
    pub min: i32,
    #[serde(default)]
    pub welcome: String,
    #[serde(default = "default_tmp_dir")]
    pub tmp: String,
    /// When true, reject requests missing valid uid/key/api parameters.
    #[serde(default)]
    pub verify_client_auth: bool,
    /// Beatmap download mirror.
    #[serde(default)]
    pub mirror: DownloadMirror,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OsuConfig {
    #[serde(rename = "clientID", default)]
    pub client_id: Option<u64>,
    #[serde(rename = "clientSecret", default)]
    pub client_secret: Option<String>,
}

/// Beatmap download mirror.
#[derive(Deserialize, Clone, Debug, Default)]
pub enum DownloadMirror {
    #[serde(rename = "hinamizawa")]
    #[default]
    Hinamizawa,
    #[serde(rename = "catboy")]
    Catboy,
    #[serde(rename = "osudirect")]
    OsuDirect,
}

impl DownloadMirror {
    pub fn url_for(&self, mapset_id: u32) -> String {
        match self {
            Self::Hinamizawa => {
                format!("https://mirror.hinamizawa.ai/api/v1/hinai/d/{mapset_id}")
            }
            Self::Catboy => format!("https://catboy.best/d/{mapset_id}"),
            Self::OsuDirect => format!("https://osu.direct/api/d/{mapset_id}"),
        }
    }
}

fn default_port() -> u16 {
    8081
}

fn default_api_version() -> i32 {
    202310
}

fn default_tmp_dir() -> String {
    "./Osu2Malody".to_string()
}

impl Config {
    /// Load configuration from a YAML file, with env var overrides.
    ///
    /// File path: `CONFIG_PATH` env var, or `application.yml` in working directory.
    ///
    /// Env vars override YAML values:
    /// - `OSU_CLIENT_ID` → `malody.osu.clientID`
    /// - `OSU_CLIENT_SECRET` → `malody.osu.clientSecret`
    /// - `SERVER_PORT` → `server.port`
    pub fn load() -> Result<Self> {
        let config_path =
            std::env::var("CONFIG_PATH").unwrap_or_else(|_| "application.yml".to_string());

        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Cannot read config file '{}'", config_path))?;

        let mut config: Config = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file '{}'", config_path))?;

        // Environment variable overrides
        if let Ok(port) = std::env::var("SERVER_PORT") {
            config.server.port = port
                .parse()
                .context("SERVER_PORT must be a valid u16")?;
        }
        if let Ok(client_id) = std::env::var("OSU_CLIENT_ID") {
            config.malody.osu.client_id = Some(client_id
                .parse()
                .context("OSU_CLIENT_ID must be a valid u64")?);
        }
        if let Ok(client_secret) = std::env::var("OSU_CLIENT_SECRET") {
            config.malody.osu.client_secret = Some(client_secret);
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate the loaded configuration.
    fn validate(&self) -> Result<()> {
        if self.malody.osu.client_id.unwrap_or(0) == 0 {
            anyhow::bail!(
                "osu! client ID is not set. Set malody.osu.clientID in {} or OSU_CLIENT_ID env var.",
                std::env::var("CONFIG_PATH").unwrap_or_else(|_| "application.yml".to_string())
            );
        }
        if self.malody.osu.client_secret.as_deref().unwrap_or("").is_empty() {
            anyhow::bail!(
                "osu! client secret is not set. Set malody.osu.clientSecret in {} or OSU_CLIENT_SECRET env var.",
                std::env::var("CONFIG_PATH").unwrap_or_else(|_| "application.yml".to_string())
            );
        }
        if self.server.port == 0 {
            anyhow::bail!("Server port must not be 0");
        }
        if self.server.host.is_empty() {
            anyhow::bail!("server.host is required — set it to your server's public IP or domain");
        }
        match (&self.server.tls_cert, &self.server.tls_key) {
            (Some(_), None) => anyhow::bail!("tls_key is required when tls_cert is set"),
            (None, Some(_)) => anyhow::bail!("tls_cert is required when tls_key is set"),
            _ => {}
        }
        Ok(())
    }
}
