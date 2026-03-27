import { useState } from "react";
import Sidebar from "./components/Sidebar";
import Home from "./pages/Home";
import Browse from "./pages/Browse";
import Settings from "./pages/Settings";
import type { Page } from "./types";

export default function App() {
    const [page, setPage] = useState<Page>({ kind: "home" });

    return (
        <>
            <Sidebar page={page} navigate={setPage} />
            <main className="main-content">
                {page.kind === "home" && <Home navigate={setPage} />}
                {page.kind === "browse" && <Browse />}
                {page.kind === "settings" && <Settings />}
            </main>
        </>
    );
}
