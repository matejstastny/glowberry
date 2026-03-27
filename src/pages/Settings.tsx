import styles from "./Settings.module.css";

export default function Settings() {
    return (
        <div className={styles.settings}>
            <h1>Settings</h1>

            <div className={styles.section}>
                <h2>General</h2>
                <div className={styles.row}>
                    <div className={styles.rowInfo}>
                        <div className={styles.label}>Default Memory</div>
                        <div className={styles.desc}>How much RAM to give Minecraft by default</div>
                    </div>
                    <select className={styles.control} defaultValue="4096">
                        <option value="2048">2 GB</option>
                        <option value="4096">4 GB</option>
                        <option value="6144">6 GB</option>
                        <option value="8192">8 GB</option>
                    </select>
                </div>
            </div>

            <div className={styles.section}>
                <h2>About</h2>
                <div className={styles.about}>
                    <div className={styles.aboutName}>Lantern</div>
                    <div className={styles.aboutVersion}>Version 0.1.0</div>
                    <div className={styles.aboutDesc}>
                        A simple Minecraft launcher built with care.
                    </div>
                </div>
            </div>
        </div>
    );
}
