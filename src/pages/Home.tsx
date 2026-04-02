import { useEffect } from "react";
import ModpackRow from "../components/ModpackRow";
import { useInstances } from "../hooks/useInstances";
import styles from "./Home.module.css";

interface HomeProps {
    onPlay: (id: string) => void;
    onDelete?: (id: string) => void;
    onOpenFolder?: (id: string) => void;
    runningInstance: string | null;
    preparingInstance: string | null;
    launchError: string | null;
    onDismissError?: () => void;
    refreshKey: number;
}

export default function Home({
    onPlay,
    onDelete,
    onOpenFolder,
    runningInstance,
    preparingInstance,
    launchError,
    onDismissError,
    refreshKey,
}: HomeProps) {
    const { instances, loading, refresh } = useInstances();

    useEffect(() => {
        refresh();
    }, [refresh, refreshKey]);

    if (loading) {
        return (
            <div className={styles.home}>
                <div className={styles.center}>Loading...</div>
            </div>
        );
    }

    if (instances.length === 0) {
        return (
            <div className={styles.home}>
                <div className={styles.empty}>
                    <GlowberryIllustration />
                    <div className={styles.emptyTitle}>No modpacks yet</div>
                    <div className={styles.emptyHint}>
                        Click Add Pack to browse and install one
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className={styles.home}>
            {launchError && (
                <div className={styles.errorBanner}>
                    <pre className={styles.errorText}>{launchError}</pre>
                    {onDismissError && (
                        <button
                            className={styles.errorClose}
                            onClick={onDismissError}
                            title="Dismiss"
                        >
                            &times;
                        </button>
                    )}
                </div>
            )}
            <div className={styles.list}>
                {instances.map((instance, i) => (
                    <ModpackRow
                        key={instance.id}
                        instance={instance}
                        onPlay={onPlay}
                        onDelete={onDelete}
                        onOpenFolder={onOpenFolder}
                        index={i}
                        isRunning={runningInstance === instance.id}
                        isPreparing={preparingInstance === instance.id}
                    />
                ))}
            </div>
        </div>
    );
}

function GlowberryIllustration() {
    return (
        <svg
            width="72"
            height="72"
            viewBox="0 0 72 72"
            fill="none"
            className={styles.emptyIllustration}
        >
            {/* Vine */}
            <path
                d="M36 4 C36 4 34 18 30 26 C26 34 22 38 22 38"
                stroke="#3a5a3a"
                strokeWidth="2.5"
                strokeLinecap="round"
                fill="none"
            />
            <path
                d="M36 4 C36 4 38 16 42 24 C46 32 50 36 50 36"
                stroke="#3a5a3a"
                strokeWidth="2.5"
                strokeLinecap="round"
                fill="none"
            />
            {/* Leaves */}
            <ellipse cx="24" cy="20" rx="6" ry="3.5" transform="rotate(-30 24 20)" fill="#4a7a4a" opacity="0.7" />
            <ellipse cx="48" cy="18" rx="6" ry="3.5" transform="rotate(25 48 18)" fill="#4a7a4a" opacity="0.7" />
            {/* Berry glow */}
            <circle cx="36" cy="46" r="16" fill="#d4a24c" opacity="0.08" />
            <circle cx="36" cy="46" r="10" fill="#d4a24c" opacity="0.12" />
            {/* Berry body */}
            <circle cx="36" cy="46" r="13" fill="#c49238" />
            <circle cx="36" cy="46" r="13" fill="url(#berryGrad)" />
            {/* Berry highlight */}
            <ellipse cx="32" cy="41" rx="4" ry="3" fill="#e8c06a" opacity="0.5" />
            {/* Berry cap */}
            <path
                d="M30 35 C30 35 33 37 36 37 C39 37 42 35 42 35 C42 35 40 33 36 33 C32 33 30 35 30 35Z"
                fill="#5a8a5a"
            />
            <defs>
                <radialGradient id="berryGrad" cx="0.4" cy="0.35" r="0.6">
                    <stop offset="0%" stopColor="#e8c06a" />
                    <stop offset="100%" stopColor="#b07828" />
                </radialGradient>
            </defs>
        </svg>
    );
}
