import { invoke } from "@tauri-apps/api/core";
import type { Instance } from "@/types";

export async function listPresets(instanceId: string): Promise<string[]> {
    return invoke("list_presets", { instanceId });
}

export async function switchPreset(instanceId: string, presetName: string): Promise<Instance> {
    return invoke("switch_preset", { instanceId, presetName });
}

export async function openDataFolder(): Promise<void> {
    return invoke("open_data_folder");
}
