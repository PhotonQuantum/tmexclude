#![allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate objc;

use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use tap::TapFallible;
use tauri::{
    ActivationPolicy, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};
use tracing::{error, instrument};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use window_vibrancy::NSVisualEffectMaterial;

use tmexclude_lib::{
    ApplyErrors, ConfigManager, ExclusionActionBatch, Metrics, Mission, PreConfig, ScanStatus,
    Store,
};

use crate::decorations::WindowExt;
use crate::metadata::build_meta;
use crate::plugins::{BackgroundPlugin, EnvironmentPlugin};

mod decorations;
mod metadata;
mod plugins;

#[tauri::command]
#[instrument(skip(mission))]
fn metrics(mission: tauri::State<Arc<Mission>>) -> Arc<Metrics> {
    mission.metrics()
}

#[tauri::command]
#[instrument(skip(mission))]
fn get_config(mission: tauri::State<Arc<Mission>>) -> Arc<PreConfig> {
    mission.config()
}

#[tauri::command]
#[instrument(skip_all)]
fn set_config(mission: tauri::State<Arc<Mission>>, config: PreConfig) -> Result<(), String> {
    let mission = mission.inner().clone();
    mission.set_config(config).map_err(|e| e.to_string())
}

#[tauri::command]
#[instrument(skip(mission))]
fn scan_status(mission: tauri::State<Arc<Mission>>) -> ScanStatus {
    mission.scan_status()
}

#[tauri::command]
#[instrument(skip(mission))]
fn start_full_scan(mission: tauri::State<Arc<Mission>>) {
    mission.inner().clone().full_scan()
}

#[tauri::command]
#[instrument(skip(mission))]
fn stop_full_scan(mission: tauri::State<Arc<Mission>>) {
    mission.stop_full_scan();
}

#[tauri::command]
#[instrument(skip_all, fields(add = batch.add.len(), remove = batch.remove.len()))]
async fn apply_action_batch(
    mission: tauri::State<'_, Arc<Mission>>,
    batch: ExclusionActionBatch,
) -> Result<(), ApplyErrors> {
    let support_dump = mission.inner().config().support_dump;
    tauri::async_runtime::spawn_blocking(move || {
        let r = batch
            .apply(support_dump)
            .tap_err(|e| e.values().for_each(|e| error!(?e, "Apply batch failed")));
        ApplyErrors::from(r)
    })
    .await
    .expect("spawn_blocking failed")
}

#[tauri::command]
#[instrument(skip(mission))]
fn store_get(mission: tauri::State<Arc<Mission>>, key: &str) -> Option<serde_json::Value> {
    mission.store_get(key)
}

#[tauri::command]
#[instrument(skip(mission))]
fn store_set(mission: tauri::State<Arc<Mission>>, key: String, value: serde_json::Value) {
    mission.store_set(key, value)
}

#[tauri::command]
#[instrument(skip(mission))]
fn store_del(mission: tauri::State<Arc<Mission>>, key: &str) {
    mission.store_del(key)
}

fn system_tray() -> SystemTray {
    let preference = CustomMenuItem::new("preference", "Preference");
    let about = CustomMenuItem::new("about", "About");
    let quit = CustomMenuItem::new("quit", "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(preference)
        .add_item(about)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

fn main() {
    static PATH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""/.*""#).unwrap());
    let _guard = sentry::init((
        env!("SENTRY_DSN"),
        sentry::ClientOptions {
            release: Some(build_meta().version.into()),
            before_send: Some(Arc::new(|mut ev| {
                ev.message = ev
                    .message
                    .map(|s| PATH_RE.replace_all(&s, "\"<SENSITIVE>\"").to_string());
                Some(ev)
            })),
            before_breadcrumb: Some(Arc::new(|mut breadcrumb| {
                breadcrumb.message = breadcrumb
                    .message
                    .map(|s| PATH_RE.replace_all(&s, "\"<SENSITIVE>\"").to_string());
                Some(breadcrumb)
            })),
            ..Default::default()
        },
    ));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(sentry::integrations::tracing::layer())
        .init();

    let context = tauri::generate_context!();

    let config_manager = ConfigManager::new().unwrap();
    tauri::Builder::default()
        .system_tray(system_tray())
        .on_system_tray_event(|app, ev| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = ev {
                match id.as_str() {
                    "preference" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "about" => {
                        let window = app.get_window("about").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            }
        })
        .plugin(BackgroundPlugin)
        .plugin(EnvironmentPlugin)
        .plugin(plugins::auto_launch::init())
        .invoke_handler(tauri::generate_handler![
            metrics,
            get_config,
            set_config,
            scan_status,
            start_full_scan,
            stop_full_scan,
            apply_action_batch,
            build_meta,
            store_get,
            store_set,
            store_del
        ])
        .setup(move |app| {
            let store = Store::new(&app.path_resolver().app_config_dir().unwrap());
            app.manage(
                Mission::new_arc(app.handle(), config_manager, store)
                    .expect("failed to create mission"),
            );
            let main_window = app.get_window("main").unwrap();
            window_vibrancy::apply_vibrancy(
                &main_window,
                NSVisualEffectMaterial::Sidebar,
                None,
                None,
            )
            .expect("unable to apply vibrancy");
            main_window.set_trafficlights_position(20., 20.);
            app.set_activation_policy(ActivationPolicy::Accessory);
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
