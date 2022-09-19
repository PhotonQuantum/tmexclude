//! Background plugin.

use tauri::{AppHandle, RunEvent, Window, WindowEvent, Wry};
use tauri::plugin::Plugin;

pub struct EnvironmentPlugin;

impl Plugin<Wry> for EnvironmentPlugin {
    fn name(&self) -> &'static str {
        "environment"
    }
    fn initialization_script(&self) -> Option<String> {
        #[cfg(debug_assertions)]
        return Some("window.__TAURI__.environment = \"development\"".to_string());
        #[cfg(not(debug_assertions))]
        return Some("window.__TAURI__.environment = \"production\"".to_string());
    }
}

pub struct BackgroundPlugin;

impl Plugin<Wry> for BackgroundPlugin {
    fn name(&self) -> &'static str {
        "background"
    }
    fn created(&mut self, window: Window<Wry>) {
        window.on_window_event({
            let window = window.clone();
            move |ev| {
                if let WindowEvent::CloseRequested { api, .. } = ev {
                    window.hide().expect("unable to hide window");
                    api.prevent_close();
                }
            }
        });
    }
    fn on_event(&mut self, _app: &AppHandle<Wry>, event: &RunEvent) {
        if let RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    }
}