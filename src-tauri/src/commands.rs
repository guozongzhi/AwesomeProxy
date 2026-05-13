use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::Child,
    sync::Mutex,
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub proxy: ProxyConfig,
    pub providers: Vec<ProviderConfig>,
    pub routes: RouteConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteConfig {
    pub default_provider: String,
    pub default_model: String,
}

#[derive(Debug, Serialize)]
pub struct SaveConfigResponse {
    pub config_path: String,
    pub reloaded: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SidecarStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub message: String,
}

pub struct SidecarState {
    pub child: Mutex<Option<Child>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                host: "127.0.0.1".to_string(),
                port: 4000,
            },
            providers: vec![ProviderConfig {
                name: "deepseek".to_string(),
                api_key: String::new(),
                base_url: "https://api.deepseek.com".to_string(),
                model: "deepseek-chat".to_string(),
            }],
            routes: RouteConfig {
                default_provider: "deepseek".to_string(),
                default_model: "deepseek-chat".to_string(),
            },
        }
    }
}

pub fn config_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|home| home.join(".awesomeproxy"))
        .ok_or_else(|| "Unable to locate the current user's home directory".to_string())
}

pub fn config_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("config.yaml"))
}

pub fn ensure_config_file() -> Result<PathBuf, String> {
    let path = config_path()?;
    if !path.exists() {
        write_config(&AppConfig::default())?;
    }
    Ok(path)
}

pub fn write_config(config: &AppConfig) -> Result<PathBuf, String> {
    let directory = config_dir()?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Failed to create config directory: {error}"))?;

    let path = directory.join("config.yaml");
    let yaml = serde_yaml::to_string(config)
        .map_err(|error| format!("Failed to serialize config to YAML: {error}"))?;
    fs::write(&path, yaml).map_err(|error| format!("Failed to write config file: {error}"))?;
    Ok(path)
}

#[tauri::command]
pub fn read_config() -> Result<AppConfig, String> {
    let path = ensure_config_file()?;
    let yaml = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read config file {}: {error}", path.display()))?;
    serde_yaml::from_str(&yaml)
        .map_err(|error| format!("Failed to parse config file {}: {error}", path.display()))
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<SaveConfigResponse, String> {
    let path = write_config(&config)?;
    let (reloaded, message) = reload_sidecar(&config.proxy.host, config.proxy.port);

    Ok(SaveConfigResponse {
        config_path: path.display().to_string(),
        reloaded,
        message,
    })
}

#[tauri::command]
pub fn get_sidecar_status(state: tauri::State<'_, SidecarState>) -> Result<SidecarStatus, String> {
    let mut child = state
        .child
        .lock()
        .map_err(|_| "Sidecar state lock was poisoned".to_string())?;

    match child.as_mut() {
        Some(process) => match process.try_wait() {
            Ok(Some(status)) => {
                *child = None;
                Ok(SidecarStatus {
                    running: false,
                    pid: None,
                    message: format!("LiteLLM Sidecar exited with status {status}"),
                })
            }
            Ok(None) => Ok(SidecarStatus {
                running: true,
                pid: Some(process.id()),
                message: "LiteLLM Sidecar is running".to_string(),
            }),
            Err(error) => Err(format!("Failed to query LiteLLM Sidecar status: {error}")),
        },
        None => Ok(SidecarStatus {
            running: false,
            pid: None,
            message: "LiteLLM Sidecar is not running".to_string(),
        }),
    }
}

fn reload_sidecar(host: &str, port: u16) -> (bool, String) {
    let address = format!("{host}:{port}");
    match TcpStream::connect_timeout(
        &address
            .parse()
            .unwrap_or_else(|_| ([127, 0, 0, 1], port).into()),
        Duration::from_millis(800),
    ) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
            let request = format!(
                "POST /health/reload HTTP/1.1\r\nHost: {address}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );

            if let Err(error) = stream.write_all(request.as_bytes()) {
                return (
                    false,
                    format!("Config saved, but reload request failed: {error}"),
                );
            }

            let mut response = String::new();
            match stream.read_to_string(&mut response) {
                Ok(_) if response.contains(" 2") || response.starts_with("HTTP/1.1 200") => (
                    true,
                    "Config saved and LiteLLM Sidecar reload was requested".to_string(),
                ),
                Ok(_) => (
                    false,
                    "Config saved, but LiteLLM Sidecar did not confirm reload".to_string(),
                ),
                Err(error) => (
                    false,
                    format!("Config saved, but reload response could not be read: {error}"),
                ),
            }
        }
        Err(error) => (
            false,
            format!("Config saved, but LiteLLM Sidecar reload endpoint is unavailable: {error}"),
        ),
    }
}
