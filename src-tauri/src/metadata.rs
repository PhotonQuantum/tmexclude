use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct BuildMeta {
    pub version: String,
    pub timestamp: String,
}

impl Default for BuildMeta {
    fn default() -> Self {
        Self {
            version: env!("VERGEN_GIT_SEMVER").to_string(),
            timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
        }
    }
}

#[tauri::command]
pub fn build_meta() -> BuildMeta {
    BuildMeta::default()
}
