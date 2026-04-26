import { invoke } from "@tauri-apps/api/core";

export interface SettingsInfo {
    data_dir: string;
    default_data_dir: string;
    data_dir_override: string | null;
}

export async function getSettings(): Promise<SettingsInfo> {
    return invoke("get_settings");
}

export async function setDataDir(path: string | null): Promise<void> {
    return invoke("set_data_dir", { path });
}

export async function showMainWindow(): Promise<void> {
    return invoke("show_main_window");
}
