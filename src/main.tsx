import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ToastProvider } from "./hooks/useToast";
import ToastContainer from "./components/ToastContainer";
import "./styles.css";

if (navigator.platform.startsWith("Mac")) {
    document.documentElement.classList.add("macos");
}

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <ToastProvider>
            <App />
            <ToastContainer />
        </ToastProvider>
    </React.StrictMode>,
);
