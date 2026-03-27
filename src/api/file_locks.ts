import { invoke } from "@tauri-apps/api/core";
import type { FileEntry } from "../types";

export async function listInstanceFiles(instanceId: string, search?: string): Promise<FileEntry[]> {
    return invoke("list_instance_files", { instanceId, search });
}

export async function setFileLock(
    instanceId: string,
    path: string,
    locked: boolean,
): Promise<void> {
    return invoke("set_file_lock", { instanceId, path, locked });
}

export async function getLockedFiles(instanceId: string): Promise<string[]> {
    return invoke("get_locked_files", { instanceId });
}
