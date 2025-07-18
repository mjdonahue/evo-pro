/**
 * Utilities for resource throttling and monitoring
 */

/**
 * Resource types that can be monitored and throttled
 */
export enum ResourceType {
  /** CPU usage */
  CPU = 'cpu',
  /** Memory usage */
  MEMORY = 'memory',
  /** Network bandwidth */
  NETWORK = 'network',
  /** Battery level */
  BATTERY = 'battery',
  /** Storage space */
  STORAGE = 'storage',
}

/**
 * Resource usage levels
 */
export enum ResourceUsageLevel {
  /** Low resource usage (0-30%) */
  LOW = 'low',
  /** Medium resource usage (31-70%) */
  MEDIUM = 'medium',
  /** High resource usage (71-90%) */
  HIGH = 'high',
  /** Critical resource usage (91-100%) */
  CRITICAL = 'critical',
}

/**
 * Resource throttling strategy
 */
export enum ThrottlingStrategy {
  /** No throttling */
  NONE = 'none',
  /** Reduce resource usage by limiting concurrent operations */
  LIMIT_CONCURRENCY = 'limit_concurrency',
  /** Reduce resource usage by increasing delays between operations */
  INCREASE_DELAY = 'increase_delay',
  /** Reduce resource usage by lowering quality or precision */
  LOWER_QUALITY = 'lower_quality',
  /** Pause non-critical operations until resources are available */
  PAUSE_NON_CRITICAL = 'pause_non_critical',
  /** Adaptive throttling based on resource usage */
  ADAPTIVE = 'adaptive',
}

/**
 * Resource usage information
 */
export interface ResourceUsage {
  /** Resource type */
  type: ResourceType;
  /** Current usage percentage (0-100) */
  usage: number;
  /** Usage level */
  level: ResourceUsageLevel;
  /** Available capacity (in appropriate units) */
  available: number;
  /** Total capacity (in appropriate units) */
  total: number;
  /** Additional resource-specific information */
  details?: Record<string, any>;
}

/**
 * Resource throttling configuration
 */
export interface ResourceThrottlingConfig {
  /** Throttling strategy to use */
  strategy: ThrottlingStrategy;
  /** Resources to monitor */
  resources: ResourceType[];
  /** Threshold for low usage level (default: 30) */
  lowThreshold?: number;
  /** Threshold for medium usage level (default: 70) */
  mediumThreshold?: number;
  /** Threshold for high usage level (default: 90) */
  highThreshold?: number;
  /** Maximum concurrent operations when using LIMIT_CONCURRENCY strategy */
  maxConcurrentOperations?: number;
  /** Delay between operations when using INCREASE_DELAY strategy (in milliseconds) */
  operationDelay?: number;
  /** Quality level when using LOWER_QUALITY strategy (0-100) */
  qualityLevel?: number;
  /** Callback when resource usage changes */
  onResourceUsageChange?: (usage: ResourceUsage) => void;
  /** Interval for checking resource usage (in milliseconds) */
  monitoringInterval?: number;
}

/**
 * Resource throttling manager
 */
export class ResourceThrottler {
  private config: Required<ResourceThrottlingConfig>;
  private resourceUsage: Map<ResourceType, ResourceUsage> = new Map();
  private monitoringIntervalId?: number;
  private concurrencyLimiter: {
    current: number;
    max: number;
    queue: Array<() => void>;
  };
  private isMonitoring: boolean = false;

  /**
   * Creates a new ResourceThrottler instance
   * 
   * @param config - Throttling configuration
   */
  constructor(config: ResourceThrottlingConfig) {
    this.config = {
      strategy: config.strategy,
      resources: config.resources,
      lowThreshold: config.lowThreshold ?? 30,
      mediumThreshold: config.mediumThreshold ?? 70,
      highThreshold: config.highThreshold ?? 90,
      maxConcurrentOperations: config.maxConcurrentOperations ?? 4,
      operationDelay: config.operationDelay ?? 100,
      qualityLevel: config.qualityLevel ?? 80,
      onResourceUsageChange: config.onResourceUsageChange ?? (() => {}),
      monitoringInterval: config.monitoringInterval ?? 5000,
    };

    this.concurrencyLimiter = {
      current: 0,
      max: this.config.maxConcurrentOperations,
      queue: [],
    };

    // Initialize resource usage
    for (const resource of this.config.resources) {
      this.resourceUsage.set(resource, {
        type: resource,
        usage: 0,
        level: ResourceUsageLevel.LOW,
        available: 0,
        total: 0,
      });
    }
  }

  /**
   * Starts monitoring resources
   */
  startMonitoring(): void {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;
    this.updateResourceUsage();

    this.monitoringIntervalId = window.setInterval(() => {
      this.updateResourceUsage();
    }, this.config.monitoringInterval);
  }

  /**
   * Stops monitoring resources
   */
  stopMonitoring(): void {
    if (!this.isMonitoring) {
      return;
    }

    this.isMonitoring = false;

    if (this.monitoringIntervalId !== undefined) {
      clearInterval(this.monitoringIntervalId);
      this.monitoringIntervalId = undefined;
    }
  }

  /**
   * Gets the current resource usage
   * 
   * @param resourceType - Resource type to get usage for
   * @returns Resource usage information
   */
  getResourceUsage(resourceType: ResourceType): ResourceUsage | undefined {
    return this.resourceUsage.get(resourceType);
  }

  /**
   * Gets all resource usage information
   * 
   * @returns Map of resource usage information
   */
  getAllResourceUsage(): Map<ResourceType, ResourceUsage> {
    return new Map(this.resourceUsage);
  }

  /**
   * Throttles an operation based on the current resource usage and throttling strategy
   * 
   * @param operation - Operation to throttle
   * @param options - Throttling options
   * @returns Promise that resolves when the operation can be executed
   */
  async throttle<T>(
    operation: () => Promise<T> | T,
    options: {
      priority?: number;
      resourceType?: ResourceType;
      bypassThrottling?: boolean;
    } = {}
  ): Promise<T> {
    // If throttling is bypassed, execute the operation immediately
    if (options.bypassThrottling) {
      return operation();
    }

    // Apply throttling based on the strategy
    switch (this.config.strategy) {
      case ThrottlingStrategy.NONE:
        return operation();

      case ThrottlingStrategy.LIMIT_CONCURRENCY:
        return this.throttleWithConcurrencyLimit(operation);

      case ThrottlingStrategy.INCREASE_DELAY:
        return this.throttleWithDelay(operation);

      case ThrottlingStrategy.LOWER_QUALITY:
        // For lower quality strategy, we just execute the operation
        // The quality level should be handled by the operation itself
        return operation();

      case ThrottlingStrategy.PAUSE_NON_CRITICAL:
        return this.throttleWithPriority(operation, options.priority ?? 0);

      case ThrottlingStrategy.ADAPTIVE:
        return this.throttleAdaptively(operation, options.resourceType);

      default:
        return operation();
    }
  }

  /**
   * Updates the throttling configuration
   * 
   * @param config - New throttling configuration
   */
  updateConfig(config: Partial<ResourceThrottlingConfig>): void {
    this.config = {
      ...this.config,
      ...config,
      resources: config.resources ?? this.config.resources,
    };

    // Update concurrency limiter
    if (config.maxConcurrentOperations !== undefined) {
      this.concurrencyLimiter.max = config.maxConcurrentOperations;
    }

    // Restart monitoring if resources changed
    if (config.resources || config.monitoringInterval) {
      this.stopMonitoring();
      this.startMonitoring();
    }
  }

  /**
   * Throttles an operation by limiting concurrency
   * 
   * @param operation - Operation to throttle
   * @returns Promise that resolves to the operation result
   */
  private async throttleWithConcurrencyLimit<T>(operation: () => Promise<T> | T): Promise<T> {
    // If we're under the concurrency limit, execute the operation immediately
    if (this.concurrencyLimiter.current < this.concurrencyLimiter.max) {
      this.concurrencyLimiter.current++;
      
      try {
        return await operation();
      } finally {
        this.concurrencyLimiter.current--;
        
        // Execute the next operation in the queue if any
        const next = this.concurrencyLimiter.queue.shift();
        if (next) {
          next();
        }
      }
    }
    
    // Otherwise, queue the operation
    return new Promise<T>((resolve, reject) => {
      this.concurrencyLimiter.queue.push(async () => {
        try {
          this.concurrencyLimiter.current++;
          const result = await operation();
          resolve(result);
        } catch (error) {
          reject(error);
        } finally {
          this.concurrencyLimiter.current--;
          
          // Execute the next operation in the queue if any
          const next = this.concurrencyLimiter.queue.shift();
          if (next) {
            next();
          }
        }
      });
    });
  }

  /**
   * Throttles an operation by adding a delay
   * 
   * @param operation - Operation to throttle
   * @returns Promise that resolves to the operation result
   */
  private async throttleWithDelay<T>(operation: () => Promise<T> | T): Promise<T> {
    // Calculate delay based on resource usage
    const delay = this.calculateAdaptiveDelay();
    
    // Add delay before executing the operation
    await new Promise(resolve => setTimeout(resolve, delay));
    
    return operation();
  }

  /**
   * Throttles an operation based on priority
   * 
   * @param operation - Operation to throttle
   * @param priority - Operation priority (lower number = higher priority)
   * @returns Promise that resolves to the operation result
   */
  private async throttleWithPriority<T>(operation: () => Promise<T> | T, priority: number): Promise<T> {
    // Check if any resource is at critical level
    const hasCriticalResource = Array.from(this.resourceUsage.values()).some(
      usage => usage.level === ResourceUsageLevel.CRITICAL
    );
    
    // If resources are critical and the operation is not high priority, delay it
    if (hasCriticalResource && priority > 0) {
      await new Promise(resolve => setTimeout(resolve, 1000));
      return this.throttleWithPriority(operation, priority);
    }
    
    // If resources are high and the operation is low priority, add some delay
    const hasHighResource = Array.from(this.resourceUsage.values()).some(
      usage => usage.level === ResourceUsageLevel.HIGH
    );
    
    if (hasHighResource && priority > 1) {
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    
    return operation();
  }

  /**
   * Throttles an operation adaptively based on resource usage
   * 
   * @param operation - Operation to throttle
   * @param resourceType - Resource type to consider for throttling
   * @returns Promise that resolves to the operation result
   */
  private async throttleAdaptively<T>(
    operation: () => Promise<T> | T,
    resourceType?: ResourceType
  ): Promise<T> {
    // Get the resource usage level
    let usageLevel = ResourceUsageLevel.LOW;
    
    if (resourceType) {
      // Use the specified resource type
      const usage = this.resourceUsage.get(resourceType);
      if (usage) {
        usageLevel = usage.level;
      }
    } else {
      // Use the highest usage level among all resources
      for (const usage of this.resourceUsage.values()) {
        if (
          usage.level === ResourceUsageLevel.CRITICAL ||
          (usage.level === ResourceUsageLevel.HIGH && usageLevel !== ResourceUsageLevel.CRITICAL) ||
          (usage.level === ResourceUsageLevel.MEDIUM && usageLevel !== ResourceUsageLevel.CRITICAL && usageLevel !== ResourceUsageLevel.HIGH)
        ) {
          usageLevel = usage.level;
        }
      }
    }
    
    // Apply throttling based on usage level
    switch (usageLevel) {
      case ResourceUsageLevel.CRITICAL:
        // Use both concurrency limiting and delay
        await new Promise(resolve => setTimeout(resolve, this.config.operationDelay * 2));
        return this.throttleWithConcurrencyLimit(operation);
        
      case ResourceUsageLevel.HIGH:
        // Use concurrency limiting
        return this.throttleWithConcurrencyLimit(operation);
        
      case ResourceUsageLevel.MEDIUM:
        // Use delay
        return this.throttleWithDelay(operation);
        
      case ResourceUsageLevel.LOW:
      default:
        // No throttling
        return operation();
    }
  }

  /**
   * Calculates an adaptive delay based on resource usage
   * 
   * @returns Delay in milliseconds
   */
  private calculateAdaptiveDelay(): number {
    let maxUsage = 0;
    
    // Find the maximum usage percentage across all resources
    for (const usage of this.resourceUsage.values()) {
      maxUsage = Math.max(maxUsage, usage.usage);
    }
    
    // Calculate delay based on usage
    // - Low usage (0-30%): minimal delay
    // - Medium usage (31-70%): moderate delay
    // - High usage (71-90%): significant delay
    // - Critical usage (91-100%): maximum delay
    if (maxUsage <= this.config.lowThreshold) {
      return this.config.operationDelay * 0.5;
    } else if (maxUsage <= this.config.mediumThreshold) {
      return this.config.operationDelay;
    } else if (maxUsage <= this.config.highThreshold) {
      return this.config.operationDelay * 2;
    } else {
      return this.config.operationDelay * 4;
    }
  }

  /**
   * Updates resource usage information
   */
  private async updateResourceUsage(): Promise<void> {
    for (const resource of this.config.resources) {
      const usage = await this.measureResourceUsage(resource);
      
      if (usage) {
        // Determine usage level
        let level: ResourceUsageLevel;
        if (usage.usage <= this.config.lowThreshold) {
          level = ResourceUsageLevel.LOW;
        } else if (usage.usage <= this.config.mediumThreshold) {
          level = ResourceUsageLevel.MEDIUM;
        } else if (usage.usage <= this.config.highThreshold) {
          level = ResourceUsageLevel.HIGH;
        } else {
          level = ResourceUsageLevel.CRITICAL;
        }
        
        // Update resource usage
        const resourceUsage: ResourceUsage = {
          ...usage,
          level,
        };
        
        this.resourceUsage.set(resource, resourceUsage);
        
        // Notify of resource usage change
        this.config.onResourceUsageChange(resourceUsage);
      }
    }
  }

  /**
   * Measures resource usage
   * 
   * @param resourceType - Resource type to measure
   * @returns Resource usage information
   */
  private async measureResourceUsage(resourceType: ResourceType): Promise<ResourceUsage | undefined> {
    switch (resourceType) {
      case ResourceType.CPU:
        return this.measureCpuUsage();
      case ResourceType.MEMORY:
        return this.measureMemoryUsage();
      case ResourceType.NETWORK:
        return this.measureNetworkUsage();
      case ResourceType.BATTERY:
        return this.measureBatteryUsage();
      case ResourceType.STORAGE:
        return this.measureStorageUsage();
      default:
        return undefined;
    }
  }

  /**
   * Measures CPU usage
   * 
   * @returns CPU usage information
   */
  private async measureCpuUsage(): Promise<ResourceUsage | undefined> {
    // CPU usage measurement is not directly available in browsers
    // We'll use a heuristic based on frame rate
    return new Promise(resolve => {
      let frameCount = 0;
      const startTime = performance.now();
      const expectedFrames = 60; // 60 fps is ideal
      
      const countFrame = () => {
        frameCount++;
        
        if (frameCount >= 10) {
          const endTime = performance.now();
          const elapsedTime = endTime - startTime;
          const fps = (frameCount / elapsedTime) * 1000;
          
          // Calculate CPU usage based on frame rate
          // Lower frame rate indicates higher CPU usage
          const usage = Math.max(0, Math.min(100, 100 - (fps / expectedFrames) * 100));
          
          resolve({
            type: ResourceType.CPU,
            usage,
            level: ResourceUsageLevel.LOW, // Will be updated by the caller
            available: 100 - usage,
            total: 100,
            details: { fps },
          });
        } else {
          requestAnimationFrame(countFrame);
        }
      };
      
      requestAnimationFrame(countFrame);
    });
  }

  /**
   * Measures memory usage
   * 
   * @returns Memory usage information
   */
  private async measureMemoryUsage(): Promise<ResourceUsage | undefined> {
    // Use performance.memory if available (Chrome only)
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      const used = memory.usedJSHeapSize;
      const total = memory.jsHeapSizeLimit;
      const usage = (used / total) * 100;
      
      return {
        type: ResourceType.MEMORY,
        usage,
        level: ResourceUsageLevel.LOW, // Will be updated by the caller
        available: total - used,
        total,
        details: {
          usedJSHeapSize: used,
          totalJSHeapSize: memory.totalJSHeapSize,
          jsHeapSizeLimit: total,
        },
      };
    }
    
    // Fallback to a simple heuristic
    return {
      type: ResourceType.MEMORY,
      usage: 50, // Default to medium usage
      level: ResourceUsageLevel.MEDIUM,
      available: 50,
      total: 100,
      details: { estimated: true },
    };
  }

  /**
   * Measures network usage
   * 
   * @returns Network usage information
   */
  private async measureNetworkUsage(): Promise<ResourceUsage | undefined> {
    // Use Navigator.connection if available
    if ('connection' in navigator) {
      const connection = (navigator as any).connection;
      
      // Estimate usage based on effective type
      let usage = 50; // Default to medium usage
      
      if (connection.effectiveType) {
        switch (connection.effectiveType) {
          case 'slow-2g':
            usage = 90;
            break;
          case '2g':
            usage = 75;
            break;
          case '3g':
            usage = 50;
            break;
          case '4g':
            usage = 25;
            break;
          default:
            usage = 50;
        }
      }
      
      // Adjust based on downlink if available
      if (connection.downlink) {
        // Higher downlink means lower usage
        usage = Math.max(0, usage - connection.downlink * 5);
      }
      
      return {
        type: ResourceType.NETWORK,
        usage,
        level: ResourceUsageLevel.LOW, // Will be updated by the caller
        available: 100 - usage,
        total: 100,
        details: {
          effectiveType: connection.effectiveType,
          downlink: connection.downlink,
          rtt: connection.rtt,
          saveData: connection.saveData,
        },
      };
    }
    
    // Fallback to a simple heuristic
    return {
      type: ResourceType.NETWORK,
      usage: 50, // Default to medium usage
      level: ResourceUsageLevel.MEDIUM,
      available: 50,
      total: 100,
      details: { estimated: true },
    };
  }

  /**
   * Measures battery usage
   * 
   * @returns Battery usage information
   */
  private async measureBatteryUsage(): Promise<ResourceUsage | undefined> {
    // Use Navigator.getBattery if available
    if ('getBattery' in navigator) {
      try {
        const battery = await (navigator as any).getBattery();
        const level = battery.level * 100;
        const usage = 100 - level;
        
        return {
          type: ResourceType.BATTERY,
          usage,
          level: ResourceUsageLevel.LOW, // Will be updated by the caller
          available: level,
          total: 100,
          details: {
            charging: battery.charging,
            chargingTime: battery.chargingTime,
            dischargingTime: battery.dischargingTime,
          },
        };
      } catch (error) {
        console.error('Error measuring battery usage:', error);
      }
    }
    
    // Fallback to a simple heuristic
    return {
      type: ResourceType.BATTERY,
      usage: 50, // Default to medium usage
      level: ResourceUsageLevel.MEDIUM,
      available: 50,
      total: 100,
      details: { estimated: true },
    };
  }

  /**
   * Measures storage usage
   * 
   * @returns Storage usage information
   */
  private async measureStorageUsage(): Promise<ResourceUsage | undefined> {
    // Use StorageManager if available
    if ('storage' in navigator && 'estimate' in navigator.storage) {
      try {
        const estimate = await navigator.storage.estimate();
        const usage = estimate.usage || 0;
        const quota = estimate.quota || 0;
        const usagePercentage = quota > 0 ? (usage / quota) * 100 : 50;
        
        return {
          type: ResourceType.STORAGE,
          usage: usagePercentage,
          level: ResourceUsageLevel.LOW, // Will be updated by the caller
          available: quota - usage,
          total: quota,
          details: {
            usageDetails: estimate.usageDetails,
          },
        };
      } catch (error) {
        console.error('Error measuring storage usage:', error);
      }
    }
    
    // Fallback to a simple heuristic
    return {
      type: ResourceType.STORAGE,
      usage: 50, // Default to medium usage
      level: ResourceUsageLevel.MEDIUM,
      available: 50,
      total: 100,
      details: { estimated: true },
    };
  }
}

/**
 * Global resource throttler instance
 */
export const globalResourceThrottler = new ResourceThrottler({
  strategy: ThrottlingStrategy.ADAPTIVE,
  resources: [ResourceType.CPU, ResourceType.MEMORY, ResourceType.NETWORK],
});

/**
 * Hook for using resource throttling in React components
 * 
 * @param config - Resource throttling configuration
 * @returns Resource throttler instance and utility functions
 */
export function useResourceThrottling(config?: Partial<ResourceThrottlingConfig>) {
  // Use the global throttler or create a new one
  const throttler = config 
    ? new ResourceThrottler({ ...config, strategy: config.strategy || ThrottlingStrategy.ADAPTIVE, resources: config.resources || [ResourceType.CPU, ResourceType.MEMORY] })
    : globalResourceThrottler;
  
  // Start monitoring if not already started
  if (!throttler['isMonitoring']) {
    throttler.startMonitoring();
  }
  
  return {
    throttler,
    
    /**
     * Throttles an operation based on resource usage
     * 
     * @param operation - Operation to throttle
     * @param options - Throttling options
     * @returns Promise that resolves to the operation result
     */
    throttle: <T>(
      operation: () => Promise<T> | T,
      options?: {
        priority?: number;
        resourceType?: ResourceType;
        bypassThrottling?: boolean;
      }
    ) => throttler.throttle(operation, options),
    
    /**
     * Gets the current resource usage
     * 
     * @param resourceType - Resource type to get usage for
     * @returns Resource usage information
     */
    getResourceUsage: (resourceType: ResourceType) => throttler.getResourceUsage(resourceType),
    
    /**
     * Gets all resource usage information
     * 
     * @returns Map of resource usage information
     */
    getAllResourceUsage: () => throttler.getAllResourceUsage(),
    
    /**
     * Updates the throttling configuration
     * 
     * @param newConfig - New throttling configuration
     */
    updateConfig: (newConfig: Partial<ResourceThrottlingConfig>) => throttler.updateConfig(newConfig),
  };
}