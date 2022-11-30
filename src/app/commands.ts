'use client';

import {invoke} from "@tauri-apps/api";
import {PreConfig} from "../bindings/PreConfig";
import {ScanStatus} from "../bindings/ScanStatus";
import {ExclusionActionBatch} from "../bindings/ExclusionActionBatch";

export const getConfig = async () => {
  return await invoke<PreConfig>("get_config");
}

export const setConfig = async (config: PreConfig) => {
  return await invoke<void>("set_config", {config});
}

export const scanStatus = async () => {
  return await invoke<ScanStatus>("scan_status");
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