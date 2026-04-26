import { invoke } from "@tauri-apps/api/core";
import type { MinecraftProfile } from "@/types";

/** Open the Microsoft login webview. Returns the auth URL for QR code display.
 *  Listen for "auth-complete" and "auth-error" events for the result. */
export async function startLogin(): Promise<string> {
    return invoke<string>("start_login");
}

/** Cancel an in-progress login (closes the webview). */
export async function cancelLogin(): Promise<void> {
    return invoke<void>("cancel_login");
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
