export interface Instance {
    id: string;
    name: string;
    minecraft_version: string;
    loader: "vanilla" | "fabric" | "forge" | "neoforge" | "quilt";
    loader_version: string | null;
    modpack: ModpackInfo | null;
    created_at: string;
    last_played: string | null;
    jvm_args: string[];
    memory_mb: number;
}

export interface ModpackInfo {
    project_id: string;
    version_id: string;
    version_name: string;
    project_slug: string;
    name: string;
    icon_url: string | null;
}

export interface Version {
    id: string;
    project_id: string;
    name: string;
    version_number: string;
    game_versions: string[];
    loaders: string[];
    date_published: string;
}

export interface MinecraftProfile {
    id: string;
    name: string;
}

export interface InstallProgress {
    stage:
        | "downloading"
        | "parsing"
        | "installing_mods"
        | "extracting_overrides"
        | "installing_loader"
        | "finalizing"
        | "complete"
        | "failed";
    message: string;
    current: number;
    total: number;
    bytes_downloaded: number;
    bytes_total: number;
    project_id: string;
}

export interface GameExitEvent {
    instance_id: string;
    exit_code: number | null;
    crash_log: string | null;
}

// GitHub release info
export interface GithubRelease {
    tag: string;
    mrpack_url: string;
    mrpack_name: string;
    mrpack_size: number;
}
