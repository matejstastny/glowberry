import { invoke } from "@tauri-apps/api/core";

export async function launchInstance(
    instanceId: string,
    online: boolean,
    username?: string,
): Promise<void> {
    return invoke("launch_instance", { instanceId, online, username });
}
