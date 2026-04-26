import { useState, useEffect, useRef } from "react";
import QRCode from "qrcode";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { startLogin, cancelLogin, logout } from "@/api/auth";
import { getSettings, setDataDir } from "@/api/settings";
import { setInstanceMemory } from "@/api/instances";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Instance, MinecraftProfile } from "@/types";
import { Spinner, extractError } from "./App";
import styles from "./SettingsPanel.module.css";

const MEMORY_OPTIONS = [
    { label: "2 GB", value: 2048 },
    { label: "4 GB", value: 4096 },
    { label: "6 GB", value: 6144 },
    { label: "8 GB", value: 8192 },
    { label: "12 GB", value: 12288 },
];

interface Props {
    profile: MinecraftProfile | null;
    isOnline: boolean;
    offlineUsername: string;
    instance: Instance | null;
    onProfileChange: (p: MinecraftProfile | null) => void;
    onOnlineChange: (online: boolean) => void;
    onUsernameChange: (username: string) => void;
    onInstanceChange: (i: Instance) => void;
}

type LoginState =
    | { step: "idle" }
    | { step: "waiting"; authUrl: string; qrDataUrl: string }
    | { step: "error"; message: string };

export default function SettingsPanel({
    profile,
    isOnline,
    offlineUsername,
    instance,
    onProfileChange,
    onOnlineChange,
    onUsernameChange,
    onInstanceChange,
}: Props) {
    const [dataDir, setDataDirState] = useState("");
    const [defaultDataDir, setDefaultDataDir] = useState("");
    const [hasCustomDir, setHasCustomDir] = useState(false);
    const [loginState, setLoginState] = useState<LoginState>({ step: "idle" });
    const [localUsername, setLocalUsername] = useState(offlineUsername);

    const unlistenRef = useRef<UnlistenFn[]>([]);

    useEffect(() => {
        getSettings()
            .then((s) => {
                setDataDirState(s.data_dir);
                setDefaultDataDir(s.default_data_dir);
                setHasCustomDir(!!s.data_dir_override);
            })
            .catch(() => {});
    }, []);

    // Clean up event listeners on unmount
    useEffect(() => {
        return () => stopListening();
    }, []);

    function stopListening() {
        for (const fn of unlistenRef.current) fn();
        unlistenRef.current = [];
    }

    async function handleSignIn() {
        setLoginState({ step: "idle" }); // reset any previous error
        try {
            const authUrl = await startLogin();

            // Generate QR code from the auth URL
            const qrDataUrl = await QRCode.toDataURL(authUrl, {
                width: 180,
                margin: 2,
                color: { dark: "#e2e0dc", light: "#0e0f14" },
            });

            setLoginState({ step: "waiting", authUrl, qrDataUrl });

            // Listen for auth events emitted by the background task
            const unlistenComplete = await listen<{ profile: MinecraftProfile }>(
                "auth-complete",
                (event) => {
                    stopListening();
                    onProfileChange(event.payload.profile);
                    onOnlineChange(true);
                    setLoginState({ step: "idle" });
                },
            );

            const unlistenError = await listen<{ message: string }>(
                "auth-error",
                (event) => {
                    stopListening();
                    setLoginState({ step: "error", message: event.payload.message });
                },
            );

            unlistenRef.current = [unlistenComplete, unlistenError];
        } catch (e) {
            setLoginState({ step: "error", message: extractError(e) });
        }
    }

    async function handleCancelLogin() {
        stopListening();
        await cancelLogin().catch(() => {});
        setLoginState({ step: "idle" });
    }

    async function handleSignOut() {
        await logout();
        onProfileChange(null);
    }

    async function handlePickDir() {
        const dir = await openDialog({ directory: true, title: "Choose data folder" });
        if (dir) {
            await setDataDir(dir);
            setDataDirState(dir);
            setHasCustomDir(true);
        }
    }

    async function handleResetDir() {
        await setDataDir(null);
        setDataDirState(defaultDataDir);
        setHasCustomDir(false);
    }

    async function handleMemoryChange(mb: number) {
        if (!instance) return;
        await setInstanceMemory(instance.id, mb);
        onInstanceChange({ ...instance, memory_mb: mb });
    }

    function handleUsernameBlur() {
        onUsernameChange(localUsername.trim());
    }

    return (
        <div className={styles.overlay}>
            <div className={styles.body}>
                {/* ── Account ─────────────────────────────────── */}
                <section className={styles.section}>
                    <div className={styles.label}>Microsoft Account</div>
                    <div className={styles.card}>
                        {profile ? (
                            <div className={styles.accountRow}>
                                <img
                                    className={styles.avatar}
                                    src={`https://mc-heads.net/avatar/${profile.id}/28`}
                                    alt={profile.name}
                                />
                                <span className={styles.accountName}>{profile.name}</span>
                                <button className={styles.smallBtn} onClick={handleSignOut}>
                                    Sign out
                                </button>
                            </div>
                        ) : loginState.step === "waiting" ? (
                            <div className={styles.loginFlow}>
                                <img
                                    className={styles.qrCode}
                                    src={loginState.qrDataUrl}
                                    alt="QR code"
                                />
                                <div className={styles.loginHint}>
                                    Scan the QR code or{" "}
                                    <button
                                        className={styles.linkBtn}
                                        onClick={() => openUrl(loginState.authUrl)}
                                    >
                                        open in browser
                                    </button>{" "}
                                    to sign in
                                </div>
                                <div className={styles.loginWaiting}>
                                    <Spinner size={13} />
                                    <span>Waiting for sign-in...</span>
                                </div>
                                <button
                                    className={styles.cancelBtn}
                                    onClick={handleCancelLogin}
                                >
                                    Cancel
                                </button>
                            </div>
                        ) : (
                            <>
                                <button
                                    className={styles.signInBtn}
                                    onClick={handleSignIn}
                                >
                                    <MicrosoftIcon />
                                    Sign in with Microsoft
                                </button>
                                {loginState.step === "error" && (
                                    <div className={styles.loginError}>
                                        {loginState.message}
                                    </div>
                                )}
                            </>
                        )}
                    </div>
                </section>

                {/* ── Offline username (only shown in offline mode) ── */}
                {!isOnline && (
                    <section className={styles.section}>
                        <div className={styles.label}>Offline Username</div>
                        <div className={styles.card}>
                            <input
                                className={styles.usernameInput}
                                type="text"
                                placeholder="Username"
                                value={localUsername}
                                onChange={(e) =>
                                    setLocalUsername(e.target.value.replace(/\s/g, ""))
                                }
                                onBlur={handleUsernameBlur}
                                maxLength={16}
                                autoCorrect="off"
                                autoComplete="off"
                                autoCapitalize="off"
                                spellCheck={false}
                            />
                        </div>
                    </section>
                )}

                {/* ── Memory ──────────────────────────────────── */}
                {instance && (
                    <section className={styles.section}>
                        <div className={styles.label}>Memory</div>
                        <div className={styles.card}>
                            <select
                                className={styles.select}
                                value={instance.memory_mb}
                                onChange={(e) => handleMemoryChange(Number(e.target.value))}
                            >
                                {MEMORY_OPTIONS.map((o) => (
                                    <option key={o.value} value={o.value}>
                                        {o.label}
                                    </option>
                                ))}
                            </select>
                        </div>
                    </section>
                )}

                {/* ── Data directory ──────────────────────────── */}
                <section className={styles.section}>
                    <div className={styles.label}>Data Directory</div>
                    <div className={styles.card}>
                        <code className={styles.dirPath}>{dataDir}</code>
                        <div className={styles.dirActions}>
                            <button
                                className={styles.smallBtn}
                                onClick={() => openUrl(`file://${dataDir}`)}
                            >
                                Open
                            </button>
                            <button className={styles.smallBtn} onClick={handlePickDir}>
                                Change
                            </button>
                            {hasCustomDir && (
                                <button className={styles.smallBtn} onClick={handleResetDir}>
                                    Reset
                                </button>
                            )}
                        </div>
                        {hasCustomDir && (
                            <div className={styles.dirNote}>
                                Restart Glowberry for changes to take effect
                            </div>
                        )}
                    </div>
                </section>
            </div>
        </div>
    );
}

// ── Inline icons ──────────────────────────────────────────────────────────────

function MicrosoftIcon() {
    return (
        <svg width="15" height="15" viewBox="0 0 21 21" fill="none">
            <rect x="1" y="1" width="9" height="9" fill="#F25022" />
            <rect x="11" y="1" width="9" height="9" fill="#7FBA00" />
            <rect x="1" y="11" width="9" height="9" fill="#00A4EF" />
            <rect x="11" y="11" width="9" height="9" fill="#FFB900" />
        </svg>
    );
}
