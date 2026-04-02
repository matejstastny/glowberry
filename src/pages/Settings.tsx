import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
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
    const [draftOverride, setDraftOverride] = useState("");
    const [editing, setEditing] = useState(false);
    const [saved, setSaved] = useState(false);

    useEffect(() => {
        getSettings().then((s) => {
            setDataDir_(s.data_dir);
            setDefaultDataDir(s.default_data_dir);
            setOverride(s.data_dir_override);
            setDraftOverride(s.data_dir_override ?? "");
        });
    }, []);

    async function handleSaveOverride() {
        const value = draftOverride.trim() || null;
        await setDataDir(value);
        setOverride(value);
        setEditing(false);
        setSaved(true);
    }

    function handleCancelEdit() {
        setDraftOverride(override_ ?? "");
        setEditing(false);
    }

    async function handleReset() {
        await setDataDir(null);
        setOverride(null);
        setDraftOverride("");
        setEditing(false);
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

                        {!editing ? (
                            <div className={styles.overrideActions}>
                                {override_ && (
                                    <div className={styles.overrideNote}>
                                        Custom path set (default: {defaultDataDir})
                                    </div>
                                )}
                                <button
                                    className={styles.pathBtn}
                                    onClick={() => setEditing(true)}
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
                        ) : (
                            <div className={styles.editRow}>
                                <input
                                    className={styles.pathInput}
                                    type="text"
                                    value={draftOverride}
                                    onChange={(e) => setDraftOverride(e.target.value)}
                                    placeholder={defaultDataDir}
                                    autoFocus
                                />
                                <button className={styles.pathBtn} onClick={handleSaveOverride}>
                                    Save
                                </button>
                                <button className={styles.pathBtn} onClick={handleCancelEdit}>
                                    Cancel
                                </button>
                            </div>
                        )}

                        {saved && (
                            <div className={styles.restartNote}>
                                Restart Lantern for changes to take effect
                            </div>
                        )}
                    </div>
                </div>
            </div>

            <div className={styles.section}>
                <div className={styles.sectionLabel}>About</div>
                <div className={styles.card}>
                    <div className={styles.aboutName}>Lantern</div>
                    <div className={styles.aboutVersion}>v0.1.0</div>
                    <div className={styles.aboutDesc}>
                        A simple Minecraft launcher built with care.
                    </div>
                    <button
                        className={styles.githubBtn}
                        onClick={() => openUrl("https://github.com/matejstastny/lantern")}
                    >
                        <GithubIcon size={14} />
                        <span>GitHub</span>
                    </button>
                </div>
            </div>
        </div>
    );
}
