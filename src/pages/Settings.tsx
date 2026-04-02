import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { ArrowLeftIcon, GithubIcon } from "../components/Icons";
import { getSettings, setDataDir } from "../api/settings";
import type { Page } from "../types";
import styles from "./Settings.module.css";

interface SettingsProps {
    navigate: (page: Page) => void;
}

export default function Settings({ navigate }: SettingsProps) {
    const [dataDir, setDataDir_] = useState("");
    const [defaultDataDir, setDefaultDataDir] = useState("");
    const [override_, setOverride] = useState<string | null>(null);
    const [saved, setSaved] = useState(false);

    useEffect(() => {
        getSettings().then((s) => {
            setDataDir_(s.data_dir);
            setDefaultDataDir(s.default_data_dir);
            setOverride(s.data_dir_override);
        });
    }, []);

    async function handlePickFolder() {
        const selected = await openDialog({
            directory: true,
            title: "Choose data directory",
            defaultPath: dataDir,
        });
        if (selected) {
            await setDataDir(selected);
            setOverride(selected);
            setDataDir_(selected);
            setSaved(true);
        }
    }

    async function handleReset() {
        await setDataDir(null);
        setOverride(null);
        setDataDir_(defaultDataDir);
        setSaved(true);
    }

    return (
        <div className={styles.settings}>
            <button className={styles.back} onClick={() => navigate({ kind: "home" })}>
                <ArrowLeftIcon size={16} />
                <span>Back</span>
            </button>

            <h1 className={styles.title}>Settings</h1>

            <div className={styles.section}>
                <div className={styles.sectionLabel}>General</div>
                <div className={styles.card}>
                    <div className={styles.row}>
                        <div>
                            <div className={styles.label}>Default Memory</div>
                            <div className={styles.desc}>RAM allocated to Minecraft</div>
                        </div>
                        <select className={styles.select} defaultValue="4096">
                            <option value="2048">2 GB</option>
                            <option value="4096">4 GB</option>
                            <option value="6144">6 GB</option>
                            <option value="8192">8 GB</option>
                        </select>
                    </div>
                </div>
            </div>

            <div className={styles.section}>
                <div className={styles.sectionLabel}>Storage</div>
                <div className={styles.card}>
                    <div className={styles.storageRow}>
                        <div className={styles.label}>Data Directory</div>
                        <div className={styles.desc}>
                            Where instances, libraries, and assets are stored
                        </div>
                        <div className={styles.pathDisplay}>
                            <code className={styles.pathText}>{dataDir}</code>
                            <button
                                className={styles.pathBtn}
                                onClick={() => openUrl(`file://${dataDir}`)}
                                title="Open in file manager"
                            >
                                Open
                            </button>
                        </div>

                        <div className={styles.overrideActions}>
                            {override_ && (
                                <div className={styles.overrideNote}>
                                    Custom path (default: {defaultDataDir})
                                </div>
                            )}
                            <button
                                className={styles.pathBtn}
                                onClick={handlePickFolder}
                            >
                                {override_ ? "Change" : "Set custom path"}
                            </button>
                            {override_ && (
                                <button
                                    className={styles.pathBtn}
                                    onClick={handleReset}
                                >
                                    Reset to default
                                </button>
                            )}
                        </div>

                        {saved && (
                            <div className={styles.restartNote}>
                                Restart Glowberry for changes to take effect
                            </div>
                        )}
                    </div>
                </div>
            </div>

            <div className={styles.section}>
                <div className={styles.sectionLabel}>About</div>
                <div className={styles.card}>
                    <div className={styles.aboutName}>Glowberry</div>
                    <div className={styles.aboutVersion}>v0.1.0</div>
                    <div className={styles.aboutDesc}>
                        A simple, fast Minecraft launcher.
                    </div>
                    <button
                        className={styles.githubBtn}
                        onClick={() => openUrl("https://github.com/matejstastny/glowberry")}
                    >
                        <GithubIcon size={14} />
                        <span>GitHub</span>
                    </button>
                </div>
            </div>
        </div>
    );
}
