/**
 * Utilities for monitoring background tasks
 */

import { Task, TaskStatus, TaskPriority } from './taskScheduler';
import { ResourceType, ResourceUsage } from './resourceThrottling';

/**
 * Task metrics collected during execution
 */
export interface TaskMetrics {
  /** Task ID */
  taskId: string;
  /** Task name */
  taskName: string;
  /** Task status */
  status: TaskStatus;
  /** Task priority */
  priority: TaskPriority;
  /** Task progress (0-100) */
  progress: number;
  /** Task duration in milliseconds */
  duration: number;
  /** Task start time */
  startTime?: number;
  /** Task end time */
  endTime?: number;
  /** Number of retries */
  retries: number;
  /** Resource usage during task execution */
  resourceUsage?: Partial<Record<ResourceType, number>>;
  /** Error message if task failed */
  error?: string;
  /** Additional task-specific metrics */
  customMetrics?: Record<string, any>;
}

/**
 * Task monitoring configuration
 */
export interface TaskMonitoringConfig {
  /** Whether to enable detailed metrics collection */
  detailedMetrics?: boolean;
  /** Maximum number of task metrics to keep in history */
  maxHistorySize?: number;
  /** Interval for collecting metrics in milliseconds */
  metricsInterval?: number;
  /** Whether to persist metrics across page reloads */
  persistMetrics?: boolean;
  /** Storage key for persisted metrics */
  storageKey?: string;
  /** Callback when task metrics are updated */
  onMetricsUpdate?: (metrics: TaskMetrics) => void;
  /** Resource types to monitor during task execution */
  monitoredResources?: ResourceType[];
}

/**
 * Task monitoring statistics
 */
export interface TaskMonitoringStats {
  /** Total number of tasks */
  totalTasks: number;
  /** Number of completed tasks */
  completedTasks: number;
  /** Number of failed tasks */
  failedTasks: number;
  /** Number of running tasks */
  runningTasks: number;
  /** Number of pending tasks */
  pendingTasks: number;
  /** Number of canceled tasks */
  canceledTasks: number;
  /** Number of paused tasks */
  pausedTasks: number;
  /** Average task duration in milliseconds */
  averageDuration: number;
  /** Average task progress */
  averageProgress: number;
  /** Task completion rate (0-1) */
  completionRate: number;
  /** Task failure rate (0-1) */
  failureRate: number;
  /** Resource usage statistics */
  resourceUsage: Partial<Record<ResourceType, { average: number; peak: number }>>;
  /** Task counts by priority */
  tasksByPriority: Record<TaskPriority, number>;
  /** Task counts by status */
  tasksByStatus: Record<TaskStatus, number>;
}

/**
 * Task monitoring filter options
 */
export interface TaskMonitoringFilter {
  /** Filter by task status */
  status?: TaskStatus | TaskStatus[];
  /** Filter by task priority */
  priority?: TaskPriority | TaskPriority[];
  /** Filter by task name */
  name?: string;
  /** Filter by task ID */
  id?: string;
  /** Filter by minimum progress */
  minProgress?: number;
  /** Filter by maximum progress */
  maxProgress?: number;
  /** Filter by minimum duration */
  minDuration?: number;
  /** Filter by maximum duration */
  maxDuration?: number;
  /** Filter by start time range */
  startTimeRange?: [number, number];
  /** Filter by end time range */
  endTimeRange?: [number, number];
  /** Filter by custom metrics */
  customMetrics?: Record<string, any>;
}

/**
 * Task monitoring manager
 */
export class TaskMonitor {
  private config: Required<TaskMonitoringConfig>;
  private taskMetrics: Map<string, TaskMetrics> = new Map();
  private taskHistory: TaskMetrics[] = [];
  private metricsIntervalId?: number;
  private isMonitoring: boolean = false;
  private listeners: Set<(metrics: TaskMetrics) => void> = new Set();

  /**
   * Creates a new TaskMonitor instance
   * 
   * @param config - Monitoring configuration
   */
  constructor(config: TaskMonitoringConfig = {}) {
    this.config = {
      detailedMetrics: config.detailedMetrics ?? true,
      maxHistorySize: config.maxHistorySize ?? 100,
      metricsInterval: config.metricsInterval ?? 1000,
      persistMetrics: config.persistMetrics ?? false,
      storageKey: config.storageKey ?? 'task-monitor-metrics',
      onMetricsUpdate: config.onMetricsUpdate ?? (() => {}),
      monitoredResources: config.monitoredResources ?? [ResourceType.CPU, ResourceType.MEMORY],
    };

    // Load persisted metrics if enabled
    if (this.config.persistMetrics) {
      this.loadPersistedMetrics();
    }
  }

  /**
   * Starts monitoring tasks
   */
  startMonitoring(): void {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;

    // Set up interval for collecting metrics
    this.metricsIntervalId = window.setInterval(() => {
      this.updateMetrics();
    }, this.config.metricsInterval);
  }

  /**
   * Stops monitoring tasks
   */
  stopMonitoring(): void {
    if (!this.isMonitoring) {
      return;
    }

    this.isMonitoring = false;

    if (this.metricsIntervalId !== undefined) {
      clearInterval(this.metricsIntervalId);
      this.metricsIntervalId = undefined;
    }
  }

  /**
   * Registers a task for monitoring
   * 
   * @param task - Task to monitor
   */
  registerTask(task: Task): void {
    const metrics: TaskMetrics = {
      taskId: task.id,
      taskName: task.name,
      status: task.status,
      priority: task.options.priority,
      progress: task.progress,
      duration: 0,
      startTime: task.startedAt,
      endTime: task.completedAt,
      retries: task.retryCount,
      error: task.error?.message,
      resourceUsage: {},
      customMetrics: {},
    };

    this.taskMetrics.set(task.id, metrics);
    this.notifyMetricsUpdate(metrics);
  }

  /**
   * Updates metrics for a task
   * 
   * @param task - Task to update metrics for
   * @param resourceUsage - Resource usage during task execution
   * @param customMetrics - Additional task-specific metrics
   */
  updateTaskMetrics(
    task: Task,
    resourceUsage?: Partial<Record<ResourceType, number>>,
    customMetrics?: Record<string, any>
  ): void {
    const existingMetrics = this.taskMetrics.get(task.id);
    
    if (!existingMetrics) {
      this.registerTask(task);
      return;
    }

    // Update basic metrics
    existingMetrics.status = task.status;
    existingMetrics.progress = task.progress;
    existingMetrics.retries = task.retryCount;
    existingMetrics.error = task.error?.message;
    
    // Update timestamps
    if (task.startedAt) {
      existingMetrics.startTime = task.startedAt;
    }
    
    if (task.completedAt) {
      existingMetrics.endTime = task.completedAt;
    }
    
    // Calculate duration
    if (existingMetrics.startTime) {
      existingMetrics.duration = (existingMetrics.endTime || Date.now()) - existingMetrics.startTime;
    }
    
    // Update resource usage if provided
    if (resourceUsage) {
      existingMetrics.resourceUsage = {
        ...existingMetrics.resourceUsage,
        ...resourceUsage,
      };
    }
    
    // Update custom metrics if provided
    if (customMetrics) {
      existingMetrics.customMetrics = {
        ...existingMetrics.customMetrics,
        ...customMetrics,
      };
    }
    
    // Notify listeners
    this.notifyMetricsUpdate(existingMetrics);
    
    // Add to history if task is completed, failed, or canceled
    if (
      task.status === TaskStatus.COMPLETED ||
      task.status === TaskStatus.FAILED ||
      task.status === TaskStatus.CANCELED
    ) {
      this.addToHistory(existingMetrics);
    }
  }

  /**
   * Gets metrics for a specific task
   * 
   * @param taskId - Task ID
   * @returns Task metrics or undefined if not found
   */
  getTaskMetrics(taskId: string): TaskMetrics | undefined {
    return this.taskMetrics.get(taskId);
  }

  /**
   * Gets metrics for all tasks
   * 
   * @returns Map of task metrics
   */
  getAllTaskMetrics(): Map<string, TaskMetrics> {
    return new Map(this.taskMetrics);
  }

  /**
   * Gets task metrics history
   * 
   * @param filter - Filter options
   * @returns Array of task metrics
   */
  getTaskHistory(filter?: TaskMonitoringFilter): TaskMetrics[] {
    if (!filter) {
      return [...this.taskHistory];
    }
    
    return this.taskHistory.filter(metrics => this.filterMetrics(metrics, filter));
  }

  /**
   * Gets task monitoring statistics
   * 
   * @returns Task monitoring statistics
   */
  getStatistics(): TaskMonitoringStats {
    const metrics = Array.from(this.taskMetrics.values());
    const completedTasks = metrics.filter(m => m.status === TaskStatus.COMPLETED).length;
    const failedTasks = metrics.filter(m => m.status === TaskStatus.FAILED).length;
    const runningTasks = metrics.filter(m => m.status === TaskStatus.RUNNING).length;
    const pendingTasks = metrics.filter(m => m.status === TaskStatus.PENDING).length;
    const canceledTasks = metrics.filter(m => m.status === TaskStatus.CANCELED).length;
    const pausedTasks = metrics.filter(m => m.status === TaskStatus.PAUSED).length;
    
    // Calculate average duration for completed tasks
    const completedMetrics = metrics.filter(m => 
      m.status === TaskStatus.COMPLETED || 
      m.status === TaskStatus.FAILED || 
      m.status === TaskStatus.CANCELED
    );
    
    const totalDuration = completedMetrics.reduce((sum, m) => sum + m.duration, 0);
    const averageDuration = completedMetrics.length > 0 ? totalDuration / completedMetrics.length : 0;
    
    // Calculate average progress
    const totalProgress = metrics.reduce((sum, m) => sum + m.progress, 0);
    const averageProgress = metrics.length > 0 ? totalProgress / metrics.length : 0;
    
    // Calculate completion and failure rates
    const totalFinishedTasks = completedTasks + failedTasks + canceledTasks;
    const completionRate = totalFinishedTasks > 0 ? completedTasks / totalFinishedTasks : 0;
    const failureRate = totalFinishedTasks > 0 ? failedTasks / totalFinishedTasks : 0;
    
    // Calculate resource usage statistics
    const resourceUsage: Partial<Record<ResourceType, { average: number; peak: number }>> = {};
    
    for (const resourceType of this.config.monitoredResources) {
      const resourceMetrics = metrics
        .filter(m => m.resourceUsage && m.resourceUsage[resourceType] !== undefined)
        .map(m => m.resourceUsage![resourceType]!);
      
      if (resourceMetrics.length > 0) {
        const total = resourceMetrics.reduce((sum, usage) => sum + usage, 0);
        const average = total / resourceMetrics.length;
        const peak = Math.max(...resourceMetrics);
        
        resourceUsage[resourceType] = { average, peak };
      }
    }
    
    // Count tasks by priority
    const tasksByPriority = Object.values(TaskPriority)
      .filter(p => typeof p === 'number')
      .reduce((acc, priority) => {
        acc[priority as TaskPriority] = metrics.filter(m => m.priority === priority).length;
        return acc;
      }, {} as Record<TaskPriority, number>);
    
    // Count tasks by status
    const tasksByStatus = Object.values(TaskStatus)
      .filter(s => typeof s === 'string')
      .reduce((acc, status) => {
        acc[status as TaskStatus] = metrics.filter(m => m.status === status).length;
        return acc;
      }, {} as Record<TaskStatus, number>);
    
    return {
      totalTasks: metrics.length,
      completedTasks,
      failedTasks,
      runningTasks,
      pendingTasks,
      canceledTasks,
      pausedTasks,
      averageDuration,
      averageProgress,
      completionRate,
      failureRate,
      resourceUsage,
      tasksByPriority,
      tasksByStatus,
    };
  }

  /**
   * Adds a listener for metrics updates
   * 
   * @param listener - Listener function
   * @returns Function to remove the listener
   */
  addListener(listener: (metrics: TaskMetrics) => void): () => void {
    this.listeners.add(listener);
    
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Clears all metrics
   */
  clearMetrics(): void {
    this.taskMetrics.clear();
    this.taskHistory = [];
    
    if (this.config.persistMetrics) {
      this.persistMetrics();
    }
  }

  /**
   * Updates metrics for all tasks
   */
  private updateMetrics(): void {
    // This method would typically be called by the task scheduler
    // to update metrics for all running tasks
    
    // For now, we'll just update durations for running tasks
    for (const [taskId, metrics] of this.taskMetrics.entries()) {
      if (metrics.status === TaskStatus.RUNNING && metrics.startTime) {
        metrics.duration = Date.now() - metrics.startTime;
        this.notifyMetricsUpdate(metrics);
      }
    }
  }

  /**
   * Adds metrics to history
   * 
   * @param metrics - Task metrics to add to history
   */
  private addToHistory(metrics: TaskMetrics): void {
    // Create a copy of the metrics
    const historicalMetrics = { ...metrics };
    
    // Add to history
    this.taskHistory.unshift(historicalMetrics);
    
    // Trim history if it exceeds the maximum size
    if (this.taskHistory.length > this.config.maxHistorySize) {
      this.taskHistory = this.taskHistory.slice(0, this.config.maxHistorySize);
    }
    
    // Persist metrics if enabled
    if (this.config.persistMetrics) {
      this.persistMetrics();
    }
  }

  /**
   * Notifies listeners of metrics updates
   * 
   * @param metrics - Updated task metrics
   */
  private notifyMetricsUpdate(metrics: TaskMetrics): void {
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
   * @param metrics - Task metrics to filter
   * @param filter - Filter options
   * @returns Whether the metrics match the filter
   */
  private filterMetrics(metrics: TaskMetrics, filter: TaskMonitoringFilter): boolean {
    // Filter by status
    if (filter.status) {
      if (Array.isArray(filter.status)) {
        if (!filter.status.includes(metrics.status)) {
          return false;
        }
      } else if (metrics.status !== filter.status) {
        return false;
      }
    }
    
    // Filter by priority
    if (filter.priority) {
      if (Array.isArray(filter.priority)) {
        if (!filter.priority.includes(metrics.priority)) {
          return false;
        }
      } else if (metrics.priority !== filter.priority) {
        return false;
      }
    }
    
    // Filter by name
    if (filter.name && !metrics.taskName.includes(filter.name)) {
      return false;
    }
    
    // Filter by ID
    if (filter.id && metrics.taskId !== filter.id) {
      return false;
    }
    
    // Filter by progress
    if (filter.minProgress !== undefined && metrics.progress < filter.minProgress) {
      return false;
    }
    
    if (filter.maxProgress !== undefined && metrics.progress > filter.maxProgress) {
      return false;
    }
    
    // Filter by duration
    if (filter.minDuration !== undefined && metrics.duration < filter.minDuration) {
      return false;
    }
    
    if (filter.maxDuration !== undefined && metrics.duration > filter.maxDuration) {
      return false;
    }
    
    // Filter by start time
    if (filter.startTimeRange && metrics.startTime) {
      const [min, max] = filter.startTimeRange;
      if (metrics.startTime < min || metrics.startTime > max) {
        return false;
      }
    }
    
    // Filter by end time
    if (filter.endTimeRange && metrics.endTime) {
      const [min, max] = filter.endTimeRange;
      if (metrics.endTime < min || metrics.endTime > max) {
        return false;
      }
    }
    
    // Filter by custom metrics
    if (filter.customMetrics && metrics.customMetrics) {
      for (const [key, value] of Object.entries(filter.customMetrics)) {
        if (metrics.customMetrics[key] !== value) {
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
      // Only persist the task history
      localStorage.setItem(this.config.storageKey, JSON.stringify(this.taskHistory));
    } catch (error) {
      console.error('Failed to persist task metrics:', error);
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
        this.taskHistory = persistedMetrics;
      }
    } catch (error) {
      console.error('Failed to load persisted task metrics:', error);
    }
  }
}

/**
 * Global task monitor instance
 */
export const globalTaskMonitor = new TaskMonitor();

/**
 * Hook for using task monitoring in React components
 * 
 * @param config - Task monitoring configuration
 * @returns Task monitor instance and utility functions
 */
export function useTaskMonitoring(config?: TaskMonitoringConfig) {
  // Use the global monitor or create a new one
  const monitor = config ? new TaskMonitor(config) : globalTaskMonitor;
  
  // Start monitoring if not already started
  if (!monitor['isMonitoring']) {
    monitor.startMonitoring();
  }
  
  return {
    monitor,
    
    /**
     * Gets metrics for a specific task
     * 
     * @param taskId - Task ID
     * @returns Task metrics or undefined if not found
     */
    getTaskMetrics: (taskId: string) => monitor.getTaskMetrics(taskId),
    
    /**
     * Gets metrics for all tasks
     * 
     * @returns Map of task metrics
     */
    getAllTaskMetrics: () => monitor.getAllTaskMetrics(),
    
    /**
     * Gets task metrics history
     * 
     * @param filter - Filter options
     * @returns Array of task metrics
     */
    getTaskHistory: (filter?: TaskMonitoringFilter) => monitor.getTaskHistory(filter),
    
    /**
     * Gets task monitoring statistics
     * 
     * @returns Task monitoring statistics
     */
    getStatistics: () => monitor.getStatistics(),
    
    /**
     * Adds a listener for metrics updates
     * 
     * @param listener - Listener function
     * @returns Function to remove the listener
     */
    addListener: (listener: (metrics: TaskMetrics) => void) => monitor.addListener(listener),
    
    /**
     * Clears all metrics
     */
    clearMetrics: () => monitor.clearMetrics(),
  };
}