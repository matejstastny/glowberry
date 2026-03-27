import { invoke } from "@tauri-apps/api/core";
import type { SearchResult, Project, Version } from "../types";

export async function searchModpacks(
    query: string,
    limit?: number,
    offset?: number,
): Promise<SearchResult> {
    return invoke("search_modpacks", { query, limit, offset });
}

export async function getProject(idOrSlug: string): Promise<Project> {
    return invoke("get_project", { idOrSlug });
}

export async function listVersions(projectId: string): Promise<Version[]> {
    return invoke("list_versions", { projectId });
}
