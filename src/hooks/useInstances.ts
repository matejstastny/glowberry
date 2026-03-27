import { useCallback, useState } from "react";
import { listInstances } from "../api/instances";
import type { Instance } from "../types";

export function useInstances() {
    const [instances, setInstances] = useState<Instance[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const refresh = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await listInstances();
            setInstances(data);
        } catch (e) {
            setError(e instanceof Error ? e.message : String(e));
        } finally {
            setLoading(false);
        }
    }, []);

    return { instances, loading, error, refresh };
}
