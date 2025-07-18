import { useState, createContext, useContext } from "react";

interface Toast {
  id: string;
  title?: string;
  description?: string;
  variant?: "default" | "destructive";
  duration?: number;
  action?: React.ReactNode;
}

interface ToastContextValue {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, "id">) => void;
  removeToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = (toast: Omit<Toast, "id">) => {
    const id = Math.random().toString(36).substring(2, 9);
    const newToast = { ...toast, id };
    
    setToasts((prev) => [...prev, newToast]);
    
    // Auto dismiss
    if (toast.duration !== 0) {
      setTimeout(() => {
        removeToast(id);
      }, toast.duration || 5000);
    }
  };

  const removeToast = (id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  };

  return (
    <ToastContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
      <ToastContainer />
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  
  if (context === undefined) {
    throw new Error("useToast must be used within a ToastProvider");
  }
  
  const { toasts, addToast } = context;
  
  return {
    toasts,
    toast: (props: Omit<Toast, "id">) => addToast(props),
  };
}

function ToastContainer() {
  const context = useContext(ToastContext);
  
  if (context === undefined) {
    return null;
  }
  
  const { toasts, removeToast } = context;
  
  if (toasts.length === 0) {
    return null;
  }
  
  return (
    <div className="fixed top-0 right-0 p-4 z-50 max-h-screen overflow-y-auto flex flex-col gap-2">
      {toasts.map((toast) => (
        <div 
          key={toast.id}
          className={`rounded-md border p-4 ${
            toast.variant === "destructive" 
              ? "bg-destructive text-destructive-foreground" 
              : "bg-background border-border"
          } shadow-md transition-all flex flex-col gap-1 max-w-md w-full relative`}
          role="alert"
        >
          <button 
            onClick={() => removeToast(toast.id)}
            className="absolute top-2 right-2 opacity-70 hover:opacity-100"
            aria-label="Close"
          >
            ✕
          </button>
          
          {toast.title && (
            <div className="font-semibold">{toast.title}</div>
          )}
          
          {toast.description && (
            <div className="text-sm opacity-90">{toast.description}</div>
          )}
        </div>
      ))}
    </div>
  );
} 