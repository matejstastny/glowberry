import { useEffect } from "react";
import ModpackRow from "../components/ModpackRow";
import { useInstances } from "../hooks/useInstances";
import styles from "./Home.module.css";

interface HomeProps {
    onPlay: (id: string) => void;
}

export default function Home({ onPlay }: HomeProps) {
    const { instances, loading, refresh } = useInstances();

    useEffect(() => {
        refresh();
    }, [refresh]);

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
            <div className={styles.list}>
                {instances.map((instance, i) => (
                    <ModpackRow key={instance.id} instance={instance} onPlay={onPlay} index={i} />
                ))}
            </div>
        </div>
    );
}
