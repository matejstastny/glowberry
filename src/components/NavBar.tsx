import type { Page } from "../types";
import { SettingsIcon, RefreshIcon, PlusIcon, UserIcon } from "./Icons";
import styles from "./NavBar.module.css";

interface NavBarProps {
    page: Page;
    navigate: (page: Page) => void;
    isOnline: boolean;
    onToggleOnline: () => void;
    appUpdateAvailable?: boolean;
    appUpdating?: boolean;
}

export default function NavBar({
    page,
    navigate,
    isOnline,
    onToggleOnline,
    appUpdateAvailable = false,
    appUpdating = false,
}: NavBarProps) {
    function handleSettingsClick() {
        navigate(page.kind === "settings" ? { kind: "home" } : { kind: "settings" });
    }

    function handleAddPackClick() {
        navigate(page.kind === "browse" ? { kind: "home" } : { kind: "browse" });
    }

    return (
        <nav className={styles.nav} data-tauri-drag-region>
            <div className={styles.left}>
                <button className={styles.navBtn} title="Account">
                    <div className={styles.avatar}>
                        <UserIcon size={14} />
                    </div>
                    <span>Player</span>
                </button>

                <div className={styles.separator} />

                <button
                    className={`${styles.navBtn} ${appUpdateAvailable ? styles.highlight : ""}`}
                    title="Check for updates"
                    disabled={appUpdating}
                >
                    {appUpdating ? <RefreshIcon size={15} /> : <RefreshIcon size={15} />}
                    <span>Update</span>
                </button>
                <button
                    className={`${styles.navBtn} ${page.kind === "settings" ? styles.active : ""}`}
                    onClick={handleSettingsClick}
                >
                    <SettingsIcon size={15} />
                    <span>Settings</span>
                </button>
            </div>

            <div className={styles.right}>
                <button className={styles.toggle} onClick={onToggleOnline}>
                    <div className={`${styles.toggleTrack} ${isOnline ? styles.toggleOnline : ""}`}>
                        <div className={styles.toggleThumb} />
                    </div>
                    <span
                        className={`${styles.toggleLabel} ${isOnline ? styles.labelOnline : styles.labelOffline}`}
                    >
                        {isOnline ? "Online" : "Offline"}
                    </span>
                </button>

                <button
                    className={`${styles.addPackBtn} ${page.kind === "browse" ? styles.addPackActive : ""}`}
                    onClick={handleAddPackClick}
                >
                    <PlusIcon size={14} />
                    <span>Add Pack</span>
                </button>
            </div>
        </nav>
    );
}
