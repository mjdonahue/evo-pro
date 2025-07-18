import { invoke } from '@tauri-apps/api/core';

export interface TauriLogLevel {
  ERROR: 'ERROR';
  WARN: 'WARN';
  INFO: 'INFO';
  DEBUG: 'DEBUG';
}

export const LogLevel: TauriLogLevel = {
  ERROR: 'ERROR',
  WARN: 'WARN',
  INFO: 'INFO',
  DEBUG: 'DEBUG'
};

export interface LogEntry {
  level: keyof TauriLogLevel;
  message: string;
  timestamp: string;
  source: string;
  metadata?: Record<string, any>;
}

export class TauriLogger {
  private static instance: TauriLogger;
  
  private constructor() {}
  
  static getInstance(): TauriLogger {
    if (!TauriLogger.instance) {
      TauriLogger.instance = new TauriLogger();
    }
    return TauriLogger.instance;
  }

  async log(entry: LogEntry): Promise<void> {
    try {
      await invoke('log_frontend_message', { entry });
    } catch (error) {
      // Fallback to console if Tauri command fails
      console.error('Failed to log to Tauri:', error);
      console.log('Original log entry:', entry);
    }
  }

  async error(message: string, metadata?: Record<string, any>): Promise<void> {
    await this.log({
      level: 'ERROR',
      message,
      timestamp: new Date().toISOString(),
      source: 'React Frontend',
      metadata
    });
  }

  async warn(message: string, metadata?: Record<string, any>): Promise<void> {
    await this.log({
      level: 'WARN',
      message,
      timestamp: new Date().toISOString(),
      source: 'React Frontend',
      metadata
    });
  }

  async info(message: string, metadata?: Record<string, any>): Promise<void> {
    await this.log({
      level: 'INFO',
      message,
      timestamp: new Date().toISOString(),
      source: 'React Frontend',
      metadata
    });
  }

  async debug(message: string, metadata?: Record<string, any>): Promise<void> {
    await this.log({
      level: 'DEBUG',
      message,
      timestamp: new Date().toISOString(),
      source: 'React Frontend',
      metadata
    });
  }

  // Log React errors with rich context
  async logReactError(error: Error, errorInfo?: any, additionalContext?: Record<string, any>): Promise<void> {
    const errorData = {
      message: error.message,
      stack: error.stack,
      name: error.name,
      errorInfo,
      userAgent: navigator.userAgent,
      url: window.location.href,
      timestamp: new Date().toISOString(),
      ...additionalContext
    };

    await this.error('React Error', errorData);
  }
}

export const tauriLogger = TauriLogger.getInstance(); 