import { useCallback, useRef, useState } from "react";
import { searchModpacks } from "../api/modpacks";
import type { SearchHit } from "../types";
import styles from "./Browse.module.css";

function formatDownloads(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return String(n);
}

export default function Browse() {
    const [query, setQuery] = useState("");
    const [results, setResults] = useState<SearchHit[]>([]);
    const [loading, setLoading] = useState(false);
    const [searched, setSearched] = useState(false);
    const [totalHits, setTotalHits] = useState(0);
    const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);

    const doSearch = useCallback(async (q: string) => {
        if (q.trim().length === 0) return;
        setLoading(true);
        try {
            const res = await searchModpacks(q.trim(), 20, 0);
            setResults(res.hits);
            setTotalHits(res.total_hits);
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

    return (
        <div className={styles.browse}>
            <div className={styles.header}>
                <h1>Browse Modpacks</h1>
            </div>

            <div className={styles.searchBar}>
                <input
                    type="text"
                    placeholder="Search modpacks on Modrinth..."
                    value={query}
                    onChange={(e) => onInput(e.target.value)}
                />
            </div>

            {loading ? (
                <div className={styles.status}>Searching...</div>
            ) : searched && results.length === 0 ? (
                <div className={styles.status}>No modpacks found for &ldquo;{query}&rdquo;</div>
            ) : results.length > 0 ? (
                <>
                    <div className={styles.resultsInfo}>{totalHits} modpacks found</div>
                    <div className={styles.resultsGrid}>
                        {results.map((hit) => (
                            <div key={hit.project_id} className={styles.modpackCard}>
                                <div className={styles.modpackIcon}>
                                    {hit.icon_url ? (
                                        <img src={hit.icon_url} alt="" />
                                    ) : (
                                        <div className={styles.iconPlaceholder}>{"\u{1F4E6}"}</div>
                                    )}
                                </div>
                                <div className={styles.modpackInfo}>
                                    <div className={styles.modpackName}>{hit.title}</div>
                                    <div className={styles.modpackAuthor}>by {hit.author}</div>
                                    <div className={styles.modpackDesc}>{hit.description}</div>
                                </div>
                                <div className={styles.modpackStats}>
                                    <span className={styles.downloads}>
                                        {formatDownloads(hit.downloads)} downloads
                                    </span>
                                </div>
                            </div>
                        ))}
                    </div>
                </>
            ) : (
                <div className={styles.empty}>
                    <div className={styles.emptyIcon}>{"\u{1F50D}"}</div>
                    <p>Search for modpacks on Modrinth to install them.</p>
                </div>
            )}
        </div>
    );
}
