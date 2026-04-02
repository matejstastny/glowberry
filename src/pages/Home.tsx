import { useEffect } from "react";
import ModpackRow from "../components/ModpackRow";
import { useInstances } from "../hooks/useInstances";
import styles from "./Home.module.css";

interface HomeProps {
    onPlay: (id: string) => void;
    runningInstance: string | null;
    preparingInstance: string | null;
    launchError: string | null;
    onDismissError?: () => void;
    refreshKey: number;
}

export default function Home({
    onPlay,
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
                <div className={styles.center}>No modpacks yet</div>
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
                        index={i}
                        isRunning={runningInstance === instance.id}
                        isPreparing={preparingInstance === instance.id}
                    />
                ))}
            </div>
        </div>
    );
}
