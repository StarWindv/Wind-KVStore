use anyhow::Result;
use dirs::home_dir;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use toml::Value;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 14514;

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: DEFAULT_HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

pub fn load_config() -> Result<ServerConfig> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        create_default_config(&config_path)?;
        return Ok(ServerConfig::default());
    }

    let config_content = fs::read_to_string(&config_path)?;
    let config_value: Value = toml::from_str(&config_content)?;

    let server_config = config_value.get("server").and_then(|s| {
        Some(ServerConfig {
            host: s.get("host")
                .and_then(|h| h.as_str())
                .unwrap_or(DEFAULT_HOST)
                .to_string(),
            port: s.get("port")
                .and_then(|p| p.as_integer())
                .map(|p| p as u16)
                .unwrap_or(DEFAULT_PORT),
        })
    }).unwrap_or_default();

    Ok(server_config)
}

fn get_config_path() -> Result<PathBuf> {
    let mut path = home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
    path.push(".stv_project");
    fs::create_dir_all(&path)?;
    path.push("wind-settings.toml");
    Ok(path)
}

fn create_default_config(path: &PathBuf) -> Result<()> {
    let default_config = r#"
[server]
host = "127.0.0.1"
port = 14514
"#;
    fs::write(path, default_config)?;
    Ok(())
}