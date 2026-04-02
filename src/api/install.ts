import { invoke } from "@tauri-apps/api/core";
import type { Instance } from "../types";

export async function installModpack(projectId: string, versionId: string): Promise<Instance> {
    return invoke("install_modpack", { projectId, versionId });
}
