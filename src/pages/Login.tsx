import { useState } from "react";
import { ArrowLeftIcon, SpinnerIcon, CheckIcon } from "@/components/Icons";
import * as authApi from "@/api/auth";
import type { Page, MinecraftProfile } from "@/types";
import styles from "./Login.module.css";

interface LoginProps {
    navigate: (page: Page) => void;
    onLoginComplete: (profile: MinecraftProfile) => void;
}

type LoginState =
    | { step: "idle" }
    | { step: "loading" }
    | { step: "done"; profile: MinecraftProfile }
    | { step: "error"; message: string };

export default function Login({ navigate, onLoginComplete }: LoginProps) {
    const [state, setState] = useState<LoginState>({ step: "idle" });

    async function handleSignIn() {
        setState({ step: "loading" });
        try {
            const profile = await authApi.startLogin();
            setState({ step: "done", profile });
            setTimeout(() => onLoginComplete(profile), 1200);
        } catch (err) {
            console.error("login failed:", err);
            const msg =
                err && typeof err === "object" && "message" in err
                    ? (err as { message: string }).message
                    : String(err);
            setState({ step: "error", message: msg });
        }
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
                    <div className={styles.welcome}>
                        <div className={styles.icon}>
                            <MicrosoftIcon />
                        </div>
                        <h2 className={styles.title}>Waiting for sign in...</h2>
                        <p className={styles.subtitle}>
                            Complete the login in the window that just opened
                        </p>
                        <SpinnerIcon size={20} />
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
