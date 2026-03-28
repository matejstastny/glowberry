import { useCallback, useRef, useState } from "react";
import { searchModpacks } from "../api/modpacks";
import {
    SearchIcon,
    DownloadIcon,
    SpinnerIcon,
    CheckIcon,
    PackageIcon,
    ArrowLeftIcon,
} from "../components/Icons";
import type { SearchHit, Page } from "../types";
import styles from "./Browse.module.css";

interface BrowseProps {
    navigate: (page: Page) => void;
}

export default function Browse({ navigate }: BrowseProps) {
    const [query, setQuery] = useState("");
    const [results, setResults] = useState<SearchHit[]>([]);
    const [loading, setLoading] = useState(false);
    const [searched, setSearched] = useState(false);
    const [installing, setInstalling] = useState<Record<string, "loading" | "done">>({});
    const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);

    const doSearch = useCallback(async (q: string) => {
        if (q.trim().length === 0) return;
        setLoading(true);
        try {
            const res = await searchModpacks(q.trim(), 20, 0);
            setResults(res.hits);
            setSearched(true);
        } catch (e) {
            console.error("Search failed:", e);
        } finally {
            setLoading(false);
        }
    }, []);

    function onInput(value: string) {
        setQuery(value);
        if (debounceRef.current) clearTimeout(debounceRef.current);
        if (value.trim().length === 0) {
            setResults([]);
            setSearched(false);
            return;
        }
        debounceRef.current = setTimeout(() => doSearch(value), 300);
    }

    function handleInstall(projectId: string) {
        setInstalling((prev) => ({ ...prev, [projectId]: "loading" }));
        // TODO: invoke Tauri install command
        setTimeout(() => {
            setInstalling((prev) => ({ ...prev, [projectId]: "done" }));
        }, 2000);
    }

    return (
        <div className={styles.browse}>
            <button className={styles.back} onClick={() => navigate({ kind: "home" })}>
                <ArrowLeftIcon size={16} />
                <span>Back</span>
            </button>

            <div className={styles.searchWrap}>
                <SearchIcon size={16} className={styles.searchIcon} />
                <input
                    className={styles.searchInput}
                    type="text"
                    placeholder="Search modpacks..."
                    value={query}
                    onChange={(e) => onInput(e.target.value)}
                    autoFocus
                />
            </div>

            <div className={styles.content}>
                {loading ? (
                    <div className={styles.status}>
                        <SpinnerIcon size={18} />
                    </div>
                ) : searched && results.length === 0 ? (
                    <div className={styles.status}>No results</div>
                ) : results.length > 0 ? (
                    <div className={styles.resultsList}>
                        {results.map((hit, i) => {
                            const state = installing[hit.project_id];
                            return (
                                <div
                                    key={hit.project_id}
                                    className={styles.resultRow}
                                    style={{
                                        animationDelay: `${i * 25}ms`,
                                    }}
                                >
                                    <div className={styles.resultIcon}>
                                        {hit.icon_url ? (
                                            <img src={hit.icon_url} alt="" />
                                        ) : (
                                            <div className={styles.resultIconPlaceholder}>
                                                <PackageIcon size={20} />
                                            </div>
                                        )}
                                    </div>
                                    <div className={styles.resultInfo}>
                                        <div className={styles.resultName}>{hit.title}</div>
                                        <div className={styles.resultAuthor}>{hit.author}</div>
                                        <div className={styles.resultDesc}>{hit.description}</div>
                                    </div>
                                    <button
                                        className={`${styles.installBtn} ${state === "done" ? styles.installed : ""}`}
                                        onClick={() => handleInstall(hit.project_id)}
                                        disabled={!!state}
                                    >
                                        {state === "loading" ? (
                                            <SpinnerIcon size={14} />
                                        ) : state === "done" ? (
                                            <CheckIcon size={14} />
                                        ) : (
                                            <DownloadIcon size={14} />
                                        )}
                                    </button>
                                </div>
                            );
                        })}
                    </div>
                ) : (
                    <div className={styles.status}>Search for modpacks to install</div>
                )}
            </div>
        </div>
    );
}
