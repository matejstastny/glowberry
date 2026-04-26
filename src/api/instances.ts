import { invoke } from "@tauri-apps/api/core";
import type { Instance } from "../types";

export async function listInstances(): Promise<Instance[]> {
    return invoke("list_instances");
}

export async function getInstance(id: string): Promise<Instance> {
    return invoke("get_instance", { id });
}

export async function deleteInstance(id: string): Promise<void> {
    return invoke("delete_instance", { id });
}

export async function setInstanceMemory(id: string, memoryMb: number): Promise<void> {
    return invoke("set_instance_memory", { id, memoryMb });
}
