import { useState } from "react";
import NavBar from "./components/NavBar";
import OfflinePopup from "./components/OfflinePopup";
import Home from "./pages/Home";
import Browse from "./pages/Browse";
import Settings from "./pages/Settings";
import type { Page } from "./types";

export default function App() {
    const [page, setPage] = useState<Page>({ kind: "home" });
    const [isOnline, setIsOnline] = useState(true);
    const [offlineUsername, setOfflineUsername] = useState<string | null>(
        localStorage.getItem("lantern_offline_username"),
    );
    const [showOfflinePopup, setShowOfflinePopup] = useState(false);
    const [pendingLaunchId, setPendingLaunchId] = useState<string | null>(null);

    function handleToggleOnline() {
        setIsOnline((prev) => !prev);
    }

    function handlePlay(instanceId: string) {
        if (!isOnline && !offlineUsername) {
            setPendingLaunchId(instanceId);
            setShowOfflinePopup(true);
            return;
        }
        // TODO: invoke Tauri launch command
        console.log("Launch:", instanceId, isOnline ? "online" : "offline");
    }

    function handleOfflineSubmit(username: string) {
        localStorage.setItem("lantern_offline_username", username);
        setOfflineUsername(username);
        setShowOfflinePopup(false);
        if (pendingLaunchId) {
            console.log("Launch:", pendingLaunchId, "offline as", username);
            setPendingLaunchId(null);
        }
    }

    return (
        <>
            <NavBar
                page={page}
                navigate={setPage}
                isOnline={isOnline}
                onToggleOnline={handleToggleOnline}
            />
            <main className="main-content">
                {page.kind === "home" && <Home onPlay={handlePlay} />}
                {page.kind === "browse" && <Browse navigate={setPage} />}
                {page.kind === "settings" && <Settings navigate={setPage} />}
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
