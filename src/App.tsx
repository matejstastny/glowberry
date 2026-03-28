import { useState } from "react";
import NavBar from "@/components/NavBar";
import OfflinePopup from "@/components/OfflinePopup";
import Home from "@/pages/Home";
import Browse from "@/pages/Browse";
import Settings from "@/pages/Settings";
import Login from "@/pages/Login";
import { useAuth } from "@/hooks/useAuth";
import { useGameStatus } from "@/hooks/useGameStatus";
import { launchInstance } from "@/api/launch";
import type { Page } from "@/types";

export default function App() {
    const [page, setPage] = useState<Page>({ kind: "home" });
    const [isOnline, setIsOnline] = useState(
        () => localStorage.getItem("lantern_online_mode") !== "offline",
    );
    const [offlineUsername, setOfflineUsername] = useState<string | null>(
        localStorage.getItem("lantern_offline_username"),
    );
    const [showOfflinePopup, setShowOfflinePopup] = useState(false);
    const [pendingLaunchId, setPendingLaunchId] = useState<string | null>(null);
    const [launchError, setLaunchError] = useState<string | null>(null);
    const [refreshKey, setRefreshKey] = useState(0);

    const { profile, setProfile, handleLogout } = useAuth();
    const runningInstance = useGameStatus();

    function handleToggleOnline() {
        setIsOnline((prev) => {
            const next = !prev;
            localStorage.setItem("lantern_online_mode", next ? "online" : "offline");
            return next;
        });
    }

    async function doLaunch(instanceId: string, username?: string) {
        setLaunchError(null);
        try {
            await launchInstance(instanceId, isOnline, username ?? undefined);
        } catch (e: unknown) {
            const msg = e instanceof Error ? e.message : String(e);
            console.error("Launch failed:", msg);
            setLaunchError(msg);
        }
    }

    function handlePlay(instanceId: string) {
        if (runningInstance) return; // already running

        if (!isOnline && !offlineUsername) {
            setPendingLaunchId(instanceId);
            setShowOfflinePopup(true);
            return;
        }

        doLaunch(instanceId, isOnline ? undefined : offlineUsername ?? undefined);
    }

    function handleOfflineSubmit(username: string) {
        localStorage.setItem("lantern_offline_username", username);
        setOfflineUsername(username);
        setShowOfflinePopup(false);
        if (pendingLaunchId) {
            doLaunch(pendingLaunchId, username);
            setPendingLaunchId(null);
        }
    }

    function handleLogoutAndNavigate() {
        handleLogout();
        if (page.kind === "login") {
            setPage({ kind: "home" });
        }
    }

    return (
        <>
            <NavBar
                page={page}
                navigate={setPage}
                isOnline={isOnline}
                onToggleOnline={handleToggleOnline}
                profile={profile}
                onLogout={handleLogoutAndNavigate}
            />
            <main className="main-content">
                {page.kind === "home" && (
                    <Home
                        onPlay={handlePlay}
                        runningInstance={runningInstance}
                        launchError={launchError}
                        refreshKey={refreshKey}
                    />
                )}
                {page.kind === "browse" && (
                    <Browse
                        navigate={setPage}
                        onInstalled={() => setRefreshKey((k) => k + 1)}
                    />
                )}
                {page.kind === "settings" && <Settings navigate={setPage} />}
                {page.kind === "login" && (
                    <Login
                        navigate={setPage}
                        onLoginComplete={(p) => {
                            setProfile(p);
                            setPage({ kind: "home" });
                        }}
                    />
                )}
            </main>
            {showOfflinePopup && (
                <OfflinePopup
                    onSubmit={handleOfflineSubmit}
                    onCancel={() => setShowOfflinePopup(false)}
                />
            )}
        </>
    );
}
