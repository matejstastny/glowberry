import type { Instance } from "../types";
import {
    PlayIcon,
    RefreshIcon,
    SettingsIcon,
    SpinnerIcon,
    PackageIcon,
    TrashIcon,
    FolderIcon,
} from "./Icons";
import styles from "./ModpackRow.module.css";

interface ModpackRowProps {
    instance: Instance;
    onPlay: (id: string) => void;
    onUpdate?: (id: string) => void;
    onSettings?: (id: string) => void;
    onDelete?: (id: string) => void;
    onOpenFolder?: (id: string) => void;
    updateAvailable?: boolean;
    index?: number;
    isRunning?: boolean;
    isPreparing?: boolean;
}

const loaderLabel: Record<string, string> = {
    vanilla: "Vanilla",
    fabric: "Fabric",
    forge: "Forge",
    neoforge: "NeoForge",
    quilt: "Quilt",
};

function timeAgo(dateStr: string | null): string | null {
    if (!dateStr) return null;
    const date = new Date(dateStr);
    const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
    if (seconds < 60) return "Just now";
    if (seconds < 3600) {
        const m = Math.floor(seconds / 60);
        return `${m}m ago`;
    }
    if (seconds < 86400) {
        const h = Math.floor(seconds / 3600);
        return `${h}h ago`;
    }
    if (seconds < 604800) {
        const d = Math.floor(seconds / 86400);
        return `${d}d ago`;
    }
    return date.toLocaleDateString();
}

export default function ModpackRow({
    instance,
    onPlay,
    onUpdate,
    onSettings,
    onDelete,
    onOpenFolder,
    updateAvailable = false,
    index = 0,
    isRunning = false,
    isPreparing = false,
}: ModpackRowProps) {
    const busy = isRunning || isPreparing;

    function handlePlay(e: React.MouseEvent) {
        e.stopPropagation();
        if (!busy) onPlay(instance.id);
    }

    const versionText = instance.modpack?.version_name || instance.minecraft_version;
    const authorText = instance.loader !== "vanilla" ? loaderLabel[instance.loader] : null;
    const lastPlayed = timeAgo(instance.last_played);

    return (
        <div className={styles.row} style={{ animationDelay: `${index * 30}ms` }}>
            <button
                className={`${styles.playBtn} ${isRunning ? styles.playBtnRunning : ""} ${isPreparing ? styles.playBtnPreparing : ""}`}
                onClick={handlePlay}
                disabled={busy}
                title={isRunning ? "Running" : isPreparing ? "Preparing..." : "Play"}
            >
                {busy ? <SpinnerIcon size={14} /> : <PlayIcon size={14} />}
            </button>

            <div className={styles.icon}>
                {instance.modpack?.icon_url ? (
                    <img src={instance.modpack.icon_url} alt="" />
                ) : (
                    <div className={styles.iconPlaceholder}>
                        <PackageIcon size={22} />
                    </div>
                )}
            </div>

            <div className={styles.info}>
                <div className={styles.name}>{instance.name}</div>
                <div className={styles.meta}>
                    <span>{versionText}</span>
                    {authorText && (
                        <>
                            <span className={styles.dot}>&middot;</span>
                            <span>{authorText}</span>
                        </>
                    )}
                    {isPreparing && (
                        <>
                            <span className={styles.dot}>&middot;</span>
                            <span className={styles.preparing}>Preparing...</span>
                        </>
                    )}
                    {isRunning && (
                        <>
                            <span className={styles.dot}>&middot;</span>
                            <span className={styles.running}>Running</span>
                        </>
                    )}
                    {!busy && lastPlayed && (
                        <>
                            <span className={styles.dot}>&middot;</span>
                            <span className={styles.lastPlayed}>{lastPlayed}</span>
                        </>
                    )}
                </div>
            </div>

            <div className={styles.actions}>
                <button
                    className={styles.actionBtn}
                    onClick={(e) => {
                        e.stopPropagation();
                        onOpenFolder?.(instance.id);
                    }}
                    title="Open folder"
                >
                    <FolderIcon size={14} />
                </button>
                <button
                    className={`${styles.actionBtn} ${updateAvailable ? styles.updateHighlight : ""}`}
                    onClick={(e) => {
                        e.stopPropagation();
                        onUpdate?.(instance.id);
                    }}
                    title="Update"
                >
                    <RefreshIcon size={14} />
                </button>
                <button
                    className={styles.actionBtn}
                    onClick={(e) => {
                        e.stopPropagation();
                        onSettings?.(instance.id);
                    }}
                    title="Settings"
                >
                    <SettingsIcon size={14} />
                </button>
                <button
                    className={`${styles.actionBtn} ${styles.deleteBtn}`}
                    onClick={(e) => {
                        e.stopPropagation();
                        onDelete?.(instance.id);
                    }}
                    title="Delete"
                >
                    <TrashIcon size={14} />
                </button>
            </div>

            {isPreparing && <div className={styles.progressBar} />}
        </div>
    );
}
