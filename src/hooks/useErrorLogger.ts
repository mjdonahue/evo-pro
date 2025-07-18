import { useCallback } from 'react';
import { tauriLogger } from '../lib/tauri-logger';

interface ErrorLogData {
  message: string;
  stack?: string;
  componentName?: string;
  userId?: string;
  sessionId?: string;
  timestamp: string;
  userAgent: string;
  url: string;
  additionalData?: Record<string, any>;
}

export const useErrorLogger = () => {
  const logError = useCallback(async (
    error: Error | string,
    additionalData?: Record<string, any>
  ) => {
    const errorData: ErrorLogData = {
      message: typeof error === 'string' ? error : error.message,
      stack: typeof error === 'string' ? undefined : error.stack,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
      additionalData,
    };

    // Log to Tauri backend
    try {
      const errorObj = typeof error === 'string' ? new Error(error) : error;
      await tauriLogger.logReactError(errorObj, undefined, {
        source: 'useErrorLogger',
        ...additionalData
      });
    } catch (tauriError) {
      console.warn('Failed to log to Tauri:', tauriError);
    }

    // Send to external logging service
    // Example: sendToSentry(errorData);
    // Example: sendToLogRocket(errorData);
    
    // Store in localStorage for debugging
    const errorLogs = JSON.parse(localStorage.getItem('errorLogs') || '[]');
    errorLogs.push(errorData);
    localStorage.setItem('errorLogs', JSON.stringify(errorLogs.slice(-50))); // Keep last 50 errors
  }, []);

  const logWarning = useCallback((
    message: string,
    additionalData?: Record<string, any>
  ) => {
    console.warn('Application Warning:', { message, additionalData });
  }, []);

  const logInfo = useCallback((
    message: string,
    additionalData?: Record<string, any>
  ) => {
    console.info('Application Info:', { message, additionalData });
  }, []);

  return { logError, logWarning, logInfo };
}; 