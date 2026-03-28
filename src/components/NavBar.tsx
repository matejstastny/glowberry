import type { Page, MinecraftProfile } from "@/types";
import { SettingsIcon, RefreshIcon, PlusIcon, UserIcon } from "./Icons";
import styles from "./NavBar.module.css";

interface NavBarProps {
    page: Page;
    navigate: (page: Page) => void;
    isOnline: boolean;
    onToggleOnline: () => void;
    profile: MinecraftProfile | null;
    onLogout: () => void;
    appUpdateAvailable?: boolean;
    appUpdating?: boolean;
}

export default function NavBar({
    page,
    navigate,
    isOnline,
    onToggleOnline,
    profile,
    onLogout,
    appUpdateAvailable = false,
    appUpdating = false,
}: NavBarProps) {
    function handleSettingsClick() {
        navigate(page.kind === "settings" ? { kind: "home" } : { kind: "settings" });
    }

    function handleAddPackClick() {
        navigate(page.kind === "browse" ? { kind: "home" } : { kind: "browse" });
    }

    function handleAccountClick() {
        if (profile) {
            onLogout();
        } else {
            navigate({ kind: "login" });
        }
    }

    const avatarUrl = profile
        ? `https://mc-heads.net/avatar/${profile.id}/24`
        : null;

    return (
        <nav className={styles.nav} data-tauri-drag-region>
            <div className={styles.left}>
                <button
                    className={`${styles.navBtn} ${page.kind === "login" ? styles.active : ""}`}
                    title={profile ? `Signed in as ${profile.name} — click to sign out` : "Sign in"}
                    onClick={handleAccountClick}
                >
                    <div className={styles.avatar}>
                        {avatarUrl ? (
                            <img src={avatarUrl} alt={profile!.name} />
                        ) : (
                            <UserIcon size={14} />
                        )}
                    </div>
                    <span>{profile ? profile.name : "Sign in"}</span>
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
