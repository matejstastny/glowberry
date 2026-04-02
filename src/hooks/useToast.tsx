import { createContext, useCallback, useContext, useState } from "react";

interface Toast {
    id: number;
    message: string;
    type: "success" | "error" | "info";
}

interface ToastContextValue {
    toasts: Toast[];
    toast: (message: string, type?: Toast["type"]) => void;
    dismiss: (id: number) => void;
}

const ToastContext = createContext<ToastContextValue>({
    toasts: [],
    toast: () => {},
    dismiss: () => {},
});

let nextId = 0;

export function ToastProvider({ children }: { children: React.ReactNode }) {
    const [toasts, setToasts] = useState<Toast[]>([]);

    const dismiss = useCallback((id: number) => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
    }, []);

    const toast = useCallback(
        (message: string, type: Toast["type"] = "info") => {
            const id = nextId++;
            setToasts((prev) => [...prev, { id, message, type }]);
            setTimeout(() => dismiss(id), 3500);
        },
        [dismiss],
    );

    return (
        <ToastContext.Provider value={{ toasts, toast, dismiss }}>
            {children}
        </ToastContext.Provider>
    );
}

export function useToast() {
    return useContext(ToastContext);
}
