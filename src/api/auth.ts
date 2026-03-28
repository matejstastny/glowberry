import { invoke } from "@tauri-apps/api/core";
import type { MinecraftProfile } from "@/types";

export async function startLogin(): Promise<MinecraftProfile> {
    return invoke<MinecraftProfile>("start_login");
}

export async function getAuthStatus(): Promise<MinecraftProfile | null> {
    return invoke<MinecraftProfile | null>("get_auth_status");
}

export async function tryRestoreSession(): Promise<MinecraftProfile | null> {
    return invoke<MinecraftProfile | null>("try_restore_session");
}

export async function logout(): Promise<void> {
    return invoke<void>("logout");
}
