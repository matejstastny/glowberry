import { invoke } from "@tauri-apps/api/core";
import type { DeviceCodeInfo, LoginPollResult, MinecraftProfile } from "@/types";

export async function startLogin(): Promise<DeviceCodeInfo> {
    return invoke<DeviceCodeInfo>("start_login");
}

export async function checkLoginStatus(): Promise<LoginPollResult> {
    return invoke<LoginPollResult>("check_login_status");
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
