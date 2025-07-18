/**
 * Utilities for implementing cancelable operations
 */

/**
 * Represents a token that can be used to cancel an operation
 */
export interface CancellationToken {
  /** Whether the token has been canceled */
  isCanceled: boolean;
  /** Throws a CancellationError if the token has been canceled */
  throwIfCanceled(): void;
  /** Registers a callback to be executed when the token is canceled */
  register(callback: () => void): { unregister: () => void };
}

/**
 * Error thrown when an operation is canceled
 */
export class CancellationError extends Error {
  constructor(message: string = 'Operation was canceled') {
    super(message);
    this.name = 'CancellationError';
  }
}

/**
 * Source of a cancellation token
 */
export class CancellationTokenSource {
  private _isCanceled: boolean = false;
  private _callbacks: Set<() => void> = new Set();
  private _linkedSources: Set<CancellationTokenSource> = new Set();
  private _token: CancellationToken;

  constructor(linkedTokens?: CancellationToken[]) {
    this._token = {
      get isCanceled(): boolean {
        return this._isCanceled;
      },
      throwIfCanceled: () => {
        if (this._isCanceled) {
          throw new CancellationError();
        }
      },
      register: (callback: () => void) => {
        if (this._isCanceled) {
          // If already canceled, execute the callback immediately
          try {
            callback();
          } catch (error) {
            console.error('Error in cancellation callback:', error);
          }
          return { unregister: () => {} };
        }

        this._callbacks.add(callback);
        return {
          unregister: () => {
            this._callbacks.delete(callback);
          }
        };
      }
    };

    // Link to other tokens
    if (linkedTokens) {
      for (const token of linkedTokens) {
        if (token.isCanceled) {
          this.cancel();
          break;
        }
        
        if ('register' in token) {
          token.register(() => this.cancel());
        }
      }
    }
  }

  /**
   * Gets the cancellation token
   */
  get token(): CancellationToken {
    return this._token;
  }

  /**
   * Cancels the token
   */
  cancel(): void {
    if (this._isCanceled) {
      return;
    }

    this._isCanceled = true;

    // Execute all callbacks
    for (const callback of this._callbacks) {
      try {
        callback();
      } catch (error) {
        console.error('Error in cancellation callback:', error);
      }
    }

    // Clear callbacks
    this._callbacks.clear();

    // Cancel linked sources
    for (const source of this._linkedSources) {
      source.cancel();
    }
    this._linkedSources.clear();
  }

  /**
   * Links this token source to another token source
   * 
   * @param source - Token source to link to
   */
  link(source: CancellationTokenSource): void {
    if (this._isCanceled) {
      source.cancel();
      return;
    }

    if (source._isCanceled) {
      this.cancel();
      return;
    }

    this._linkedSources.add(source);
    source._linkedSources.add(this);
  }

  /**
   * Disposes the token source
   */
  dispose(): void {
    this._callbacks.clear();
    this._linkedSources.clear();
  }
}

/**
 * Creates a cancellation token that is canceled after a specified timeout
 * 
 * @param timeout - Timeout in milliseconds
 * @returns Cancellation token source
 */
export function createTimeoutCancellationToken(timeout: number): CancellationTokenSource {
  const source = new CancellationTokenSource();
  
  const timeoutId = setTimeout(() => {
    source.cancel();
  }, timeout);
  
  // Register a callback to clear the timeout if the token is canceled
  source.token.register(() => {
    clearTimeout(timeoutId);
  });
  
  return source;
}

/**
 * Creates a cancellation token that is canceled when the window is unloaded
 * 
 * @returns Cancellation token source
 */
export function createUnloadCancellationToken(): CancellationTokenSource {
  const source = new CancellationTokenSource();
  
  const handleUnload = () => {
    source.cancel();
  };
  
  window.addEventListener('beforeunload', handleUnload);
  
  // Register a callback to remove the event listener if the token is canceled
  source.token.register(() => {
    window.removeEventListener('beforeunload', handleUnload);
  });
  
  return source;
}

/**
 * Creates a cancellation token that is canceled when the AbortSignal is aborted
 * 
 * @param signal - AbortSignal to listen to
 * @returns Cancellation token source
 */
export function createAbortSignalCancellationToken(signal: AbortSignal): CancellationTokenSource {
  const source = new CancellationTokenSource();
  
  if (signal.aborted) {
    source.cancel();
    return source;
  }
  
  const handleAbort = () => {
    source.cancel();
  };
  
  signal.addEventListener('abort', handleAbort);
  
  // Register a callback to remove the event listener if the token is canceled
  source.token.register(() => {
    signal.removeEventListener('abort', handleAbort);
  });
  
  return source;
}

/**
 * Executes an operation with cancellation support
 * 
 * @param operation - Operation to execute
 * @param token - Cancellation token
 * @param options - Options for the operation
 * @returns Promise that resolves to the operation result
 */
export async function withCancellation<T>(
  operation: (token: CancellationToken) => Promise<T>,
  token: CancellationToken,
  options: {
    onCanceled?: () => void;
    throwOnCancellation?: boolean;
  } = {}
): Promise<T> {
  // Check if already canceled
  if (token.isCanceled) {
    options.onCanceled?.();
    
    if (options.throwOnCancellation !== false) {
      throw new CancellationError();
    }
    
    return Promise.reject(new CancellationError());
  }
  
  // Create a promise that rejects when the token is canceled
  const cancellationPromise = new Promise<never>((_, reject) => {
    const registration = token.register(() => {
      options.onCanceled?.();
      reject(new CancellationError());
    });
    
    // Clean up the registration when the operation completes
    operation(token).finally(() => {
      registration.unregister();
    });
  });
  
  // Race the operation against cancellation
  if (options.throwOnCancellation === false) {
    try {
      return await Promise.race([operation(token), cancellationPromise]);
    } catch (error) {
      if (error instanceof CancellationError) {
        options.onCanceled?.();
        return Promise.reject(error);
      }
      throw error;
    }
  } else {
    return Promise.race([operation(token), cancellationPromise]);
  }
}

/**
 * Wraps a function to make it cancelable
 * 
 * @param fn - Function to wrap
 * @returns Cancelable function
 */
export function makeCancelable<T, Args extends any[]>(
  fn: (...args: Args) => Promise<T>
): {
  (...args: Args): Promise<T>;
  cancel: () => void;
} {
  const source = new CancellationTokenSource();
  
  const cancelableFn = async (...args: Args): Promise<T> => {
    return withCancellation(
      async (token) => {
        // Check for cancellation periodically
        const result = await fn(...args);
        token.throwIfCanceled();
        return result;
      },
      source.token
    );
  };
  
  cancelableFn.cancel = () => {
    source.cancel();
  };
  
  return cancelableFn;
}

/**
 * Creates a cancelable promise
 * 
 * @param promise - Promise to make cancelable
 * @returns Cancelable promise
 */
export function cancelablePromise<T>(promise: Promise<T>): {
  promise: Promise<T>;
  cancel: () => void;
} {
  const source = new CancellationTokenSource();
  
  const cancelablePromise = withCancellation(
    async () => {
      return promise;
    },
    source.token
  );
  
  return {
    promise: cancelablePromise,
    cancel: () => {
      source.cancel();
    }
  };
}

/**
 * Executes multiple operations in parallel with cancellation support
 * 
 * @param operations - Operations to execute
 * @param token - Cancellation token
 * @returns Promise that resolves to an array of operation results
 */
export async function allWithCancellation<T>(
  operations: ((token: CancellationToken) => Promise<T>)[],
  token: CancellationToken
): Promise<T[]> {
  // Check if already canceled
  if (token.isCanceled) {
    throw new CancellationError();
  }
  
  // Create a promise for each operation
  const promises = operations.map(operation => {
    return withCancellation(operation, token);
  });
  
  // Wait for all promises to resolve
  return Promise.all(promises);
}

/**
 * Executes an operation with a timeout
 * 
 * @param operation - Operation to execute
 * @param timeout - Timeout in milliseconds
 * @returns Promise that resolves to the operation result
 */
export async function withTimeout<T>(
  operation: (token: CancellationToken) => Promise<T>,
  timeout: number
): Promise<T> {
  const source = createTimeoutCancellationToken(timeout);
  
  try {
    return await withCancellation(operation, source.token);
  } finally {
    source.dispose();
  }
}

/**
 * Hook for using cancellation in React components
 * 
 * @returns Cancellation utilities
 */
export function useCancellation() {
  const sourceRef = React.useRef<CancellationTokenSource | null>(null);
  
  // Create a new token source if one doesn't exist
  if (!sourceRef.current) {
    sourceRef.current = new CancellationTokenSource();
  }
  
  // Clean up the token source when the component unmounts
  React.useEffect(() => {
    return () => {
      if (sourceRef.current) {
        sourceRef.current.cancel();
        sourceRef.current.dispose();
        sourceRef.current = null;
      }
    };
  }, []);
  
  return {
    /**
     * Gets the current cancellation token
     */
    token: sourceRef.current.token,
    
    /**
     * Cancels the current token
     */
    cancel: () => {
      sourceRef.current?.cancel();
    },
    
    /**
     * Creates a new token source
     */
    createTokenSource: () => {
      return new CancellationTokenSource();
    },
    
    /**
     * Executes an operation with cancellation support
     * 
     * @param operation - Operation to execute
     * @returns Promise that resolves to the operation result
     */
    withCancellation: <T>(operation: (token: CancellationToken) => Promise<T>): Promise<T> => {
      return withCancellation(operation, sourceRef.current!.token);
    },
    
    /**
     * Makes a function cancelable
     * 
     * @param fn - Function to make cancelable
     * @returns Cancelable function
     */
    makeCancelable: <T, Args extends any[]>(
      fn: (...args: Args) => Promise<T>
    ): {
      (...args: Args): Promise<T>;
      cancel: () => void;
    } => {
      return makeCancelable(fn);
    },
  };
}