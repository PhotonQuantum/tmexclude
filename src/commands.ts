import {PreConfig} from "./bindings/PreConfig";
import {ScanStatus} from "./bindings/ScanStatus";
import {ExclusionActionBatch} from "./bindings/ExclusionActionBatch";
import {InvokeArgs} from "@tauri-apps/api/tauri";

const invoke = async <T>(cmd: string, args?: InvokeArgs) => {
  if (typeof window === "undefined") {
    return null;
  }
  const _invoke = await import("@tauri-apps/api").then((api) => api.invoke);
  return await _invoke<T>(cmd, args);
};

export const enableAutoStart = async () => {
  return await invoke<void>('plugin:auto_launch|enable')
}

export const disableAutoStart = async () => {
  return await invoke<void>('plugin:auto_launch|disable')
}

export const getAutoStart = async () => {
  return await invoke<boolean>('plugin:auto_launch|is_enabled') ?? false;
}

export const getStore = async (key: string) => {
  return await invoke<string | null>("store_get", {key});
}

export const setStore = async (key: string, value: any) => {
  return await invoke<void>("store_set", {key, value});
}

export const getConfig = async () => {
  return await invoke<PreConfig>("get_config");
}

export const setConfig = async (config: PreConfig) => {
  return await invoke<void>("set_config", {config});
}

export const scanStatus = async () => {
  return await invoke<ScanStatus>("scan_status") ?? {step: "idle"} as ScanStatus;
}

export const startFullScan = async () => {
  return await invoke<void>("start_full_scan");
}

export const stopFullScan = async () => {
  return await invoke<void>("stop_full_scan");
}

export const applyActionBatch = async (batch: ExclusionActionBatch) => {
  return await invoke<void>("apply_action_batch", {batch});
}