import type { Instance, Page } from "../types";
import styles from "./InstanceCard.module.css";

interface InstanceCardProps {
    instance: Instance;
    navigate: (page: Page) => void;
}

function formatDate(dateStr: string | null): string {
    if (!dateStr) return "Never";
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    if (days === 0) return "Today";
    if (days === 1) return "Yesterday";
    if (days < 7) return `${days} days ago`;
    return date.toLocaleDateString();
}

const loaderLabels: Record<string, string> = {
    vanilla: "Vanilla",
    fabric: "Fabric",
    forge: "Forge",
    neoforge: "NeoForge",
    quilt: "Quilt",
};

export default function InstanceCard({ instance, navigate }: InstanceCardProps) {
    return (
        <div
            className={styles.card}
            role="button"
            tabIndex={0}
            onClick={() => navigate({ kind: "instance", id: instance.id })}
            onKeyDown={(e) => {
                if (e.key === "Enter") navigate({ kind: "instance", id: instance.id });
            }}
        >
            <div className={styles.icon}>
                {instance.modpack?.icon_url ? (
                    <img src={instance.modpack.icon_url} alt="" className={styles.packIcon} />
                ) : (
                    <div className={styles.iconPlaceholder}>{"\u{1F4E6}"}</div>
                )}
            </div>
            <div className={styles.info}>
                <div className={styles.name}>{instance.name}</div>
                <div className={styles.meta}>
                    {instance.minecraft_version} &middot;{" "}
                    {loaderLabels[instance.loader] ?? instance.loader}
                </div>
                <div className={styles.played}>Played {formatDate(instance.last_played)}</div>
            </div>
            <div className={styles.actions}>
                <button
                    className={styles.playBtn}
                    onClick={(e) => {
                        e.stopPropagation();
                    }}
                >
                    {"\u25B6"}
                </button>
            </div>
        </div>
    );
}
