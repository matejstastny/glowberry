import { invoke } from "@tauri-apps/api/core";
import type { Version } from "../types";

export async function listVersions(projectId: string): Promise<Version[]> {
    return invoke("list_versions", { projectId });
}
