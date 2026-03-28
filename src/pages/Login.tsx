import { useState, useEffect, useRef } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowLeftIcon, SpinnerIcon, CheckIcon } from "@/components/Icons";
import * as authApi from "@/api/auth";
import type { Page, MinecraftProfile, DeviceCodeInfo } from "@/types";
import styles from "./Login.module.css";

interface LoginProps {
    navigate: (page: Page) => void;
    onLoginComplete: (profile: MinecraftProfile) => void;
}

type LoginState =
    | { step: "idle" }
    | { step: "loading" }
    | { step: "code"; info: DeviceCodeInfo }
    | { step: "done"; profile: MinecraftProfile }
    | { step: "error"; message: string };

export default function Login({ navigate, onLoginComplete }: LoginProps) {
    const [state, setState] = useState<LoginState>({ step: "idle" });
    const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

    useEffect(() => {
        return () => {
            if (pollRef.current) clearInterval(pollRef.current);
        };
    }, []);

    async function handleSignIn() {
        setState({ step: "loading" });
        try {
            const info = await authApi.startLogin();
            setState({ step: "code", info });

            // Start polling for completion
            pollRef.current = setInterval(async () => {
                try {
                    const result = await authApi.checkLoginStatus();
                    if (result.status === "complete") {
                        if (pollRef.current) clearInterval(pollRef.current);
                        pollRef.current = null;
                        setState({ step: "done", profile: result.profile });
                        setTimeout(() => onLoginComplete(result.profile), 1200);
                    }
                } catch (err) {
                    console.error("check_login_status failed:", err);
                    if (pollRef.current) clearInterval(pollRef.current);
                    pollRef.current = null;
                    const msg =
                        err && typeof err === "object" && "message" in err
                            ? (err as { message: string }).message
                            : String(err);
                    setState({ step: "error", message: msg });
                }
            }, 5000);
        } catch (err) {
            console.error("start_login failed:", err);
            const msg =
                err && typeof err === "object" && "message" in err
                    ? (err as { message: string }).message
                    : String(err);
            setState({ step: "error", message: msg });
        }
    }

    function handleCopyCode(code: string) {
        navigator.clipboard.writeText(code);
    }

    return (
        <div className={styles.login}>
            <button className={styles.back} onClick={() => navigate({ kind: "home" })}>
                <ArrowLeftIcon size={16} />
                <span>Back</span>
            </button>

            <div className={styles.container}>
                {state.step === "idle" && (
                    <div className={styles.welcome}>
                        <div className={styles.icon}>
                            <MicrosoftIcon />
                        </div>
                        <h1 className={styles.title}>Sign in with Microsoft</h1>
                        <p className={styles.subtitle}>
                            Connect your Microsoft account to play online
                        </p>
                        <button className={styles.signInBtn} onClick={handleSignIn}>
                            Sign in
                        </button>
                    </div>
                )}

                {state.step === "loading" && (
                    <div className={styles.center}>
                        <SpinnerIcon size={24} />
                    </div>
                )}

                {state.step === "code" && (
                    <div className={styles.codeFlow}>
                        <h2 className={styles.codeTitle}>Enter this code</h2>
                        <button
                            className={styles.code}
                            onClick={() => handleCopyCode(state.info.user_code)}
                            title="Click to copy"
                        >
                            {state.info.user_code}
                        </button>
                        <p className={styles.codeHint}>Click the code to copy it</p>

                        <p className={styles.instructions}>
                            Then open the link below and paste the code to sign in
                        </p>

                        <button
                            className={styles.linkBtn}
                            onClick={() => openUrl(state.info.verification_uri)}
                        >
                            Open {state.info.verification_uri}
                        </button>

                        <div className={styles.waiting}>
                            <SpinnerIcon size={14} />
                            <span>Waiting for you to sign in...</span>
                        </div>
                    </div>
                )}

                {state.step === "done" && (
                    <div className={styles.success}>
                        <div className={styles.checkCircle}>
                            <CheckIcon size={20} />
                        </div>
                        <h2 className={styles.successTitle}>Welcome, {state.profile.name}!</h2>
                        <p className={styles.subtitle}>You're all set to play online</p>
                    </div>
                )}

                {state.step === "error" && (
                    <div className={styles.welcome}>
                        <div className={styles.errorIcon}>!</div>
                        <h2 className={styles.errorTitle}>Something went wrong</h2>
                        <p className={styles.subtitle}>{state.message}</p>
                        <button className={styles.signInBtn} onClick={handleSignIn}>
                            Try again
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}

function MicrosoftIcon() {
    return (
        <svg width="20" height="20" viewBox="0 0 21 21" fill="none">
            <rect x="1" y="1" width="9" height="9" fill="#F25022" />
            <rect x="11" y="1" width="9" height="9" fill="#7FBA00" />
            <rect x="1" y="11" width="9" height="9" fill="#00A4EF" />
            <rect x="11" y="11" width="9" height="9" fill="#FFB900" />
        </svg>
    );
}
