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

pub mod auto_launch {
    use std::ptr;

    use cocoa::base::id;
    use cocoa::foundation::NSInteger;
    use objc::runtime::{BOOL, NO};
    use tauri::{Manager, Runtime, State};
    use tauri::plugin::{Builder, TauriPlugin};
    use tracing::{error, instrument};

    pub fn init<R: Runtime>() -> TauriPlugin<R> {
        Builder::new("auto_launch")
            .invoke_handler(tauri::generate_handler![enable, disable, is_enabled])
            .setup(move |app| {
                let manager = LaunchManager;
                app.manage(manager);
                Ok(())
            })
            .build()
    }

    #[tauri::command]
    #[instrument(skip(manager))]
    async fn enable(manager: State<'_, LaunchManager>) -> Result<(), ()> {
        manager.enable();
        Ok(())
    }

    #[tauri::command]
    #[instrument(skip(manager))]
    async fn disable(manager: State<'_, LaunchManager>) -> Result<(), ()> {
        manager.disable();
        Ok(())
    }

    #[tauri::command]
    #[instrument(skip(manager))]
    async fn is_enabled(manager: State<'_, LaunchManager>) -> Result<bool, ()> {
        Ok(manager.is_enabled())
    }

    struct LaunchManager;

    impl LaunchManager {
        fn enable(&self) -> bool {
            let service: id = unsafe { msg_send![class!(SMAppService), mainAppService] };
            let result: BOOL =
                unsafe { msg_send![service, registerAndReturnError: ptr::null_mut::<id>()] };
            !matches!(result, NO)
        }
        fn disable(&self) -> bool {
            let service: id = unsafe { msg_send![class!(SMAppService), mainAppService] };
            let result: BOOL =
                unsafe { msg_send![service, unregisterAndReturnError: ptr::null_mut::<id>()] };
            !matches!(result, NO)
        }
        fn is_enabled(&self) -> bool {
            let service: id = unsafe { msg_send![class!(SMAppService), mainAppService] };
            let r: NSInteger = unsafe { msg_send![service, status] };
            let status = SmAppServiceStatus::from(r);
            match status {
                SmAppServiceStatus::Enabled => true,
                SmAppServiceStatus::NotRegistered => false,
                _ => {
                    error!(?status, "Unexpected status");
                    false
                }
            }
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum SmAppServiceStatus {
        NotRegistered = 0,
        Enabled = 1,
        RequiresApproval = 2,
        NotFound = 3,
    }

    impl From<NSInteger> for SmAppServiceStatus {
        fn from(i: NSInteger) -> Self {
            match i {
                0 => SmAppServiceStatus::NotRegistered,
                1 => SmAppServiceStatus::Enabled,
                2 => SmAppServiceStatus::RequiresApproval,
                3 => SmAppServiceStatus::NotFound,
                _ => unreachable!(),
            }
        }
    }
}
