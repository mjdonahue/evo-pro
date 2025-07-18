import { tauriLogger } from '../lib/tauri-logger';

interface ErrorHandlerConfig {
  enableConsoleLogging?: boolean;
  enableExternalLogging?: boolean;
  enableLocalStorage?: boolean;
  enableTauriLogging?: boolean;
  maxLocalStorageErrors?: number;
}

export class ErrorHandler {
  private config: ErrorHandlerConfig;
  private originalConsoleError: typeof console.error;

  constructor(config: ErrorHandlerConfig = {}) {
    this.config = {
      enableConsoleLogging: true,
      enableExternalLogging: false,
      enableLocalStorage: true,
      enableTauriLogging: true,
      maxLocalStorageErrors: 50,
      ...config,
    };

    // Store original console.error before any modifications
    this.originalConsoleError = console.error.bind(console);
    this.setupGlobalHandlers();
  }

  private setupGlobalHandlers() {
    // Handle unhandled promise rejections
    window.addEventListener('unhandledrejection', (event) => {
      this.handleError(new Error(event.reason), 'Unhandled Promise Rejection');
    });

    // Handle global errors
    window.addEventListener('error', (event) => {
      this.handleError(event.error || new Error(event.message), 'Global Error');
    });

    // Don't override console.error - it causes infinite loops
  }

  public handleError(error: Error, source: string) {
    const errorData = {
      message: error.message,
      stack: error.stack,
      source,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
    };

    if (this.config.enableConsoleLogging) {
      this.originalConsoleError(`[${source}]`, errorData);
    }

    if (this.config.enableLocalStorage) {
      this.storeErrorLocally(errorData);
    }

    if (this.config.enableExternalLogging) {
      this.sendToExternalService(errorData);
    }

    if (this.config.enableTauriLogging) {
      this.sendToTauri(error, source, errorData);
    }
  }

  private storeErrorLocally(errorData: any) {
    try {
      const errorLogs = JSON.parse(localStorage.getItem('errorLogs') || '[]');
      errorLogs.push(errorData);
      
      // Keep only the last N errors
      if (errorLogs.length > this.config.maxLocalStorageErrors!) {
        errorLogs.splice(0, errorLogs.length - this.config.maxLocalStorageErrors!);
      }
      
      localStorage.setItem('errorLogs', JSON.stringify(errorLogs));
    } catch (e) {
      // Use original console to avoid loops
      console.warn('Failed to store error locally:', e);
    }
  }

  private sendToExternalService(errorData: any) {
    // Implement your external logging service here
    // Example: Sentry, LogRocket, etc.
    // console.log('Sending to external service:', errorData);
  }

  private async sendToTauri(error: Error, source: string, errorData: any) {
    try {
      await tauriLogger.logReactError(error, { source }, errorData);
    } catch (tauriError) {
      // Fallback to console if Tauri logging fails
      console.warn('Failed to log to Tauri:', tauriError);
    }
  }

  public logError(error: Error | string, additionalData?: Record<string, any>) {
    const errorObj = typeof error === 'string' ? new Error(error) : error;
    this.handleError(errorObj, 'Manual Log');
  }

  public getStoredErrors(): any[] {
    try {
      return JSON.parse(localStorage.getItem('errorLogs') || '[]');
    } catch {
      return [];
    }
  }

  public clearStoredErrors() {
    localStorage.removeItem('errorLogs');
  }
}

// Create a global instance
export const globalErrorHandler = new ErrorHandler(); 