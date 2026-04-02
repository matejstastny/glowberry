export interface Instance {
    id: string;
    name: string;
    minecraft_version: string;
    loader: "vanilla" | "fabric" | "forge" | "neoforge" | "quilt";
    loader_version: string | null;
    modpack: ModpackInfo | null;
    locked_files: string[];
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

export interface SearchResult {
    hits: SearchHit[];
    offset: number;
    limit: number;
    total_hits: number;
}

export interface SearchHit {
    project_id: string;
    slug: string;
    title: string;
    description: string;
    icon_url: string | null;
    author: string;
    downloads: number;
    project_type: string;
    client_side: string;
    server_side: string;
}

export interface Project {
    id: string;
    slug: string;
    title: string;
    description: string;
    body: string;
    icon_url: string | null;
    downloads: number;
    project_type: string;
    game_versions: string[];
    loaders: string[];
}

export interface Version {
    id: string;
    project_id: string;
    name: string;
    version_number: string;
    game_versions: string[];
    loaders: string[];
    files: VersionFile[];
    date_published: string;
}

export interface VersionFile {
    hashes: { sha1: string | null; sha512: string | null };
    url: string;
    filename: string;
    primary: boolean;
    size: number;
}

export interface FileEntry {
    path: string;
    name: string;
    is_directory: boolean;
    size: number;
    is_locked: boolean;
}

export type Page =
    | { kind: "home" }
    | { kind: "browse" }
    | { kind: "instance"; id: string }
    | { kind: "settings" }
    | { kind: "login" };

// Auth types

export interface MinecraftProfile {
    id: string;
    name: string;
}

// Install progress

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

// Game events

export interface GameLogEvent {
    instance_id: string;
    line: string;
    stream: "stdout" | "stderr";
}

export interface GameExitEvent {
    instance_id: string;
    exit_code: number | null;
    crash_log: string | null;
}
