import { useEffect } from "react";
import InstanceCard from "../components/InstanceCard";
import { useInstances } from "../hooks/useInstances";
import type { Page } from "../types";
import styles from "./Home.module.css";

interface HomeProps {
    navigate: (page: Page) => void;
}

export default function Home({ navigate }: HomeProps) {
    const { instances, loading, refresh } = useInstances();

    useEffect(() => {
        refresh();
    }, [refresh]);

    return (
        <div className={styles.home}>
            <div className={styles.header}>
                <h1>My Packs</h1>
                <button className={styles.addBtn} onClick={() => navigate({ kind: "browse" })}>
                    + Add Pack
                </button>
            </div>

            {loading ? (
                <div className={styles.loading}>Loading...</div>
            ) : instances.length === 0 ? (
                <div className={styles.empty}>
                    <div className={styles.emptyIcon}>{"\u{1F3EE}"}</div>
                    <h2>Welcome to Lantern!</h2>
                    <p>You don't have any packs yet. Browse modpacks to get started.</p>
                    <button
                        className={styles.browseBtn}
                        onClick={() => navigate({ kind: "browse" })}
                    >
                        Browse Modpacks
                    </button>
                </div>
            ) : (
                <div className={styles.list}>
                    {instances.map((instance) => (
                        <InstanceCard key={instance.id} instance={instance} navigate={navigate} />
                    ))}
                </div>
            )}
        </div>
    );
}
