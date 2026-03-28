import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { GameExitEvent } from "../types";

export function useGameStatus() {
    const [running, setRunning] = useState<string | null>(null);
    const unlistenRefs = useRef<(() => void)[]>([]);

    useEffect(() => {
        let cancelled = false;

        listen<{ instance_id: string }>("game-started", (event) => {
            if (!cancelled) setRunning(event.payload.instance_id);
        }).then((unlisten) => {
            if (cancelled) unlisten();
            else unlistenRefs.current.push(unlisten);
        });

        listen<GameExitEvent>("game-exit", () => {
            if (!cancelled) setRunning(null);
        }).then((unlisten) => {
            if (cancelled) unlisten();
            else unlistenRefs.current.push(unlisten);
        });

        return () => {
            cancelled = true;
            unlistenRefs.current.forEach((fn) => fn());
        };
    }, []);

    return running;
}
