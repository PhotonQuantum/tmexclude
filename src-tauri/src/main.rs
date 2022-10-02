#![allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

#[macro_use]
extern crate objc;
use std::sync::Arc;

use tauri::{ActivationPolicy, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use window_vibrancy::NSVisualEffectMaterial;

use tmexclude_lib::{Metrics, Mission};
use crate::decorations::WindowExt;

use crate::plugins::{BackgroundPlugin, EnvironmentPlugin};
use crate::utils::collect_config;

mod utils;
mod plugins;
mod decorations;

#[tauri::command]
fn metrics(mission: tauri::State<Arc<Mission>>) -> Arc<Metrics> {
    mission.metrics()
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
    let context = tauri::generate_context!();

    let config = Arc::new(collect_config(None).unwrap());
    let mission = Mission::new(config);
    tauri::Builder::default()
        .manage(mission)
        .system_tray(system_tray())
        .on_system_tray_event(|app, ev|
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
            })
        .plugin(BackgroundPlugin)
        .plugin(EnvironmentPlugin)
        .invoke_handler(tauri::generate_handler![metrics])
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            window_vibrancy::apply_vibrancy(&main_window, NSVisualEffectMaterial::Sidebar).expect("unable to apply vibrancy");
            main_window.set_transparent_titlebar();
            main_window.set_trafficlights_position(20., 20.);
            app.set_activation_policy(ActivationPolicy::Accessory);
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
