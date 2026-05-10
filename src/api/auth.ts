import { invoke } from "@tauri-apps/api/core";
import type { DeviceCodeInfo, MinecraftProfile } from "@/types";

/** Start the device-code login flow. Returns the user code and verification URL
 *  to display. Listen for "auth-complete" and "auth-error" events for the result. */
export async function startLogin(): Promise<DeviceCodeInfo> {
    return invoke<DeviceCodeInfo>("start_login");
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
