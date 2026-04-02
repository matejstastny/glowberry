import type { Instance } from "../types";
import { PlayIcon, RefreshIcon, SettingsIcon, SpinnerIcon, PackageIcon } from "./Icons";
import styles from "./ModpackRow.module.css";

interface ModpackRowProps {
    instance: Instance;
    onPlay: (id: string) => void;
    onUpdate?: (id: string) => void;
    onSettings?: (id: string) => void;
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

export default function ModpackRow({
    instance,
    onPlay,
    onUpdate,
    onSettings,
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

    return (
        <div className={styles.row} style={{ animationDelay: `${index * 30}ms` }}>
            <button
                className={`${styles.playBtn} ${isRunning ? styles.playBtnRunning : ""} ${isPreparing ? styles.playBtnPreparing : ""}`}
                onClick={handlePlay}
                disabled={busy}
                title={isRunning ? "Running" : isPreparing ? "Preparing..." : "Play"}
            >
                {busy ? <SpinnerIcon size={16} /> : <PlayIcon size={16} />}
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
                </div>
            </div>

            <div className={styles.actions}>
                <button
                    className={`${styles.actionBtn} ${updateAvailable ? styles.updateHighlight : ""}`}
                    onClick={(e) => {
                        e.stopPropagation();
                        onUpdate?.(instance.id);
                    }}
                    title="Update"
                >
                    <RefreshIcon size={15} />
                </button>
                <button
                    className={styles.actionBtn}
                    onClick={(e) => {
                        e.stopPropagation();
                        onSettings?.(instance.id);
                    }}
                    title="Settings"
                >
                    <SettingsIcon size={15} />
                </button>
            </div>
        </div>
    );
}
