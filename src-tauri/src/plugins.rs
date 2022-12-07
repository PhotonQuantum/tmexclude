//! Background plugin.

use tauri::plugin::Plugin;
use tauri::{AppHandle, RunEvent, Window, WindowEvent, Wry};

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
    use std::{env, ptr};

    use auto_launch::{AutoLaunch, AutoLaunchBuilder};
    use cocoa::base::id;
    use cocoa::foundation::NSInteger;
    use objc::runtime::{Class, BOOL, NO};
    use tauri::plugin::{Builder, TauriPlugin};
    use tauri::{Manager, Runtime, State};
    use tracing::{error, instrument};

    pub fn init<R: Runtime>() -> TauriPlugin<R> {
        Builder::new("auto_launch")
            .invoke_handler(tauri::generate_handler![enable, disable, is_enabled])
            .setup(move |app| {
                let current_exe = env::current_exe()?;
                let auto_launch = AutoLaunchBuilder::new()
                    .set_app_name(&app.package_info().name)
                    .set_use_launch_agent(false)
                    .set_app_path(&current_exe.canonicalize()?.display().to_string())
                    .build()
                    .map_err(|e| e.to_string())?;
                let manager = LaunchManager(auto_launch);
                app.manage(manager);
                Ok(())
            })
            .build()
    }

    #[tauri::command]
    #[instrument(skip(manager))]
    async fn enable(manager: State<'_, LaunchManager>) -> Result<(), ()> {
        if !manager.enable() {
            error!("failed to enable auto launch");
        }
        Ok(())
    }

    #[tauri::command]
    #[instrument(skip(manager))]
    async fn disable(manager: State<'_, LaunchManager>) -> Result<(), ()> {
        if !manager.disable() {
            error!("Failed to disable auto launch");
        }
        Ok(())
    }

    #[tauri::command]
    #[instrument(skip(manager))]
    async fn is_enabled(manager: State<'_, LaunchManager>) -> Result<bool, ()> {
        Ok(manager.is_enabled())
    }

    struct LaunchManager(AutoLaunch);

    impl LaunchManager {
        fn enable(&self) -> bool {
            if let Some(cls) = Class::get("SMAppService") {
                let service: id = unsafe { msg_send![cls, mainAppService] };
                let result: BOOL =
                    unsafe { msg_send![service, registerAndReturnError: ptr::null_mut::<id>()] };
                let succ = !matches!(result, NO);
                if !succ {
                    error!("Failed to register app to login items through SMAppService");
                }
                succ
            } else {
                let r = self.0.enable();
                if let Err(ref e) = r {
                    error!(?e, "Failed to register app to login items");
                }
                r.is_ok()
            }
        }
        fn disable(&self) -> bool {
            if let Some(cls) = Class::get("SMAppService") {
                let service: id = unsafe { msg_send![cls, mainAppService] };
                let result: BOOL =
                    unsafe { msg_send![service, unregisterAndReturnError: ptr::null_mut::<id>()] };
                let succ = !matches!(result, NO);
                if !succ {
                    error!("Failed to unregister app to login items through SMAppService");
                }
                succ
            } else {
                let r = self.0.disable();
                if let Err(ref e) = r {
                    error!(?e, "Failed to unregister app to login items");
                }
                r.is_ok()
            }
        }
        fn is_enabled(&self) -> bool {
            if let Some(cls) = Class::get("SMAppService") {
                let service: id = unsafe { msg_send![cls, mainAppService] };
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
            } else {
                let r = self.0.is_enabled();
                if let Err(ref e) = r {
                    error!(?e, "Failed to check status of auto launch");
                }
                r.unwrap_or_default()
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
