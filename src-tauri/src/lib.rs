use std::{
    io,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Mutex,
};

use commands::{get_sidecar_status, read_config, save_config, SidecarState};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SidecarState {
            child: Mutex::new(None),
        })
        .setup(|app| {
            commands::ensure_config_file().map_err(io::Error::other)?;
            configure_tray(app)?;
            if let Err(error) = start_sidecar(app) {
                eprintln!("Failed to start LiteLLM Sidecar: {error}");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open_settings" => show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            read_config,
            save_config,
            get_sidecar_status
        ])
        .build(tauri::generate_context!())
        .expect("error while building AwesomeProxy")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                stop_sidecar(app);
            }
        });
}

fn configure_tray(app: &tauri::App) -> tauri::Result<()> {
    let open_settings =
        MenuItem::with_id(app, "open_settings", "Open Settings", true, None::<&str>)?;
    let status = MenuItem::with_id(
        app,
        "proxy_status",
        "Proxy Status: managed by AwesomeProxy",
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_settings, &status, &quit])?;

    let mut builder = TrayIconBuilder::with_id("awesomeproxy-tray")
        .tooltip("AwesomeProxy")
        .menu(&menu)
        .show_menu_on_left_click(true);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn start_sidecar(app: &tauri::App) -> Result<(), String> {
    let config = commands::read_config()?;
    let binary_path = sidecar_binary_path(app);

    if !binary_path.exists() {
        eprintln!(
            "LiteLLM Sidecar binary not found at {}. The UI will still start for configuration.",
            binary_path.display()
        );
        return Ok(());
    }

    let config_path = commands::config_path()?;
    let child = Command::new(&binary_path)
        .arg("--config")
        .arg(config_path)
        .arg("--port")
        .arg(config.app_settings.port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Failed to spawn LiteLLM Sidecar: {error}"))?;

    let state = app.state::<SidecarState>();
    let mut managed_child = state
        .child
        .lock()
        .map_err(|_| "Sidecar state lock was poisoned".to_string())?;
    *managed_child = Some(child);
    Ok(())
}

fn stop_sidecar(app: &tauri::AppHandle) {
    let state = app.state::<SidecarState>();
    let Ok(mut managed_child) = state.child.lock() else {
        return;
    };

    if let Some(mut child) = managed_child.take() {
        if let Ok(None) = child.try_wait() {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

fn sidecar_binary_path(app: &tauri::App) -> PathBuf {
    app.path()
        .resolve(
            "binaries/litellm-sidecar",
            tauri::path::BaseDirectory::Resource,
        )
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/litellm-sidecar")
        })
}

fn show_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
