/**
 * Utilities for monitoring user experience metrics
 */

import { useState, useEffect } from 'react';

/**
 * User experience metrics collected during application usage
 */
export interface UXMetrics {
  /** Unique identifier for the metric */
  id: string;
  /** Timestamp when the metric was recorded */
  timestamp: number;
  /** User session identifier */
  sessionId: string;
  /** Page or component where the metric was recorded */
  location: string;
  /** Type of UX metric */
  metricType: UXMetricType;
  /** Metric value (interpretation depends on metricType) */
  value: number;
  /** Additional context for the metric */
  context?: Record<string, any>;
}

/**
 * Types of user experience metrics
 */
export enum UXMetricType {
  /** Time to first meaningful paint */
  TTFP = 'time_to_first_paint',
  /** Time to interactive */
  TTI = 'time_to_interactive',
  /** First input delay */
  FID = 'first_input_delay',
  /** Cumulative layout shift */
  CLS = 'cumulative_layout_shift',
  /** Largest contentful paint */
  LCP = 'largest_contentful_paint',
  /** User interaction time */
  INTERACTION_TIME = 'interaction_time',
  /** Page load time */
  PAGE_LOAD = 'page_load',
  /** Component render time */
  COMPONENT_RENDER = 'component_render',
  /** API response time */
  API_RESPONSE = 'api_response',
  /** User satisfaction score */
  SATISFACTION = 'satisfaction',
  /** Error encountered */
  ERROR = 'error',
  /** Custom metric */
  CUSTOM = 'custom'
}

/**
 * UX monitoring configuration
 */
export interface UXMonitoringConfig {
  /** Whether to enable detailed metrics collection */
  detailedMetrics?: boolean;
  /** Maximum number of metrics to keep in history */
  maxHistorySize?: number;
  /** Whether to persist metrics across page reloads */
  persistMetrics?: boolean;
  /** Storage key for persisted metrics */
  storageKey?: string;
  /** Callback when metrics are updated */
  onMetricsUpdate?: (metrics: UXMetrics) => void;
  /** Whether to automatically collect web vitals */
  autoCollectWebVitals?: boolean;
  /** Whether to automatically track page navigation */
  trackPageNavigation?: boolean;
  /** Whether to automatically track user interactions */
  trackUserInteractions?: boolean;
  /** Whether to automatically track API calls */
  trackApiCalls?: boolean;
  /** Sampling rate (0-1) to control how often metrics are collected */
  samplingRate?: number;
}

/**
 * UX monitoring statistics
 */
export interface UXMonitoringStats {
  /** Total number of metrics collected */
  totalMetrics: number;
  /** Average values by metric type */
  averagesByType: Partial<Record<UXMetricType, number>>;
  /** Metrics counts by type */
  countsByType: Partial<Record<UXMetricType, number>>;
  /** Metrics by location */
  metricsByLocation: Record<string, number>;
  /** Performance score (0-100) */
  performanceScore: number;
  /** User satisfaction score (0-100) */
  satisfactionScore: number;
  /** Error rate */
  errorRate: number;
  /** Metrics over time */
  trendsOverTime: {
    timeframe: string;
    values: Partial<Record<UXMetricType, number[]>>;
  };
}

/**
 * UX monitoring filter options
 */
export interface UXMonitoringFilter {
  /** Filter by metric type */
  metricType?: UXMetricType | UXMetricType[];
  /** Filter by location */
  location?: string | string[];
  /** Filter by session ID */
  sessionId?: string;
  /** Filter by time range */
  timeRange?: [number, number];
  /** Filter by minimum value */
  minValue?: number;
  /** Filter by maximum value */
  maxValue?: number;
  /** Filter by context properties */
  context?: Record<string, any>;
}

/**
 * UX monitoring manager
 */
export class UXMonitor {
  private config: Required<UXMonitoringConfig>;
  private metrics: UXMetrics[] = [];
  private sessionId: string;
  private listeners: Set<(metrics: UXMetrics) => void> = new Set();
  private navigationObserver?: any;
  private interactionObserver?: any;
  private apiCallObserver?: any;
  private webVitalsInitialized: boolean = false;

  /**
   * Creates a new UXMonitor instance
   * 
   * @param config - Monitoring configuration
   */
  constructor(config: UXMonitoringConfig = {}) {
    this.config = {
      detailedMetrics: config.detailedMetrics ?? true,
      maxHistorySize: config.maxHistorySize ?? 1000,
      persistMetrics: config.persistMetrics ?? true,
      storageKey: config.storageKey ?? 'ux-monitor-metrics',
      onMetricsUpdate: config.onMetricsUpdate ?? (() => {}),
      autoCollectWebVitals: config.autoCollectWebVitals ?? true,
      trackPageNavigation: config.trackPageNavigation ?? true,
      trackUserInteractions: config.trackUserInteractions ?? true,
      trackApiCalls: config.trackApiCalls ?? true,
      samplingRate: config.samplingRate ?? 1.0,
    };

    // Generate a session ID
    this.sessionId = this.generateSessionId();

    // Load persisted metrics if enabled
    if (this.config.persistMetrics) {
      this.loadPersistedMetrics();
    }
  }

  /**
   * Starts monitoring UX metrics
   */
  startMonitoring(): void {
    // Initialize web vitals collection if enabled
    if (this.config.autoCollectWebVitals && !this.webVitalsInitialized) {
      this.initWebVitals();
    }

    // Set up page navigation tracking if enabled
    if (this.config.trackPageNavigation) {
      this.setupNavigationTracking();
    }

    // Set up user interaction tracking if enabled
    if (this.config.trackUserInteractions) {
      this.setupInteractionTracking();
    }

    // Set up API call tracking if enabled
    if (this.config.trackApiCalls) {
      this.setupApiCallTracking();
    }
  }

  /**
   * Stops monitoring UX metrics
   */
  stopMonitoring(): void {
    // Clean up navigation observer
    if (this.navigationObserver) {
      // Cleanup logic would depend on implementation
      this.navigationObserver = undefined;
    }

    // Clean up interaction observer
    if (this.interactionObserver) {
      document.removeEventListener('click', this.interactionObserver);
      document.removeEventListener('input', this.interactionObserver);
      this.interactionObserver = undefined;
    }

    // Clean up API call observer
    if (this.apiCallObserver) {
      // Cleanup logic would depend on implementation
      this.apiCallObserver = undefined;
    }
  }

  /**
   * Records a UX metric
   * 
   * @param metricType - Type of metric
   * @param value - Metric value
   * @param location - Location where the metric was recorded
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordMetric(
    metricType: UXMetricType,
    value: number,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    // Apply sampling rate
    if (Math.random() > this.config.samplingRate) {
      // Create a dummy metric object to return but don't store it
      return {
        id: this.generateId(),
        timestamp: Date.now(),
        sessionId: this.sessionId,
        location,
        metricType,
        value,
        context
      };
    }

    const metric: UXMetrics = {
      id: this.generateId(),
      timestamp: Date.now(),
      sessionId: this.sessionId,
      location,
      metricType,
      value,
      context
    };

    // Add to metrics collection
    this.metrics.push(metric);

    // Trim metrics if they exceed the maximum size
    if (this.metrics.length > this.config.maxHistorySize) {
      this.metrics = this.metrics.slice(this.metrics.length - this.config.maxHistorySize);
    }

    // Notify listeners
    this.notifyMetricsUpdate(metric);

    // Persist metrics if enabled
    if (this.config.persistMetrics) {
      this.persistMetrics();
    }

    return metric;
  }

  /**
   * Gets all collected metrics
   * 
   * @param filter - Filter options
   * @returns Array of metrics
   */
  getMetrics(filter?: UXMonitoringFilter): UXMetrics[] {
    if (!filter) {
      return [...this.metrics];
    }

    return this.metrics.filter(metric => this.filterMetric(metric, filter));
  }

  /**
   * Gets UX monitoring statistics
   * 
   * @returns UX monitoring statistics
   */
  getStatistics(): UXMonitoringStats {
    const metrics = this.metrics;
    
    // Calculate counts by type
    const countsByType: Partial<Record<UXMetricType, number>> = {};
    for (const metric of metrics) {
      countsByType[metric.metricType] = (countsByType[metric.metricType] || 0) + 1;
    }
    
    // Calculate averages by type
    const averagesByType: Partial<Record<UXMetricType, number>> = {};
    for (const type of Object.values(UXMetricType)) {
      const typeMetrics = metrics.filter(m => m.metricType === type);
      if (typeMetrics.length > 0) {
        const sum = typeMetrics.reduce((acc, m) => acc + m.value, 0);
        averagesByType[type] = sum / typeMetrics.length;
      }
    }
    
    // Count metrics by location
    const metricsByLocation: Record<string, number> = {};
    for (const metric of metrics) {
      metricsByLocation[metric.location] = (metricsByLocation[metric.location] || 0) + 1;
    }
    
    // Calculate error rate
    const errorMetrics = metrics.filter(m => m.metricType === UXMetricType.ERROR);
    const errorRate = metrics.length > 0 ? errorMetrics.length / metrics.length : 0;
    
    // Calculate performance score (simplified example)
    let performanceScore = 100;
    
    // Penalize for slow page loads
    const pageLoadMetrics = metrics.filter(m => m.metricType === UXMetricType.PAGE_LOAD);
    if (pageLoadMetrics.length > 0) {
      const avgPageLoad = pageLoadMetrics.reduce((acc, m) => acc + m.value, 0) / pageLoadMetrics.length;
      // Penalize 1 point for every 100ms over 1000ms
      performanceScore -= Math.max(0, Math.min(20, Math.floor((avgPageLoad - 1000) / 100)));
    }
    
    // Penalize for slow API responses
    const apiMetrics = metrics.filter(m => m.metricType === UXMetricType.API_RESPONSE);
    if (apiMetrics.length > 0) {
      const avgApiResponse = apiMetrics.reduce((acc, m) => acc + m.value, 0) / apiMetrics.length;
      // Penalize 1 point for every 50ms over 200ms
      performanceScore -= Math.max(0, Math.min(20, Math.floor((avgApiResponse - 200) / 50)));
    }
    
    // Penalize for layout shifts
    const clsMetrics = metrics.filter(m => m.metricType === UXMetricType.CLS);
    if (clsMetrics.length > 0) {
      const avgCls = clsMetrics.reduce((acc, m) => acc + m.value, 0) / clsMetrics.length;
      // Penalize up to 20 points based on CLS (0.1 is a good score, 0.25+ is poor)
      performanceScore -= Math.max(0, Math.min(20, Math.floor(avgCls * 100)));
    }
    
    // Calculate satisfaction score
    const satisfactionMetrics = metrics.filter(m => m.metricType === UXMetricType.SATISFACTION);
    const satisfactionScore = satisfactionMetrics.length > 0
      ? (satisfactionMetrics.reduce((acc, m) => acc + m.value, 0) / satisfactionMetrics.length) * 100
      : 75; // Default to 75 if no satisfaction metrics
    
    // Calculate trends over time (simplified)
    // Group metrics by day for the last 7 days
    const now = Date.now();
    const oneDay = 24 * 60 * 60 * 1000;
    const days = Array.from({ length: 7 }, (_, i) => now - (6 - i) * oneDay);
    
    const trendsOverTime = {
      timeframe: 'last7days',
      values: {} as Partial<Record<UXMetricType, number[]>>
    };
    
    for (const type of Object.values(UXMetricType)) {
      trendsOverTime.values[type] = days.map(day => {
        const dayStart = new Date(day).setHours(0, 0, 0, 0);
        const dayEnd = dayStart + oneDay;
        const dayMetrics = metrics.filter(
          m => m.metricType === type && m.timestamp >= dayStart && m.timestamp < dayEnd
        );
        
        if (dayMetrics.length === 0) return 0;
        return dayMetrics.reduce((acc, m) => acc + m.value, 0) / dayMetrics.length;
      });
    }
    
    return {
      totalMetrics: metrics.length,
      averagesByType,
      countsByType,
      metricsByLocation,
      performanceScore,
      satisfactionScore,
      errorRate,
      trendsOverTime
    };
  }

  /**
   * Adds a listener for metrics updates
   * 
   * @param listener - Listener function
   * @returns Function to remove the listener
   */
  addListener(listener: (metrics: UXMetrics) => void): () => void {
    this.listeners.add(listener);
    
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Clears all metrics
   */
  clearMetrics(): void {
    this.metrics = [];
    
    if (this.config.persistMetrics) {
      this.persistMetrics();
    }
  }

  /**
   * Records user satisfaction
   * 
   * @param score - Satisfaction score (0-1)
   * @param location - Location where the score was recorded
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordSatisfaction(
    score: number,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    return this.recordMetric(UXMetricType.SATISFACTION, score, location, context);
  }

  /**
   * Records an error
   * 
   * @param error - Error object or message
   * @param location - Location where the error occurred
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordError(
    error: Error | string,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    const errorMessage = error instanceof Error ? error.message : error;
    const errorContext = {
      ...(context || {}),
      errorMessage,
      stack: error instanceof Error ? error.stack : undefined
    };
    
    return this.recordMetric(UXMetricType.ERROR, 1, location, errorContext);
  }

  /**
   * Records page load time
   * 
   * @param loadTime - Load time in milliseconds
   * @param location - Page location
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordPageLoad(
    loadTime: number,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    return this.recordMetric(UXMetricType.PAGE_LOAD, loadTime, location, context);
  }

  /**
   * Records API response time
   * 
   * @param responseTime - Response time in milliseconds
   * @param endpoint - API endpoint
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordApiResponse(
    responseTime: number,
    endpoint: string,
    context?: Record<string, any>
  ): UXMetrics {
    return this.recordMetric(UXMetricType.API_RESPONSE, responseTime, endpoint, context);
  }

  /**
   * Records component render time
   * 
   * @param renderTime - Render time in milliseconds
   * @param componentName - Component name
   * @param location - Page location
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordComponentRender(
    renderTime: number,
    componentName: string,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    const fullContext = {
      ...(context || {}),
      componentName
    };
    
    return this.recordMetric(UXMetricType.COMPONENT_RENDER, renderTime, location, fullContext);
  }

  /**
   * Records user interaction time
   * 
   * @param interactionTime - Interaction time in milliseconds
   * @param interactionType - Type of interaction
   * @param location - Page location
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordInteraction(
    interactionTime: number,
    interactionType: string,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    const fullContext = {
      ...(context || {}),
      interactionType
    };
    
    return this.recordMetric(UXMetricType.INTERACTION_TIME, interactionTime, location, fullContext);
  }

  /**
   * Records a custom metric
   * 
   * @param name - Metric name
   * @param value - Metric value
   * @param location - Location
   * @param context - Additional context
   * @returns The recorded metric
   */
  recordCustomMetric(
    name: string,
    value: number,
    location: string = window.location.pathname,
    context?: Record<string, any>
  ): UXMetrics {
    const fullContext = {
      ...(context || {}),
      metricName: name
    };
    
    return this.recordMetric(UXMetricType.CUSTOM, value, location, fullContext);
  }

  /**
   * Initializes web vitals collection
   */
  private initWebVitals(): void {
    // In a real implementation, this would use the web-vitals library
    // For this example, we'll simulate it with some basic metrics
    
    this.webVitalsInitialized = true;
    
    // Record Time to First Paint (simulated)
    setTimeout(() => {
      this.recordMetric(UXMetricType.TTFP, Math.random() * 500 + 100);
    }, 0);
    
    // Record Time to Interactive (simulated)
    setTimeout(() => {
      this.recordMetric(UXMetricType.TTI, Math.random() * 1000 + 500);
    }, 0);
    
    // Record Largest Contentful Paint (simulated)
    setTimeout(() => {
      this.recordMetric(UXMetricType.LCP, Math.random() * 1500 + 800);
    }, 0);
    
    // Record Cumulative Layout Shift (simulated)
    setTimeout(() => {
      this.recordMetric(UXMetricType.CLS, Math.random() * 0.2);
    }, 0);
    
    // Record First Input Delay (simulated)
    const fidHandler = () => {
      this.recordMetric(UXMetricType.FID, Math.random() * 100 + 10);
      document.removeEventListener('click', fidHandler);
      document.removeEventListener('keydown', fidHandler);
    };
    
    document.addEventListener('click', fidHandler, { once: true });
    document.addEventListener('keydown', fidHandler, { once: true });
  }

  /**
   * Sets up page navigation tracking
   */
  private setupNavigationTracking(): void {
    // In a real implementation, this would use the History API or a router's navigation events
    // For this example, we'll use a simple approach
    
    const originalPushState = history.pushState;
    const originalReplaceState = history.replaceState;
    
    // Record initial page load
    this.recordPageLoad(
      performance.now(),
      window.location.pathname
    );
    
    // Track navigation via history API
    history.pushState = (...args) => {
      originalPushState.apply(history, args);
      this.handleNavigation();
    };
    
    history.replaceState = (...args) => {
      originalReplaceState.apply(history, args);
      this.handleNavigation();
    };
    
    // Track navigation via popstate event
    window.addEventListener('popstate', () => {
      this.handleNavigation();
    });
    
    this.navigationObserver = {
      cleanup: () => {
        history.pushState = originalPushState;
        history.replaceState = originalReplaceState;
        window.removeEventListener('popstate', this.handleNavigation);
      }
    };
  }

  /**
   * Handles a navigation event
   */
  private handleNavigation(): void {
    const location = window.location.pathname;
    
    // Record page load time (simulated for this example)
    this.recordPageLoad(
      Math.random() * 500 + 100,
      location
    );
  }

  /**
   * Sets up user interaction tracking
   */
  private setupInteractionTracking(): void {
    // Track clicks
    const clickHandler = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target) return;
      
      const tagName = target.tagName.toLowerCase();
      const id = target.id;
      const classes = Array.from(target.classList).join(' ');
      
      this.recordInteraction(
        0, // We don't have a duration for clicks
        'click',
        window.location.pathname,
        {
          elementType: tagName,
          elementId: id || undefined,
          elementClass: classes || undefined,
          x: e.clientX,
          y: e.clientY
        }
      );
    };
    
    // Track input interactions
    const inputStartTimes = new WeakMap<HTMLElement, number>();
    
    const inputStartHandler = (e: Event) => {
      const target = e.target as HTMLElement;
      if (!target) return;
      
      inputStartTimes.set(target, performance.now());
    };
    
    const inputEndHandler = (e: Event) => {
      const target = e.target as HTMLElement;
      if (!target) return;
      
      const startTime = inputStartTimes.get(target);
      if (!startTime) return;
      
      const duration = performance.now() - startTime;
      const tagName = target.tagName.toLowerCase();
      const id = target.id;
      const type = (target as HTMLInputElement).type || undefined;
      
      this.recordInteraction(
        duration,
        'input',
        window.location.pathname,
        {
          elementType: tagName,
          elementId: id || undefined,
          inputType: type
        }
      );
      
      inputStartTimes.delete(target);
    };
    
    document.addEventListener('click', clickHandler);
    document.addEventListener('focusin', inputStartHandler);
    document.addEventListener('focusout', inputEndHandler);
    
    this.interactionObserver = {
      cleanup: () => {
        document.removeEventListener('click', clickHandler);
        document.removeEventListener('focusin', inputStartHandler);
        document.removeEventListener('focusout', inputEndHandler);
      }
    };
  }

  /**
   * Sets up API call tracking
   */
  private setupApiCallTracking(): void {
    // In a real implementation, this would intercept fetch/XHR calls
    // For this example, we'll create a simple wrapper for fetch
    
    const originalFetch = window.fetch;
    
    window.fetch = async (input: RequestInfo, init?: RequestInit) => {
      const startTime = performance.now();
      let response: Response;
      let error: Error | undefined;
      
      try {
        response = await originalFetch(input, init);
      } catch (e) {
        error = e as Error;
        throw e;
      } finally {
        const endTime = performance.now();
        const duration = endTime - startTime;
        
        const url = typeof input === 'string' ? input : input.url;
        const method = init?.method || 'GET';
        
        if (error) {
          this.recordError(
            error,
            url,
            {
              method,
              duration,
              apiCall: true
            }
          );
        } else {
          this.recordApiResponse(
            duration,
            url,
            {
              method,
              status: response!.status
            }
          );
        }
      }
      
      return response!;
    };
    
    this.apiCallObserver = {
      cleanup: () => {
        window.fetch = originalFetch;
      }
    };
  }

  /**
   * Notifies listeners of metrics updates
   * 
   * @param metrics - Updated metrics
   */
  private notifyMetricsUpdate(metrics: UXMetrics): void {
    // Call the onMetricsUpdate callback
    this.config.onMetricsUpdate(metrics);
    
    // Notify all listeners
    for (const listener of this.listeners) {
      listener(metrics);
    }
  }

  /**
   * Filters metrics based on filter options
   * 
   * @param metric - Metric to filter
   * @param filter - Filter options
   * @returns Whether the metric matches the filter
   */
  private filterMetric(metric: UXMetrics, filter: UXMonitoringFilter): boolean {
    // Filter by metric type
    if (filter.metricType) {
      if (Array.isArray(filter.metricType)) {
        if (!filter.metricType.includes(metric.metricType)) {
          return false;
        }
      } else if (metric.metricType !== filter.metricType) {
        return false;
      }
    }
    
    // Filter by location
    if (filter.location) {
      if (Array.isArray(filter.location)) {
        if (!filter.location.includes(metric.location)) {
          return false;
        }
      } else if (metric.location !== filter.location) {
        return false;
      }
    }
    
    // Filter by session ID
    if (filter.sessionId && metric.sessionId !== filter.sessionId) {
      return false;
    }
    
    // Filter by time range
    if (filter.timeRange) {
      const [min, max] = filter.timeRange;
      if (metric.timestamp < min || metric.timestamp > max) {
        return false;
      }
    }
    
    // Filter by value range
    if (filter.minValue !== undefined && metric.value < filter.minValue) {
      return false;
    }
    
    if (filter.maxValue !== undefined && metric.value > filter.maxValue) {
      return false;
    }
    
    // Filter by context properties
    if (filter.context && metric.context) {
      for (const [key, value] of Object.entries(filter.context)) {
        if (metric.context[key] !== value) {
          return false;
        }
      }
    }
    
    return true;
  }

  /**
   * Persists metrics to storage
   */
  private persistMetrics(): void {
    if (typeof window === 'undefined' || !window.localStorage) {
      return;
    }
    
    try {
      localStorage.setItem(this.config.storageKey, JSON.stringify(this.metrics));
    } catch (error) {
      console.error('Failed to persist UX metrics:', error);
    }
  }

  /**
   * Loads persisted metrics from storage
   */
  private loadPersistedMetrics(): void {
    if (typeof window === 'undefined' || !window.localStorage) {
      return;
    }
    
    try {
      const persistedMetricsJson = localStorage.getItem(this.config.storageKey);
      
      if (!persistedMetricsJson) {
        return;
      }
      
      const persistedMetrics = JSON.parse(persistedMetricsJson);
      
      if (Array.isArray(persistedMetrics)) {
        this.metrics = persistedMetrics;
      }
    } catch (error) {
      console.error('Failed to load persisted UX metrics:', error);
    }
  }

  /**
   * Generates a unique ID
   * 
   * @returns Unique ID
   */
  private generateId(): string {
    return Math.random().toString(36).substring(2, 15) + 
           Math.random().toString(36).substring(2, 15);
  }

  /**
   * Generates a session ID
   * 
   * @returns Session ID
   */
  private generateSessionId(): string {
    return 'session_' + new Date().toISOString().replace(/[-:.TZ]/g, '') + 
           '_' + Math.random().toString(36).substring(2, 9);
  }
}

/**
 * Global UX monitor instance
 */
export const globalUXMonitor = new UXMonitor();

/**
 * Hook for using UX monitoring in React components
 * 
 * @param config - UX monitoring configuration
 * @returns UX monitor instance and utility functions
 */
export function useUXMonitoring(config?: UXMonitoringConfig) {
  // Use the global monitor or create a new one
  const monitor = config ? new UXMonitor(config) : globalUXMonitor;
  
  // Start monitoring when the component mounts
  useEffect(() => {
    monitor.startMonitoring();
    
    return () => {
      // No need to stop the global monitor when a component unmounts
      if (config) {
        monitor.stopMonitoring();
      }
    };
  }, [monitor, config]);
  
  // State for component-specific metrics
  const [componentMetrics, setComponentMetrics] = useState<UXMetrics[]>([]);
  
  // Update component metrics when new metrics are recorded
  useEffect(() => {
    const updateComponentMetrics = () => {
      const location = window.location.pathname;
      const metrics = monitor.getMetrics({
        location
      });
      setComponentMetrics(metrics);
    };
    
    const removeListener = monitor.addListener(() => {
      updateComponentMetrics();
    });
    
    // Initial update
    updateComponentMetrics();
    
    return removeListener;
  }, [monitor]);
  
  return {
    monitor,
    
    /**
     * Gets all collected metrics
     * 
     * @param filter - Filter options
     * @returns Array of metrics
     */
    getMetrics: (filter?: UXMonitoringFilter) => monitor.getMetrics(filter),
    
    /**
     * Gets UX monitoring statistics
     * 
     * @returns UX monitoring statistics
     */
    getStatistics: () => monitor.getStatistics(),
    
    /**
     * Adds a listener for metrics updates
     * 
     * @param listener - Listener function
     * @returns Function to remove the listener
     */
    addListener: (listener: (metrics: UXMetrics) => void) => monitor.addListener(listener),
    
    /**
     * Clears all metrics
     */
    clearMetrics: () => monitor.clearMetrics(),
    
    /**
     * Records user satisfaction
     * 
     * @param score - Satisfaction score (0-1)
     * @param context - Additional context
     * @returns The recorded metric
     */
    recordSatisfaction: (score: number, context?: Record<string, any>) => 
      monitor.recordSatisfaction(score, window.location.pathname, context),
    
    /**
     * Records an error
     * 
     * @param error - Error object or message
     * @param context - Additional context
     * @returns The recorded metric
     */
    recordError: (error: Error | string, context?: Record<string, any>) => 
      monitor.recordError(error, window.location.pathname, context),
    
    /**
     * Records component render time
     * 
     * @param renderTime - Render time in milliseconds
     * @param componentName - Component name
     * @param context - Additional context
     * @returns The recorded metric
     */
    recordComponentRender: (renderTime: number, componentName: string, context?: Record<string, any>) => 
      monitor.recordComponentRender(renderTime, componentName, window.location.pathname, context),
    
    /**
     * Records a custom metric
     * 
     * @param name - Metric name
     * @param value - Metric value
     * @param context - Additional context
     * @returns The recorded metric
     */
    recordCustomMetric: (name: string, value: number, context?: Record<string, any>) => 
      monitor.recordCustomMetric(name, value, window.location.pathname, context),
    
    /**
     * Component-specific metrics
     */
    componentMetrics
  };
}