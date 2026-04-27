import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { tryRestoreSession } from "@/api/auth";
import { listInstances } from "@/api/instances";
import { installStarlight } from "@/api/install";
import { checkStarlightUpdate } from "@/api/github";
import { launchInstance } from "@/api/launch";
import { check as checkAppUpdate, type Update as AppUpdate } from "@tauri-apps/plugin-updater";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
    Instance,
    MinecraftProfile,
    InstallProgress,
    GameExitEvent,
    GithubRelease,
} from "@/types";
import SettingsPanel from "./SettingsPanel";
import styles from "./App.module.css";

const MODPACK_SLUG = "starlightmodpack";

type Phase = "loading" | "installing" | "updating" | "ready" | "error";

/** Tauri errors arrive as { kind, message } objects — extract a readable string. */
export function extractError(e: unknown): string {
    if (e && typeof e === "object") {
        const obj = e as Record<string, unknown>;
        if (typeof obj.message === "string") return obj.message;
    }
    return String(e);
}

function describeProgress(p: InstallProgress): string {
    switch (p.stage) {
        case "downloading": {
            if (p.bytes_total > 0) {
                const done = (p.bytes_downloaded / 1024 / 1024).toFixed(1);
                const total = (p.bytes_total / 1024 / 1024).toFixed(1);
                return `Downloading... ${done} / ${total} MB`;
            }
            return "Downloading...";
        }
        case "installing_mods":
            return p.total > 0 ? `Installing mods (${p.current}/${p.total})` : "Installing mods...";
        case "parsing":
            return "Reading modpack...";
        case "extracting_overrides":
            return "Extracting files...";
        case "installing_loader":
            return "Installing Fabric...";
        case "finalizing":
            return "Finishing up...";
        default:
            return p.message;
    }
}

export default function App() {
    const [phase, setPhase] = useState<Phase>("loading");
    const [instance, setInstance] = useState<Instance | null>(null);
    const [errorMsg, setErrorMsg] = useState<string | null>(null);

    const [profile, setProfile] = useState<MinecraftProfile | null>(null);
    const [appVersion, setAppVersion] = useState<string | null>(null);
    const [isOnline, setIsOnline] = useState(() => localStorage.getItem("gb_online") !== "offline");
    const [offlineUsername, setOfflineUsername] = useState(
        () => localStorage.getItem("gb_username") ?? "",
    );

    const [gameRunning, setGameRunning] = useState(false);
    const [preparing, setPreparing] = useState(false);
    const [gameError, setGameError] = useState<string | null>(null);

    const [progress, setProgress] = useState<InstallProgress | null>(null);
    // latest GitHub release — used for both first install and update
    const [latestRelease, setLatestRelease] = useState<GithubRelease | null>(null);
    const [updateAvailable, setUpdateAvailable] = useState(false);
    const [checkingUpdate, setCheckingUpdate] = useState(false);

    const [appUpdate, setAppUpdate] = useState<AppUpdate | null>(null);
    const [checkingAppUpdate, setCheckingAppUpdate] = useState(false);
    const [appUpdating, setAppUpdating] = useState(false);

    const [showSettings, setShowSettings] = useState(false);

    // Show the window once React has painted the first frame.
    // The window starts hidden (visible:false in tauri.conf.json) to avoid
    // the blank WebView2 flash on Windows before content is ready.
    useEffect(() => {
        const appWindow = getCurrentWindow();
        appWindow
            .show()
            .then(() => appWindow.setFocus())
            .catch(() => {});
    }, []);

    useEffect(() => {
        return () => {
            appUpdate?.close().catch(() => {});
        };
    }, [appUpdate]);

    // Restore auth + init on startup
    useEffect(() => {
        tryRestoreSession()
            .then(setProfile)
            .catch(() => {});
        loadInstance();
    }, []);

    useEffect(() => {
        getVersion()
            .then(setAppVersion)
            .catch(() => {});
    }, []);

    // Check whether Glowberry itself has a newer GitHub release.
    useEffect(() => {
        checkAppVersion();
    }, []);

    // Game events
    useEffect(() => {
        let cancelled = false;
        const unlisten: (() => void)[] = [];

        listen<{ instance_id: string }>("game-started", () => {
            if (!cancelled) {
                setGameRunning(true);
                setGameError(null);
            }
        }).then((u) => (cancelled ? u() : unlisten.push(u)));

        listen<GameExitEvent>("game-exit", (e) => {
            if (cancelled) return;
            setGameRunning(false);
            const { exit_code, crash_log } = e.payload;
            if (exit_code !== null && exit_code !== 0) {
                setGameError(crash_log ?? `Exited with code ${exit_code}`);
            }
        }).then((u) => (cancelled ? u() : unlisten.push(u)));

        return () => {
            cancelled = true;
            unlisten.forEach((f) => f());
        };
    }, []);

    // Install progress events
    useEffect(() => {
        let cancelled = false;
        let unsub: (() => void) | null = null;

        listen<InstallProgress>("install-progress", (e) => {
            if (!cancelled) setProgress(e.payload);
        }).then((u) => (cancelled ? u() : (unsub = u)));

        return () => {
            cancelled = true;
            unsub?.();
        };
    }, []);

    async function loadInstance() {
        try {
            const all = await listInstances();
            const found = all.find((i) => i.modpack?.project_slug === MODPACK_SLUG);

            if (found) {
                setInstance(found);
                setPhase("ready");
                checkUpdate(found);
            } else {
                // First run — fetch latest release then install
                setPhase("installing");
                const release = await checkStarlightUpdate();
                if (!release) {
                    setErrorMsg("No release available yet. Check back soon!");
                    setPhase("error");
                    return;
                }
                setLatestRelease(release);
                await doInstall(release);
            }
        } catch (e) {
            setErrorMsg(extractError(e));
            setPhase("error");
        }
    }

    async function checkUpdate(inst: Instance) {
        setCheckingUpdate(true);
        try {
            const release = await checkStarlightUpdate();
            setLatestRelease(release);
            const hasUpdate =
                !!release && !!inst.modpack && release.tag !== inst.modpack.version_id;
            setUpdateAvailable(hasUpdate);
        } catch {
            // silently ignore — network might be down
        } finally {
            setCheckingUpdate(false);
        }
    }

    async function checkAppVersion() {
        setCheckingAppUpdate(true);
        try {
            const update = await checkAppUpdate();
            appUpdate?.close().catch(() => {});
            setAppUpdate(update);
        } catch {
            // Ignore update check failures; network or signing may be unavailable.
        } finally {
            setCheckingAppUpdate(false);
        }
    }

    async function doInstall(release: GithubRelease) {
        setProgress(null);
        try {
            const installed = await installStarlight(release);
            setInstance(installed);
            setPhase("ready");
            setUpdateAvailable(false);
        } catch (e) {
            setErrorMsg(extractError(e));
            setPhase("error");
        }
    }

    async function handleUpdate() {
        setPhase("updating");
        setUpdateAvailable(false);
        try {
            const release = await checkStarlightUpdate();
            if (!release) {
                // No release available — go back to ready
                setPhase("ready");
                return;
            }
            setLatestRelease(release);
            await doInstall(release);
        } catch (e) {
            setErrorMsg(extractError(e));
            setPhase("error");
        }
    }

    async function handleAppUpdate() {
        if (appUpdating) return;

        if (!appUpdate) {
            await checkAppVersion();
            return;
        }

        setAppUpdating(true);
        try {
            await appUpdate.downloadAndInstall();
            setAppUpdate(null);

            const appWindow = getCurrentWindow();
            await appWindow.close().catch(() => {});
        } catch {
            // Keep the app usable even if the updater fails.
        } finally {
            setAppUpdating(false);
        }
    }

    async function handlePlay() {
        if (!instance || gameRunning || preparing) return;
        if (!isOnline && !offlineUsername.trim()) {
            setShowSettings(true);
            return;
        }
        setPreparing(true);
        setGameError(null);
        try {
            await launchInstance(
                instance.id,
                isOnline,
                isOnline ? undefined : offlineUsername.trim(),
            );
        } catch (e) {
            setGameError(extractError(e));
        } finally {
            setPreparing(false);
        }
    }

    const busy = gameRunning || preparing || appUpdating;
    const iconUrl = instance?.modpack?.icon_url;
    const versionParts = [
        instance?.modpack?.version_id,
        instance?.minecraft_version,
        instance?.loader && instance.loader !== "vanilla" ? instance.loader : null,
    ].filter(Boolean);

    return (
        <div className={styles.app}>
            {/* Persistent top bar — always rendered above main content and the
                settings overlay.  The drag region fills the left portion;
                the settings button sits on the right as a normal flex child.
                On macOS (titleBarStyle Overlay) this strip aligns with the
                traffic-light buttons so they never clash with any content. */}
            <div className={styles.topBar}>
                <div className={styles.topBarDrag} data-tauri-drag-region />
                <button
                    className={styles.settingsBtn}
                    onClick={() => setShowSettings((v) => !v)}
                    title={showSettings ? "Home" : "Settings"}
                >
                    {showSettings ? <HomeIcon /> : <SettingsIcon />}
                </button>
            </div>

            {/* Main content */}
            <div className={styles.main}>
                {phase === "loading" && (
                    <div className={styles.center}>
                        <Spinner size={28} />
                    </div>
                )}

                {phase === "error" && (
                    <div className={styles.center}>
                        <div className={styles.errorBox}>
                            <div className={styles.errorTitle}>Something went wrong</div>
                            <div className={styles.errorMsg}>{errorMsg}</div>
                            <button className={styles.retryBtn} onClick={loadInstance}>
                                Try again
                            </button>
                        </div>
                    </div>
                )}

                {(phase === "installing" || phase === "updating") && (
                    <div className={styles.center}>
                        <div className={styles.installBox}>
                            <Spinner size={32} />
                            <div className={styles.installTitle}>
                                {phase === "updating"
                                    ? "Updating Starlight..."
                                    : "Installing Starlight..."}
                            </div>
                            {progress && (
                                <>
                                    <div className={styles.installMsg}>
                                        {describeProgress(progress)}
                                    </div>
                                    {progress.bytes_total > 0 && (
                                        <div className={styles.progressBar}>
                                            <div
                                                className={styles.progressFill}
                                                style={{
                                                    width: `${Math.round((progress.bytes_downloaded / progress.bytes_total) * 100)}%`,
                                                }}
                                            />
                                        </div>
                                    )}
                                </>
                            )}
                        </div>
                    </div>
                )}

                {phase === "ready" && (
                    <div className={styles.launcher}>
                        {/* Modpack icon */}
                        <div className={styles.iconWrap}>
                            {iconUrl ? (
                                <img className={styles.icon} src={iconUrl} alt="Starlight" />
                            ) : (
                                <div className={styles.iconPlaceholder}>
                                    <GlowberryIcon />
                                </div>
                            )}
                        </div>

                        <div className={styles.packName}>Starlight</div>
                        {versionParts.length > 0 && (
                            <div className={styles.packVersion}>{versionParts.join(" · ")}</div>
                        )}

                        {/* Play button */}
                        <button
                            className={`${styles.playBtn} ${busy ? styles.playBusy : ""}`}
                            onClick={handlePlay}
                            disabled={busy}
                        >
                            {busy ? <Spinner size={15} /> : <PlayIcon />}
                            {preparing ? "Preparing..." : gameRunning ? "Running" : "Play"}
                        </button>

                        {/* Online / Offline toggle */}
                        <div className={styles.modeRow}>
                            <button
                                className={`${styles.modeBtn} ${isOnline ? styles.modeBtnActive : ""}`}
                                onClick={() => {
                                    setIsOnline(true);
                                    localStorage.setItem("gb_online", "online");
                                }}
                            >
                                Online
                            </button>
                            <button
                                className={`${styles.modeBtn} ${!isOnline ? styles.modeBtnActive : ""}`}
                                onClick={() => {
                                    setIsOnline(false);
                                    localStorage.setItem("gb_online", "offline");
                                }}
                            >
                                Offline
                            </button>
                        </div>

                        {/* Update button */}
                        <button
                            className={`${styles.updateBtn} ${updateAvailable ? styles.updateAvailable : ""}`}
                            onClick={handleUpdate}
                            disabled={busy || checkingUpdate}
                            title={
                                checkingUpdate
                                    ? "Checking..."
                                    : updateAvailable
                                      ? `Update to ${latestRelease?.tag}`
                                      : "Up to date"
                            }
                        >
                            {checkingUpdate ? <Spinner size={12} /> : <UpdateIcon />}
                            {checkingUpdate
                                ? "Checking..."
                                : updateAvailable
                                  ? `Update to ${latestRelease?.tag}`
                                  : "Up to date"}
                        </button>

                        <button
                            className={`${styles.updateBtn} ${styles.appUpdateBtn} ${appUpdate ? styles.updateAvailable : ""}`}
                            onClick={handleAppUpdate}
                            disabled={busy || checkingAppUpdate}
                            title={
                                checkingAppUpdate
                                    ? "Checking for app updates..."
                                    : appUpdating
                                      ? "Installing update..."
                                      : appUpdate
                                        ? `Update Glowberry to ${appUpdate.version}`
                                        : "Check for app updates"
                            }
                        >
                            {checkingAppUpdate || appUpdating ? (
                                <Spinner size={12} />
                            ) : (
                                <UpdateIcon />
                            )}
                            {checkingAppUpdate
                                ? "Checking for app updates..."
                                : appUpdating
                                  ? "Installing app update..."
                                  : appUpdate
                                    ? `Update Glowberry to ${appUpdate.version}`
                                    : "Check for app updates"}
                        </button>

                        {/* Game error */}
                        <div className={styles.gameErrorSlot}>
                            {gameError && (
                                <div className={styles.gameError}>
                                    <span className={styles.gameErrorText}>
                                        {gameError.split("\n")[0]}
                                    </span>
                                    <button
                                        className={styles.dismissBtn}
                                        onClick={() => setGameError(null)}
                                    >
                                        ×
                                    </button>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </div>

            {/* Bottom status bar */}
            <div className={styles.bottomBar}>
                {profile ? (
                    <>
                        <img
                            className={styles.avatar}
                            src={`https://mc-heads.net/avatar/${profile.id}/20`}
                            alt={profile.name}
                        />
                        <span className={styles.accountName}>{profile.name}</span>
                    </>
                ) : (
                    <span className={styles.accountName}>
                        {isOnline ? "Not signed in" : offlineUsername || "No username set"}
                    </span>
                )}
            </div>

            {/* Settings overlay */}
            {showSettings && (
                <SettingsPanel
                    profile={profile}
                    appVersion={appVersion}
                    isOnline={isOnline}
                    offlineUsername={offlineUsername}
                    instance={instance}
                    onProfileChange={setProfile}
                    onOnlineChange={(v) => {
                        setIsOnline(v);
                        localStorage.setItem("gb_online", v ? "online" : "offline");
                    }}
                    onUsernameChange={(u) => {
                        setOfflineUsername(u);
                        localStorage.setItem("gb_username", u);
                    }}
                    onInstanceChange={setInstance}
                />
            )}
        </div>
    );
}

// ── Inline icons ──────────────────────────────────────────────────────────────

function PlayIcon() {
    return (
        <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 4l15 8-15 8V4z" />
        </svg>
    );
}

function UpdateIcon() {
    return (
        <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
        >
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
        </svg>
    );
}

function HomeIcon() {
    return (
        <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
        >
            <path d="M3 9.5L12 3l9 6.5V20a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V9.5z" />
            <path d="M9 21V12h6v9" />
        </svg>
    );
}

function SettingsIcon() {
    return (
        <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
        >
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06-.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
    );
}

export function Spinner({ size = 20 }: { size?: number }) {
    return (
        <svg
            width={size}
            height={size}
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            style={{ animation: "spin 0.8s linear infinite" }}
        >
            <path d="M12 2a10 10 0 0 1 10 10" />
        </svg>
    );
}

function GlowberryIcon() {
    return (
        <svg width="56" height="56" viewBox="0 0 72 72" fill="none">
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
            <ellipse
                cx="24"
                cy="20"
                rx="6"
                ry="3.5"
                transform="rotate(-30 24 20)"
                fill="#4a7a4a"
                opacity="0.7"
            />
            <ellipse
                cx="48"
                cy="18"
                rx="6"
                ry="3.5"
                transform="rotate(25 48 18)"
                fill="#4a7a4a"
                opacity="0.7"
            />
            <circle cx="36" cy="46" r="16" fill="#d4a24c" opacity="0.08" />
            <circle cx="36" cy="46" r="13" fill="#c49238" />
            <circle cx="36" cy="46" r="13" fill="url(#gbg)" />
            <ellipse cx="32" cy="41" rx="4" ry="3" fill="#e8c06a" opacity="0.5" />
            <path
                d="M30 35 C30 35 33 37 36 37 C39 37 42 35 42 35 C42 35 40 33 36 33 C32 33 30 35 30 35Z"
                fill="#5a8a5a"
            />
            <defs>
                <radialGradient id="gbg" cx="0.4" cy="0.35" r="0.6">
                    <stop offset="0%" stopColor="#e8c06a" />
                    <stop offset="100%" stopColor="#b07828" />
                </radialGradient>
            </defs>
        </svg>
    );
}
