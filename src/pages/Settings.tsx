import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowLeftIcon, GithubIcon } from "../components/Icons";
import type { Page } from "../types";
import styles from "./Settings.module.css";

interface SettingsProps {
    navigate: (page: Page) => void;
}

export default function Settings({ navigate }: SettingsProps) {
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
