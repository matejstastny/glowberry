import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { InstallProgress } from "../types";

export function useInstallProgress() {
    const [progress, setProgress] = useState<Record<string, InstallProgress>>({});
    const unlistenRef = useRef<(() => void) | null>(null);

    useEffect(() => {
        let cancelled = false;

        listen<InstallProgress>("install-progress", (event) => {
            if (cancelled) return;
            setProgress((prev) => ({
                ...prev,
                [event.payload.project_id]: event.payload,
            }));
        }).then((unlisten) => {
            if (cancelled) {
                unlisten();
            } else {
                unlistenRef.current = unlisten;
            }
        });

        return () => {
            cancelled = true;
            unlistenRef.current?.();
        };
    }, []);

    return progress;
}
