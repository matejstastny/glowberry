import { useState } from "react";
import styles from "./OfflinePopup.module.css";

interface OfflinePopupProps {
    onSubmit: (username: string) => void;
    onCancel: () => void;
}

export default function OfflinePopup({ onSubmit, onCancel }: OfflinePopupProps) {
    const [username, setUsername] = useState("");

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const trimmed = username.trim();
        if (trimmed.length > 0) {
            onSubmit(trimmed);
        }
    }

    return (
        <div className={styles.overlay} onClick={onCancel}>
            <div className={styles.popup} onClick={(e) => e.stopPropagation()}>
                <div className={styles.title}>Offline Username</div>
                <form onSubmit={handleSubmit}>
                    <input
                        className={styles.input}
                        type="text"
                        placeholder="Enter username..."
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        autoFocus
                        maxLength={16}
                    />
                    <button
                        className={styles.confirmBtn}
                        type="submit"
                        disabled={username.trim().length === 0}
                    >
                        Confirm
                    </button>
                </form>
            </div>
        </div>
    );
}
