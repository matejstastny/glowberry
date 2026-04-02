import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { GameExitEvent } from "../types";

interface GameStatus {
    running: string | null;
    crashLog: string | null;
}

export function useGameStatus(): GameStatus {
    const [running, setRunning] = useState<string | null>(null);
    const [crashLog, setCrashLog] = useState<string | null>(null);
    const unlistenRefs = useRef<(() => void)[]>([]);

    useEffect(() => {
        let cancelled = false;

        listen<{ instance_id: string }>("game-started", (event) => {
            if (!cancelled) {
                setRunning(event.payload.instance_id);
                setCrashLog(null);
            }
        }).then((unlisten) => {
            if (cancelled) unlisten();
            else unlistenRefs.current.push(unlisten);
        });

        listen<GameExitEvent>("game-exit", (event) => {
            if (!cancelled) {
                setRunning(null);
                const { exit_code, crash_log } = event.payload;
                if (exit_code !== null && exit_code !== 0) {
                    const header = `Game crashed with exit code ${exit_code}`;
                    setCrashLog(crash_log ? `${header}\n${crash_log}` : header);
                }
            }
        }).then((unlisten) => {
            if (cancelled) unlisten();
            else unlistenRefs.current.push(unlisten);
        });

        return () => {
            cancelled = true;
            unlistenRefs.current.forEach((fn) => fn());
        };
    }, []);

    return { running, crashLog };
}
