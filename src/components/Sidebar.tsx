import type { Page } from "../types";
import styles from "./Sidebar.module.css";

type SidebarPage = Extract<Page, { kind: "home" | "browse" | "settings" }>;

const navItems: { page: SidebarPage; label: string; icon: string }[] = [
    { page: { kind: "home" }, label: "My Packs", icon: "\u{1F3AE}" },
    { page: { kind: "browse" }, label: "Browse", icon: "\u{1F50D}" },
    { page: { kind: "settings" }, label: "Settings", icon: "\u{2699}" },
];

interface SidebarProps {
    page: Page;
    navigate: (page: Page) => void;
}

export default function Sidebar({ page, navigate }: SidebarProps) {
    return (
        <nav className={styles.sidebar}>
            <div className={styles.navItems}>
                {navItems.map((item) => (
                    <button
                        key={item.page.kind}
                        className={`${styles.navItem} ${page.kind === item.page.kind ? styles.active : ""}`}
                        onClick={() => navigate(item.page)}
                    >
                        <span className={styles.navIcon}>{item.icon}</span>
                        <span className={styles.navLabel}>{item.label}</span>
                    </button>
                ))}
            </div>
            <div className={styles.footer}>
                <div className={styles.version}>v0.1.0</div>
            </div>
        </nav>
    );
}
