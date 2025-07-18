import React, { useState, useEffect, useMemo } from 'react';
import { 
  useTaskMonitoring, 
  TaskMetrics, 
  TaskMonitoringStats, 
  TaskMonitoringFilter 
} from '../../utils/taskMonitoring';
import { TaskStatus, TaskPriority } from '../../utils/taskScheduler';
import { ResourceType } from '../../utils/resourceThrottling';

/**
 * Props for the TaskMonitoringDashboard component
 */
interface TaskMonitoringDashboardProps {
  /** Title for the dashboard */
  title?: string;
  /** Whether to auto-refresh the dashboard */
  autoRefresh?: boolean;
  /** Refresh interval in milliseconds */
  refreshInterval?: number;
  /** Initial filter for tasks */
  initialFilter?: TaskMonitoringFilter;
  /** Whether to show the filter controls */
  showFilters?: boolean;
  /** Whether to show the statistics section */
  showStatistics?: boolean;
  /** Whether to show the task history section */
  showHistory?: boolean;
  /** Whether to show the active tasks section */
  showActiveTasks?: boolean;
  /** Maximum number of tasks to show in the history */
  maxHistoryItems?: number;
  /** Callback when a task is selected */
  onTaskSelect?: (taskId: string) => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * A dashboard component for monitoring background tasks
 */
export const TaskMonitoringDashboard: React.FC<TaskMonitoringDashboardProps> = ({
  title = 'Task Monitoring Dashboard',
  autoRefresh = true,
  refreshInterval = 2000,
  initialFilter = {},
  showFilters = true,
  showStatistics = true,
  showHistory = true,
  showActiveTasks = true,
  maxHistoryItems = 10,
  onTaskSelect,
  className = '',
}) => {
  // Get task monitoring utilities
  const { 
    getAllTaskMetrics, 
    getTaskHistory, 
    getStatistics, 
    addListener 
  } = useTaskMonitoring();
  
  // State for the dashboard
  const [filter, setFilter] = useState<TaskMonitoringFilter>(initialFilter);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<Map<string, TaskMetrics>>(new Map());
  const [history, setHistory] = useState<TaskMetrics[]>([]);
  const [stats, setStats] = useState<TaskMonitoringStats | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  
  // Refresh the dashboard data
  const refreshData = () => {
    setMetrics(getAllTaskMetrics());
    setHistory(getTaskHistory(filter).slice(0, maxHistoryItems));
    setStats(getStatistics());
  };
  
  // Set up auto-refresh
  useEffect(() => {
    refreshData();
    
    if (autoRefresh) {
      const intervalId = setInterval(() => {
        refreshData();
      }, refreshInterval);
      
      return () => {
        clearInterval(intervalId);
      };
    }
  }, [autoRefresh, refreshInterval, filter, refreshKey]);
  
  // Set up listener for real-time updates
  useEffect(() => {
    const removeListener = addListener((updatedMetrics) => {
      setMetrics(prevMetrics => {
        const newMetrics = new Map(prevMetrics);
        newMetrics.set(updatedMetrics.taskId, updatedMetrics);
        return newMetrics;
      });
      
      // If this is a completed task, refresh the history
      if (
        updatedMetrics.status === TaskStatus.COMPLETED ||
        updatedMetrics.status === TaskStatus.FAILED ||
        updatedMetrics.status === TaskStatus.CANCELED
      ) {
        setHistory(getTaskHistory(filter).slice(0, maxHistoryItems));
        setStats(getStatistics());
      }
    });
    
    return removeListener;
  }, [addListener, filter, maxHistoryItems]);
  
  // Handle task selection
  const handleTaskSelect = (taskId: string) => {
    setSelectedTaskId(taskId);
    onTaskSelect?.(taskId);
  };
  
  // Filter tasks by status
  const activeTaskMetrics = useMemo(() => {
    return Array.from(metrics.values()).filter(
      m => m.status === TaskStatus.RUNNING || m.status === TaskStatus.PENDING
    );
  }, [metrics]);
  
  // Manual refresh button handler
  const handleRefresh = () => {
    setRefreshKey(prev => prev + 1);
  };
  
  // Filter change handlers
  const handleFilterChange = (newFilter: Partial<TaskMonitoringFilter>) => {
    setFilter(prevFilter => ({
      ...prevFilter,
      ...newFilter,
    }));
  };
  
  return (
    <div className={`task-monitoring-dashboard p-4 ${className}`}>
      <div className="dashboard-header flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">{title}</h2>
        <button 
          className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
          onClick={handleRefresh}
        >
          Refresh
        </button>
      </div>
      
      {showFilters && (
        <div className="filter-controls bg-gray-100 p-3 rounded mb-4">
          <h3 className="text-lg font-semibold mb-2">Filters</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div>
              <label className="block text-sm font-medium mb-1">Status</label>
              <select 
                className="w-full p-2 border rounded"
                value={filter.status as string || ''}
                onChange={(e) => handleFilterChange({ 
                  status: e.target.value ? e.target.value as TaskStatus : undefined 
                })}
              >
                <option value="">All Statuses</option>
                {Object.values(TaskStatus).map(status => (
                  <option key={status} value={status}>{status}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Priority</label>
              <select 
                className="w-full p-2 border rounded"
                value={filter.priority as string || ''}
                onChange={(e) => handleFilterChange({ 
                  priority: e.target.value ? Number(e.target.value) as TaskPriority : undefined 
                })}
              >
                <option value="">All Priorities</option>
                {Object.entries(TaskPriority)
                  .filter(([key, value]) => typeof value === 'number')
                  .map(([key, value]) => (
                    <option key={key} value={value}>{key}</option>
                  ))
                }
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Task Name</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={filter.name || ''}
                onChange={(e) => handleFilterChange({ name: e.target.value || undefined })}
                placeholder="Filter by name..."
              />
            </div>
          </div>
        </div>
      )}
      
      {showStatistics && stats && (
        <div className="statistics-section bg-white p-4 rounded shadow mb-4">
          <h3 className="text-lg font-semibold mb-3">Statistics</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard 
              title="Total Tasks" 
              value={stats.totalTasks.toString()} 
              icon="📊"
            />
            <StatCard 
              title="Running Tasks" 
              value={stats.runningTasks.toString()} 
              icon="⚙️"
              color="text-blue-500"
            />
            <StatCard 
              title="Completed Tasks" 
              value={stats.completedTasks.toString()} 
              icon="✅"
              color="text-green-500"
            />
            <StatCard 
              title="Failed Tasks" 
              value={stats.failedTasks.toString()} 
              icon="❌"
              color="text-red-500"
            />
            <StatCard 
              title="Avg. Duration" 
              value={`${Math.round(stats.averageDuration)}ms`} 
              icon="⏱️"
            />
            <StatCard 
              title="Avg. Progress" 
              value={`${Math.round(stats.averageProgress)}%`} 
              icon="📈"
            />
            <StatCard 
              title="Completion Rate" 
              value={`${Math.round(stats.completionRate * 100)}%`} 
              icon="🎯"
              color={stats.completionRate > 0.8 ? "text-green-500" : "text-yellow-500"}
            />
            <StatCard 
              title="Failure Rate" 
              value={`${Math.round(stats.failureRate * 100)}%`} 
              icon="💔"
              color={stats.failureRate < 0.2 ? "text-green-500" : "text-red-500"}
            />
          </div>
          
          {/* Resource Usage */}
          {Object.entries(stats.resourceUsage).length > 0 && (
            <div className="mt-4">
              <h4 className="text-md font-semibold mb-2">Resource Usage</h4>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                {Object.entries(stats.resourceUsage).map(([resourceType, usage]) => (
                  <div key={resourceType} className="bg-gray-50 p-3 rounded">
                    <div className="flex justify-between items-center">
                      <span className="font-medium">{resourceType}</span>
                      <span>
                        Avg: {Math.round(usage.average)}% | Peak: {Math.round(usage.peak)}%
                      </span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2.5">
                      <div 
                        className="bg-blue-600 h-2.5 rounded-full" 
                        style={{ width: `${Math.min(100, usage.average)}%` }}
                      ></div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
      
      {showActiveTasks && activeTaskMetrics.length > 0 && (
        <div className="active-tasks-section bg-white p-4 rounded shadow mb-4">
          <h3 className="text-lg font-semibold mb-3">Active Tasks</h3>
          <div className="overflow-x-auto">
            <table className="min-w-full bg-white">
              <thead className="bg-gray-100">
                <tr>
                  <th className="py-2 px-3 text-left">Name</th>
                  <th className="py-2 px-3 text-left">Status</th>
                  <th className="py-2 px-3 text-left">Priority</th>
                  <th className="py-2 px-3 text-left">Progress</th>
                  <th className="py-2 px-3 text-left">Duration</th>
                </tr>
              </thead>
              <tbody>
                {activeTaskMetrics.map(task => (
                  <tr 
                    key={task.taskId} 
                    className={`border-t hover:bg-gray-50 cursor-pointer ${
                      selectedTaskId === task.taskId ? 'bg-blue-50' : ''
                    }`}
                    onClick={() => handleTaskSelect(task.taskId)}
                  >
                    <td className="py-2 px-3">{task.taskName}</td>
                    <td className="py-2 px-3">
                      <TaskStatusBadge status={task.status} />
                    </td>
                    <td className="py-2 px-3">
                      <TaskPriorityBadge priority={task.priority} />
                    </td>
                    <td className="py-2 px-3">
                      <div className="flex items-center">
                        <div className="w-full bg-gray-200 rounded-full h-2.5 mr-2">
                          <div 
                            className="bg-blue-600 h-2.5 rounded-full" 
                            style={{ width: `${task.progress}%` }}
                          ></div>
                        </div>
                        <span className="text-sm">{task.progress}%</span>
                      </div>
                    </td>
                    <td className="py-2 px-3">{formatDuration(task.duration)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
      
      {showHistory && history.length > 0 && (
        <div className="task-history-section bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Task History</h3>
          <div className="overflow-x-auto">
            <table className="min-w-full bg-white">
              <thead className="bg-gray-100">
                <tr>
                  <th className="py-2 px-3 text-left">Name</th>
                  <th className="py-2 px-3 text-left">Status</th>
                  <th className="py-2 px-3 text-left">Priority</th>
                  <th className="py-2 px-3 text-left">Duration</th>
                  <th className="py-2 px-3 text-left">Completed</th>
                  <th className="py-2 px-3 text-left">Retries</th>
                </tr>
              </thead>
              <tbody>
                {history.map(task => (
                  <tr 
                    key={task.taskId} 
                    className={`border-t hover:bg-gray-50 cursor-pointer ${
                      selectedTaskId === task.taskId ? 'bg-blue-50' : ''
                    }`}
                    onClick={() => handleTaskSelect(task.taskId)}
                  >
                    <td className="py-2 px-3">{task.taskName}</td>
                    <td className="py-2 px-3">
                      <TaskStatusBadge status={task.status} />
                    </td>
                    <td className="py-2 px-3">
                      <TaskPriorityBadge priority={task.priority} />
                    </td>
                    <td className="py-2 px-3">{formatDuration(task.duration)}</td>
                    <td className="py-2 px-3">
                      {task.endTime ? formatTime(task.endTime) : 'N/A'}
                    </td>
                    <td className="py-2 px-3">{task.retries}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
      
      {selectedTaskId && (
        <TaskDetailModal 
          taskId={selectedTaskId} 
          onClose={() => setSelectedTaskId(null)} 
        />
      )}
    </div>
  );
};

/**
 * Props for the StatCard component
 */
interface StatCardProps {
  title: string;
  value: string;
  icon?: string;
  color?: string;
}

/**
 * A card component for displaying a statistic
 */
const StatCard: React.FC<StatCardProps> = ({ 
  title, 
  value, 
  icon, 
  color = 'text-gray-800' 
}) => {
  return (
    <div className="bg-gray-50 p-3 rounded shadow-sm">
      <div className="flex items-center mb-1">
        {icon && <span className="mr-2">{icon}</span>}
        <h4 className="text-sm font-medium text-gray-500">{title}</h4>
      </div>
      <p className={`text-xl font-bold ${color}`}>{value}</p>
    </div>
  );
};

/**
 * Props for the TaskStatusBadge component
 */
interface TaskStatusBadgeProps {
  status: TaskStatus;
}

/**
 * A badge component for displaying task status
 */
const TaskStatusBadge: React.FC<TaskStatusBadgeProps> = ({ status }) => {
  let bgColor = 'bg-gray-200';
  let textColor = 'text-gray-800';
  
  switch (status) {
    case TaskStatus.RUNNING:
      bgColor = 'bg-blue-100';
      textColor = 'text-blue-800';
      break;
    case TaskStatus.COMPLETED:
      bgColor = 'bg-green-100';
      textColor = 'text-green-800';
      break;
    case TaskStatus.FAILED:
      bgColor = 'bg-red-100';
      textColor = 'text-red-800';
      break;
    case TaskStatus.PENDING:
      bgColor = 'bg-yellow-100';
      textColor = 'text-yellow-800';
      break;
    case TaskStatus.CANCELED:
      bgColor = 'bg-gray-100';
      textColor = 'text-gray-800';
      break;
    case TaskStatus.PAUSED:
      bgColor = 'bg-purple-100';
      textColor = 'text-purple-800';
      break;
  }
  
  return (
    <span className={`inline-block px-2 py-1 rounded-full text-xs font-semibold ${bgColor} ${textColor}`}>
      {status}
    </span>
  );
};

/**
 * Props for the TaskPriorityBadge component
 */
interface TaskPriorityBadgeProps {
  priority: TaskPriority;
}

/**
 * A badge component for displaying task priority
 */
const TaskPriorityBadge: React.FC<TaskPriorityBadgeProps> = ({ priority }) => {
  let bgColor = 'bg-gray-200';
  let textColor = 'text-gray-800';
  let label = 'Unknown';
  
  switch (priority) {
    case TaskPriority.CRITICAL:
      bgColor = 'bg-red-100';
      textColor = 'text-red-800';
      label = 'Critical';
      break;
    case TaskPriority.HIGH:
      bgColor = 'bg-orange-100';
      textColor = 'text-orange-800';
      label = 'High';
      break;
    case TaskPriority.NORMAL:
      bgColor = 'bg-blue-100';
      textColor = 'text-blue-800';
      label = 'Normal';
      break;
    case TaskPriority.LOW:
      bgColor = 'bg-green-100';
      textColor = 'text-green-800';
      label = 'Low';
      break;
    case TaskPriority.BACKGROUND:
      bgColor = 'bg-gray-100';
      textColor = 'text-gray-800';
      label = 'Background';
      break;
  }
  
  return (
    <span className={`inline-block px-2 py-1 rounded-full text-xs font-semibold ${bgColor} ${textColor}`}>
      {label}
    </span>
  );
};

/**
 * Props for the TaskDetailModal component
 */
interface TaskDetailModalProps {
  taskId: string;
  onClose: () => void;
}

/**
 * A modal component for displaying detailed task information
 */
const TaskDetailModal: React.FC<TaskDetailModalProps> = ({ taskId, onClose }) => {
  const { getTaskMetrics } = useTaskMonitoring();
  const [task, setTask] = useState<TaskMetrics | null>(null);
  
  useEffect(() => {
    const metrics = getTaskMetrics(taskId);
    setTask(metrics || null);
    
    // Set up interval to refresh task data
    const intervalId = setInterval(() => {
      const updatedMetrics = getTaskMetrics(taskId);
      setTask(updatedMetrics || null);
    }, 1000);
    
    return () => {
      clearInterval(intervalId);
    };
  }, [taskId, getTaskMetrics]);
  
  if (!task) {
    return null;
  }
  
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-lg w-full max-w-2xl max-h-[80vh] overflow-y-auto">
        <div className="p-4 border-b flex justify-between items-center">
          <h3 className="text-lg font-semibold">{task.taskName}</h3>
          <button 
            className="text-gray-500 hover:text-gray-700"
            onClick={onClose}
          >
            ✕
          </button>
        </div>
        <div className="p-4">
          <div className="grid grid-cols-2 gap-4 mb-4">
            <div>
              <p className="text-sm text-gray-500">ID</p>
              <p className="font-mono text-sm">{task.taskId}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Status</p>
              <TaskStatusBadge status={task.status} />
            </div>
            <div>
              <p className="text-sm text-gray-500">Priority</p>
              <TaskPriorityBadge priority={task.priority} />
            </div>
            <div>
              <p className="text-sm text-gray-500">Progress</p>
              <div className="flex items-center mt-1">
                <div className="w-full bg-gray-200 rounded-full h-2.5 mr-2">
                  <div 
                    className="bg-blue-600 h-2.5 rounded-full" 
                    style={{ width: `${task.progress}%` }}
                  ></div>
                </div>
                <span className="text-sm">{task.progress}%</span>
              </div>
            </div>
            <div>
              <p className="text-sm text-gray-500">Duration</p>
              <p>{formatDuration(task.duration)}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Retries</p>
              <p>{task.retries}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Started</p>
              <p>{task.startTime ? formatTime(task.startTime) : 'N/A'}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Completed</p>
              <p>{task.endTime ? formatTime(task.endTime) : 'N/A'}</p>
            </div>
          </div>
          
          {task.error && (
            <div className="mb-4">
              <p className="text-sm text-gray-500 mb-1">Error</p>
              <div className="bg-red-50 border border-red-200 rounded p-2 text-red-800 font-mono text-sm">
                {task.error}
              </div>
            </div>
          )}
          
          {task.resourceUsage && Object.keys(task.resourceUsage).length > 0 && (
            <div className="mb-4">
              <p className="text-sm text-gray-500 mb-1">Resource Usage</p>
              <div className="grid grid-cols-1 gap-2">
                {Object.entries(task.resourceUsage).map(([resourceType, usage]) => (
                  <div key={resourceType} className="bg-gray-50 p-2 rounded">
                    <div className="flex justify-between items-center">
                      <span className="font-medium">{resourceType}</span>
                      <span>{Math.round(usage)}%</span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2.5">
                      <div 
                        className="bg-blue-600 h-2.5 rounded-full" 
                        style={{ width: `${Math.min(100, usage)}%` }}
                      ></div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
          
          {task.customMetrics && Object.keys(task.customMetrics).length > 0 && (
            <div>
              <p className="text-sm text-gray-500 mb-1">Custom Metrics</p>
              <div className="bg-gray-50 rounded p-2">
                <pre className="text-sm overflow-x-auto">
                  {JSON.stringify(task.customMetrics, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

/**
 * Formats a duration in milliseconds to a human-readable string
 * 
 * @param duration - Duration in milliseconds
 * @returns Formatted duration string
 */
function formatDuration(duration: number): string {
  if (duration < 1000) {
    return `${duration}ms`;
  } else if (duration < 60000) {
    return `${(duration / 1000).toFixed(1)}s`;
  } else {
    const minutes = Math.floor(duration / 60000);
    const seconds = Math.floor((duration % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  }
}

/**
 * Formats a timestamp to a human-readable string
 * 
 * @param timestamp - Timestamp in milliseconds
 * @returns Formatted time string
 */
function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}