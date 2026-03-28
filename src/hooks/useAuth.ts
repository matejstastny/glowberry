import { useState, useEffect, useCallback } from "react";
import * as authApi from "@/api/auth";
import type { MinecraftProfile } from "@/types";

interface UseAuth {
    profile: MinecraftProfile | null;
    loading: boolean;
    setProfile: (profile: MinecraftProfile | null) => void;
    handleLogout: () => Promise<void>;
}

export function useAuth(): UseAuth {
    const [profile, setProfile] = useState<MinecraftProfile | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        authApi
            .tryRestoreSession()
            .then((p) => setProfile(p))
            .catch(() => {})
            .finally(() => setLoading(false));
    }, []);

    const handleLogout = useCallback(async () => {
        await authApi.logout();
        setProfile(null);
    }, []);

    return { profile, loading, setProfile, handleLogout };
}
