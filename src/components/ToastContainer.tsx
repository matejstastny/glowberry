import { useToast } from "@/hooks/useToast";
import styles from "./ToastContainer.module.css";

export default function ToastContainer() {
    const { toasts, dismiss } = useToast();

    if (toasts.length === 0) return null;

    return (
        <div className={styles.container}>
            {toasts.map((t) => (
                <div
                    key={t.id}
                    className={`${styles.toast} ${styles[t.type]}`}
                    onClick={() => dismiss(t.id)}
                >
                    {t.message}
                </div>
            ))}
        </div>
    );
}
