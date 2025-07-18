/**
 * Error handling system for API client
 * 
 * This module provides a standardized approach to error classification and handling
 * with support for error categories, contextual information, and error utilities.
 */

import { ApiResponse } from './types';

/**
 * Error categories for classification
 */
export enum ErrorCategory {
  // Client-side errors (4xx)
  VALIDATION = 'validation',
  AUTHENTICATION = 'authentication',
  AUTHORIZATION = 'authorization',
  NOT_FOUND = 'not_found',
  CONFLICT = 'conflict',
  RATE_LIMIT = 'rate_limit',
  BAD_REQUEST = 'bad_request',
  
  // Server-side errors (5xx)
  SERVER = 'server',
  DATABASE = 'database',
  TIMEOUT = 'timeout',
  UNAVAILABLE = 'unavailable',
  
  // Network/connectivity errors
  NETWORK = 'network',
  OFFLINE = 'offline',
  
  // Application-specific errors
  BUSINESS_LOGIC = 'business_logic',
  
  // Unknown/unexpected errors
  UNKNOWN = 'unknown'
}

/**
 * Error severity levels
 */
export enum ErrorSeverity {
  INFO = 'info',
  WARNING = 'warning',
  ERROR = 'error',
  CRITICAL = 'critical'
}

/**
 * Error context information
 */
export interface ErrorContext {
  /** The operation that was being performed */
  operation?: string;
  /** The entity type involved in the operation */
  entityType?: string;
  /** The entity ID involved in the operation */
  entityId?: string | string[];
  /** The HTTP status code (if applicable) */
  statusCode?: number;
  /** The request that caused the error */
  request?: Record<string, any>;
  /** Additional details about the error */
  details?: Record<string, any>;
  /** The original error that was caught */
  originalError?: Error | unknown;
  /** Timestamp when the error occurred */
  timestamp?: number;
  /** Correlation ID for tracking related errors */
  correlationId?: string;
}

/**
 * Base API error class with enhanced error information
 */
export class ApiError extends Error {
  /** Error code (specific error identifier) */
  public code: string;
  
  /** Error category (for classification) */
  public category: ErrorCategory;
  
  /** Error severity */
  public severity: ErrorSeverity;
  
  /** Error context with additional information */
  public context: ErrorContext;
  
  /** Whether the error is retryable */
  public retryable: boolean;
  
  /** Suggested user-friendly message */
  public userMessage?: string;
  
  /** Suggested recovery action */
  public recoveryAction?: string;
  
  /**
   * Creates a new API error
   * @param code - Specific error code
   * @param message - Technical error message
   * @param category - Error category for classification
   * @param context - Additional context information
   * @param options - Additional error options
   */
  constructor(
    code: string,
    message: string,
    category: ErrorCategory = ErrorCategory.UNKNOWN,
    context: ErrorContext = {},
    options: {
      severity?: ErrorSeverity;
      retryable?: boolean;
      userMessage?: string;
      recoveryAction?: string;
    } = {}
  ) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.category = category;
    this.severity = options.severity || ErrorSeverity.ERROR;
    this.retryable = options.retryable || false;
    this.userMessage = options.userMessage;
    this.recoveryAction = options.recoveryAction;
    
    // Add timestamp if not provided
    this.context = {
      ...context,
      timestamp: context.timestamp || Date.now()
    };
    
    // Capture stack trace
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, ApiError);
    }
  }
  
  /**
   * Creates a string representation of the error
   */
  toString(): string {
    return `[${this.category}] ${this.code}: ${this.message}`;
  }
  
  /**
   * Creates a JSON representation of the error
   */
  toJSON(): Record<string, any> {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      category: this.category,
      severity: this.severity,
      retryable: this.retryable,
      userMessage: this.userMessage,
      recoveryAction: this.recoveryAction,
      context: this.context
    };
  }
  
  /**
   * Enriches the error with additional context
   * @param context - Additional context to add
   * @returns The enriched error
   */
  enrich(context: Partial<ErrorContext>): this {
    this.context = {
      ...this.context,
      ...context
    };
    return this;
  }
}

/**
 * Factory functions for creating common error types
 */
export const Errors = {
  /**
   * Creates a validation error
   */
  validation: (
    message: string,
    details?: Record<string, any>,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      'validation_error',
      message,
      ErrorCategory.VALIDATION,
      { details, ...options },
      {
        severity: ErrorSeverity.WARNING,
        retryable: false,
        userMessage: options?.userMessage || 'The provided data is invalid.',
        recoveryAction: options?.recoveryAction || 'Please check your input and try again.'
      }
    );
  },
  
  /**
   * Creates a not found error
   */
  notFound: (
    entityType: string,
    entityId?: string | string[],
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    const idStr = Array.isArray(entityId) 
      ? entityId.join(', ') 
      : entityId || 'unknown';
    
    return new ApiError(
      'not_found_error',
      `${entityType} with ID ${idStr} not found`,
      ErrorCategory.NOT_FOUND,
      { entityType, entityId, ...options },
      {
        severity: ErrorSeverity.WARNING,
        retryable: false,
        userMessage: options?.userMessage || `The requested ${entityType} could not be found.`,
        recoveryAction: options?.recoveryAction || 'Please check the ID and try again.'
      }
    );
  },
  
  /**
   * Creates an authentication error
   */
  authentication: (
    message: string,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      'authentication_error',
      message,
      ErrorCategory.AUTHENTICATION,
      options,
      {
        severity: ErrorSeverity.ERROR,
        retryable: false,
        userMessage: options?.userMessage || 'Authentication failed.',
        recoveryAction: options?.recoveryAction || 'Please sign in again.'
      }
    );
  },
  
  /**
   * Creates an authorization error
   */
  authorization: (
    message: string,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      'authorization_error',
      message,
      ErrorCategory.AUTHORIZATION,
      options,
      {
        severity: ErrorSeverity.ERROR,
        retryable: false,
        userMessage: options?.userMessage || 'You do not have permission to perform this action.',
        recoveryAction: options?.recoveryAction || 'Please contact an administrator if you need access.'
      }
    );
  },
  
  /**
   * Creates a network error
   */
  network: (
    message: string,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      'network_error',
      message,
      ErrorCategory.NETWORK,
      options,
      {
        severity: ErrorSeverity.ERROR,
        retryable: true,
        userMessage: options?.userMessage || 'A network error occurred.',
        recoveryAction: options?.recoveryAction || 'Please check your connection and try again.'
      }
    );
  },
  
  /**
   * Creates a server error
   */
  server: (
    message: string,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      'server_error',
      message,
      ErrorCategory.SERVER,
      options,
      {
        severity: ErrorSeverity.ERROR,
        retryable: true,
        userMessage: options?.userMessage || 'An unexpected server error occurred.',
        recoveryAction: options?.recoveryAction || 'Please try again later.'
      }
    );
  },
  
  /**
   * Creates a timeout error
   */
  timeout: (
    message: string,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      'timeout_error',
      message,
      ErrorCategory.TIMEOUT,
      options,
      {
        severity: ErrorSeverity.WARNING,
        retryable: true,
        userMessage: options?.userMessage || 'The operation timed out.',
        recoveryAction: options?.recoveryAction || 'Please try again.'
      }
    );
  },
  
  /**
   * Creates a business logic error
   */
  businessLogic: (
    code: string,
    message: string,
    options?: Partial<ErrorContext & { userMessage?: string; recoveryAction?: string }>
  ): ApiError => {
    return new ApiError(
      code,
      message,
      ErrorCategory.BUSINESS_LOGIC,
      options,
      {
        severity: ErrorSeverity.WARNING,
        retryable: false,
        userMessage: options?.userMessage,
        recoveryAction: options?.recoveryAction
      }
    );
  },
  
  /**
   * Creates a generic error
   */
  generic: (
    code: string,
    message: string,
    category: ErrorCategory = ErrorCategory.UNKNOWN,
    options?: Partial<ErrorContext & { 
      severity?: ErrorSeverity;
      retryable?: boolean;
      userMessage?: string; 
      recoveryAction?: string 
    }>
  ): ApiError => {
    return new ApiError(
      code,
      message,
      category,
      options,
      {
        severity: options?.severity || ErrorSeverity.ERROR,
        retryable: options?.retryable || false,
        userMessage: options?.userMessage,
        recoveryAction: options?.recoveryAction
      }
    );
  }
};

/**
 * Utility functions for error handling
 */
export const ErrorUtils = {
  /**
   * Parses an API response error into an ApiError
   * @param response - The API response with an error
   * @param context - Additional context information
   * @returns An ApiError instance
   */
  parseApiResponse: <T>(
    response: ApiResponse<T>,
    context: Partial<ErrorContext> = {}
  ): ApiError => {
    // If the response has an error code, use it to determine the category
    const errorCode = response.errorCode || 'api_error';
    const message = response.error || 'Unknown API error';
    
    // Determine error category based on error code pattern
    let category = ErrorCategory.UNKNOWN;
    if (errorCode.includes('not_found')) {
      category = ErrorCategory.NOT_FOUND;
    } else if (errorCode.includes('validation')) {
      category = ErrorCategory.VALIDATION;
    } else if (errorCode.includes('auth')) {
      category = ErrorCategory.AUTHENTICATION;
    } else if (errorCode.includes('permission')) {
      category = ErrorCategory.AUTHORIZATION;
    } else if (errorCode.includes('conflict')) {
      category = ErrorCategory.CONFLICT;
    } else if (errorCode.includes('server')) {
      category = ErrorCategory.SERVER;
    } else if (errorCode.includes('database')) {
      category = ErrorCategory.DATABASE;
    } else if (errorCode.includes('timeout')) {
      category = ErrorCategory.TIMEOUT;
    } else if (errorCode.includes('business')) {
      category = ErrorCategory.BUSINESS_LOGIC;
    }
    
    return new ApiError(
      errorCode,
      message,
      category,
      context,
      {
        retryable: [
          ErrorCategory.NETWORK,
          ErrorCategory.TIMEOUT,
          ErrorCategory.UNAVAILABLE,
          ErrorCategory.SERVER
        ].includes(category)
      }
    );
  },
  
  /**
   * Converts an unknown error to an ApiError
   * @param error - The error to convert
   * @param context - Additional context information
   * @returns An ApiError instance
   */
  fromUnknown: (
    error: unknown,
    context: Partial<ErrorContext> = {}
  ): ApiError => {
    // If it's already an ApiError, just enrich it with the context
    if (error instanceof ApiError) {
      return error.enrich(context);
    }
    
    // If it's a standard Error, convert it
    if (error instanceof Error) {
      return new ApiError(
        'unknown_error',
        error.message,
        ErrorCategory.UNKNOWN,
        {
          ...context,
          originalError: error
        }
      );
    }
    
    // For other types, create a generic error
    return new ApiError(
      'unknown_error',
      typeof error === 'string' ? error : 'An unknown error occurred',
      ErrorCategory.UNKNOWN,
      {
        ...context,
        originalError: error
      }
    );
  },
  
  /**
   * Checks if an error is retryable
   * @param error - The error to check
   * @returns Whether the error is retryable
   */
  isRetryable: (error: unknown): boolean => {
    if (error instanceof ApiError) {
      return error.retryable;
    }
    return false;
  },
  
  /**
   * Gets a user-friendly message for an error
   * @param error - The error to get a message for
   * @param fallback - Fallback message if no user message is available
   * @returns A user-friendly error message
   */
  getUserMessage: (
    error: unknown,
    fallback: string = 'An unexpected error occurred.'
  ): string => {
    if (error instanceof ApiError && error.userMessage) {
      return error.userMessage;
    }
    return fallback;
  },
  
  /**
   * Gets a recovery action for an error
   * @param error - The error to get a recovery action for
   * @param fallback - Fallback recovery action if none is available
   * @returns A recovery action suggestion
   */
  getRecoveryAction: (
    error: unknown,
    fallback: string = 'Please try again or contact support if the problem persists.'
  ): string => {
    if (error instanceof ApiError && error.recoveryAction) {
      return error.recoveryAction;
    }
    return fallback;
  }
};