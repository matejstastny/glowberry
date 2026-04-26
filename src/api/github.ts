import { invoke } from "@tauri-apps/api/core";
import type { GithubRelease } from "@/types";

/**
 * Fetch the latest Starlight release from GitHub.
 * Returns null if there are no releases yet or no client mrpack is attached.
 */
export async function checkStarlightUpdate(): Promise<GithubRelease | null> {
    return invoke<GithubRelease | null>("check_starlight_update");
}
