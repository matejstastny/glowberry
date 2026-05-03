import { invoke } from "@tauri-apps/api/core";
import type { GithubRelease, Instance } from "../types";

export async function installModpack(projectId: string, versionId: string): Promise<Instance> {
    return invoke("install_modpack", { projectId, versionId });
}

export async function installStarlight(release: GithubRelease): Promise<Instance> {
    return invoke("install_starlight", {
        assetUrl: release.asset_url,
        assetName: release.asset_name,
        assetSize: release.asset_size,
        versionTag: release.tag,
    });
}
