use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::Child,
    sync::Mutex,
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub app_settings: AppSettings,
    pub model_list: Vec<ModelConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ModelConfig {
    pub model_name: String,
    pub litellm_params: LiteLlmParams,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LiteLlmParams {
    pub model: String,
    pub api_key: String,
    pub api_base: String,
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
            app_settings: AppSettings {
                host: "127.0.0.1".to_string(),
                port: 4000,
            },
            model_list: vec![ModelConfig {
                model_name: "deepseek-chat".to_string(),
                litellm_params: LiteLlmParams {
                    model: "deepseek/deepseek-chat".to_string(),
                    api_key: String::new(),
                    api_base: "https://api.deepseek.com".to_string(),
                },
            }],
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

pub fn read_config_from_path(path: &Path) -> Result<AppConfig, String> {
    let yaml = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read config file {}: {error}", path.display()))?;
    serde_yaml::from_str(&yaml)
        .map_err(|error| format!("Failed to parse config file {}: {error}", path.display()))
}

pub fn write_config_to_path(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)
            .map_err(|error| format!("Failed to create config directory: {error}"))?;
    }

    let yaml = serde_yaml::to_string(config)
        .map_err(|error| format!("Failed to serialize config to YAML: {error}"))?;
    fs::write(path, yaml).map_err(|error| format!("Failed to write config file: {error}"))
}

pub fn write_config(config: &AppConfig) -> Result<PathBuf, String> {
    let path = config_path()?;
    write_config_to_path(&path, config)?;
    Ok(path)
}

#[tauri::command]
pub fn read_config() -> Result<AppConfig, String> {
    let path = ensure_config_file()?;
    read_config_from_path(&path)
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<SaveConfigResponse, String> {
    let path = write_config(&config)?;
    let (reloaded, message) = reload_sidecar(&config.app_settings.host, config.app_settings.port);

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
                Ok(_) => reload_message_from_http_response(&response),
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

fn reload_message_from_http_response(response: &str) -> (bool, String) {
    let status_code = response
        .lines()
        .next()
        .and_then(|status_line| status_line.split_whitespace().nth(1))
        .and_then(|status_code| status_code.parse::<u16>().ok());

    match status_code {
        Some(200..=299) => (
            true,
            "Config saved and LiteLLM Sidecar reload was requested".to_string(),
        ),
        Some(code) => (
            false,
            format!("Config saved, but LiteLLM Sidecar reload returned HTTP {code}"),
        ),
        None => (
            false,
            "Config saved, but LiteLLM Sidecar returned an invalid HTTP response".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn default_config_uses_litellm_model_list_shape() {
        let config = AppConfig::default();

        assert_eq!(config.app_settings.port, 4000);
        assert_eq!(config.model_list[0].model_name, "deepseek-chat");
        assert_eq!(
            config.model_list[0].litellm_params.model,
            "deepseek/deepseek-chat"
        );
    }

    #[test]
    fn config_round_trips_as_yaml() {
        let unique_name = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("awesomeproxy-{unique_name}.yaml"));
        let config = AppConfig::default();

        write_config_to_path(&path, &config).expect("config should be writable");
        let loaded = read_config_from_path(&path).expect("config should be readable");
        let _ = fs::remove_file(&path);

        assert_eq!(loaded, config);
    }

    #[test]
    fn reload_response_accepts_any_2xx_status() {
        let (reloaded, _) = reload_message_from_http_response("HTTP/1.1 204 No Content\r\n\r\n");
        assert!(reloaded);
    }

    #[test]
    fn reload_response_rejects_non_2xx_status() {
        let (reloaded, message) =
            reload_message_from_http_response("HTTP/1.1 500 Server Error\r\n\r\n");
        assert!(!reloaded);
        assert!(message.contains("HTTP 500"));
    }
}
