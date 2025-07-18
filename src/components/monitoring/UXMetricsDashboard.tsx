import React, { useState, useEffect, useMemo } from 'react';
import { 
  useUXMonitoring, 
  UXMetrics, 
  UXMetricType, 
  UXMonitoringStats, 
  UXMonitoringFilter 
} from '../../utils/uxMetrics';

/**
 * Props for the UXMetricsDashboard component
 */
interface UXMetricsDashboardProps {
  /** Title for the dashboard */
  title?: string;
  /** Whether to auto-refresh the dashboard */
  autoRefresh?: boolean;
  /** Refresh interval in milliseconds */
  refreshInterval?: number;
  /** Initial filter for metrics */
  initialFilter?: UXMonitoringFilter;
  /** Whether to show the filter controls */
  showFilters?: boolean;
  /** Whether to show the statistics section */
  showStatistics?: boolean;
  /** Whether to show the metrics history section */
  showMetricsHistory?: boolean;
  /** Maximum number of metrics to show in the history */
  maxHistoryItems?: number;
  /** Callback when a metric is selected */
  onMetricSelect?: (metricId: string) => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * A dashboard component for monitoring user experience metrics
 */
export const UXMetricsDashboard: React.FC<UXMetricsDashboardProps> = ({
  title = 'User Experience Metrics Dashboard',
  autoRefresh = true,
  refreshInterval = 5000,
  initialFilter = {},
  showFilters = true,
  showStatistics = true,
  showMetricsHistory = true,
  maxHistoryItems = 20,
  onMetricSelect,
  className = '',
}) => {
  // Get UX monitoring utilities
  const { 
    getMetrics, 
    getStatistics, 
    addListener 
  } = useUXMonitoring();
  
  // State for the dashboard
  const [filter, setFilter] = useState<UXMonitoringFilter>(initialFilter);
  const [selectedMetricId, setSelectedMetricId] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<UXMetrics[]>([]);
  const [stats, setStats] = useState<UXMonitoringStats | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  
  // Refresh the dashboard data
  const refreshData = () => {
    setMetrics(getMetrics(filter).slice(0, maxHistoryItems));
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
    const removeListener = addListener(() => {
      refreshData();
    });
    
    return removeListener;
  }, [addListener, filter, maxHistoryItems]);
  
  // Handle metric selection
  const handleMetricSelect = (metricId: string) => {
    setSelectedMetricId(metricId);
    onMetricSelect?.(metricId);
  };
  
  // Group metrics by type
  const metricsByType = useMemo(() => {
    const grouped: Record<string, UXMetrics[]> = {};
    
    for (const metric of metrics) {
      if (!grouped[metric.metricType]) {
        grouped[metric.metricType] = [];
      }
      
      grouped[metric.metricType].push(metric);
    }
    
    return grouped;
  }, [metrics]);
  
  // Manual refresh button handler
  const handleRefresh = () => {
    setRefreshKey(prev => prev + 1);
  };
  
  // Filter change handlers
  const handleFilterChange = (newFilter: Partial<UXMonitoringFilter>) => {
    setFilter(prevFilter => ({
      ...prevFilter,
      ...newFilter,
    }));
  };
  
  return (
    <div className={`ux-metrics-dashboard p-4 ${className}`}>
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
              <label className="block text-sm font-medium mb-1">Metric Type</label>
              <select 
                className="w-full p-2 border rounded"
                value={filter.metricType as string || ''}
                onChange={(e) => handleFilterChange({ 
                  metricType: e.target.value ? e.target.value as UXMetricType : undefined 
                })}
              >
                <option value="">All Types</option>
                {Object.values(UXMetricType).map(type => (
                  <option key={type} value={type}>{formatMetricType(type)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Location</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={filter.location as string || ''}
                onChange={(e) => handleFilterChange({ 
                  location: e.target.value || undefined 
                })}
                placeholder="Filter by location..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Time Range</label>
              <div className="flex space-x-2">
                <select 
                  className="w-full p-2 border rounded"
                  onChange={(e) => {
                    const value = e.target.value;
                    if (!value) {
                      handleFilterChange({ timeRange: undefined });
                      return;
                    }
                    
                    const now = Date.now();
                    let startTime: number;
                    
                    switch (value) {
                      case '1h':
                        startTime = now - 60 * 60 * 1000;
                        break;
                      case '24h':
                        startTime = now - 24 * 60 * 60 * 1000;
                        break;
                      case '7d':
                        startTime = now - 7 * 24 * 60 * 60 * 1000;
                        break;
                      case '30d':
                        startTime = now - 30 * 24 * 60 * 60 * 1000;
                        break;
                      default:
                        return;
                    }
                    
                    handleFilterChange({ timeRange: [startTime, now] });
                  }}
                >
                  <option value="">All Time</option>
                  <option value="1h">Last Hour</option>
                  <option value="24h">Last 24 Hours</option>
                  <option value="7d">Last 7 Days</option>
                  <option value="30d">Last 30 Days</option>
                </select>
              </div>
            </div>
          </div>
        </div>
      )}
      
      {showStatistics && stats && (
        <div className="statistics-section bg-white p-4 rounded shadow mb-4">
          <h3 className="text-lg font-semibold mb-3">Statistics</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard 
              title="Total Metrics" 
              value={stats.totalMetrics.toString()} 
              icon="📊"
            />
            <StatCard 
              title="Performance Score" 
              value={`${Math.round(stats.performanceScore)}/100`} 
              icon="⚡"
              color={getScoreColor(stats.performanceScore)}
            />
            <StatCard 
              title="Satisfaction Score" 
              value={`${Math.round(stats.satisfactionScore)}/100`} 
              icon="😊"
              color={getScoreColor(stats.satisfactionScore)}
            />
            <StatCard 
              title="Error Rate" 
              value={`${(stats.errorRate * 100).toFixed(1)}%`} 
              icon="❌"
              color={stats.errorRate < 0.05 ? "text-green-500" : stats.errorRate < 0.1 ? "text-yellow-500" : "text-red-500"}
            />
          </div>
          
          {/* Performance Metrics */}
          <div className="mt-4">
            <h4 className="text-md font-semibold mb-2">Performance Metrics</h4>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              {[
                UXMetricType.PAGE_LOAD,
                UXMetricType.API_RESPONSE,
                UXMetricType.COMPONENT_RENDER,
                UXMetricType.TTFP,
                UXMetricType.TTI,
                UXMetricType.LCP
              ].map(metricType => {
                const average = stats.averagesByType[metricType];
                if (average === undefined) return null;
                
                return (
                  <div key={metricType} className="bg-gray-50 p-3 rounded">
                    <div className="flex justify-between items-center">
                      <span className="font-medium">{formatMetricType(metricType)}</span>
                      <span>
                        {formatMetricValue(metricType, average)}
                      </span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2.5">
                      <div 
                        className={`h-2.5 rounded-full ${getPerformanceBarColor(metricType, average)}`}
                        style={{ width: `${getPerformanceBarWidth(metricType, average)}%` }}
                      ></div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
          
          {/* User Interaction Metrics */}
          <div className="mt-4">
            <h4 className="text-md font-semibold mb-2">User Interaction Metrics</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {[
                UXMetricType.INTERACTION_TIME,
                UXMetricType.FID,
                UXMetricType.CLS,
                UXMetricType.SATISFACTION
              ].map(metricType => {
                const average = stats.averagesByType[metricType];
                if (average === undefined) return null;
                
                return (
                  <div key={metricType} className="bg-gray-50 p-3 rounded">
                    <div className="flex justify-between items-center">
                      <span className="font-medium">{formatMetricType(metricType)}</span>
                      <span>
                        {formatMetricValue(metricType, average)}
                      </span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2.5">
                      <div 
                        className={`h-2.5 rounded-full ${getPerformanceBarColor(metricType, average)}`}
                        style={{ width: `${getPerformanceBarWidth(metricType, average)}%` }}
                      ></div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
          
          {/* Metrics by Location */}
          {Object.keys(stats.metricsByLocation).length > 0 && (
            <div className="mt-4">
              <h4 className="text-md font-semibold mb-2">Metrics by Location</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {Object.entries(stats.metricsByLocation)
                  .sort((a, b) => b[1] - a[1])
                  .slice(0, 6)
                  .map(([location, count]) => (
                    <div key={location} className="bg-gray-50 p-3 rounded">
                      <div className="flex justify-between items-center">
                        <span className="font-medium truncate" title={location}>
                          {location.length > 30 ? location.substring(0, 27) + '...' : location}
                        </span>
                        <span>{count} metrics</span>
                      </div>
                    </div>
                  ))
                }
              </div>
            </div>
          )}
        </div>
      )}
      
      {showMetricsHistory && metrics.length > 0 && (
        <div className="metrics-history-section bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Metrics History</h3>
          
          {/* Group metrics by type */}
          {Object.entries(metricsByType).map(([type, typeMetrics]) => (
            <div key={type} className="mb-4">
              <h4 className="text-md font-semibold mb-2">{formatMetricType(type as UXMetricType)}</h4>
              <div className="overflow-x-auto">
                <table className="min-w-full bg-white">
                  <thead className="bg-gray-100">
                    <tr>
                      <th className="py-2 px-3 text-left">Value</th>
                      <th className="py-2 px-3 text-left">Location</th>
                      <th className="py-2 px-3 text-left">Timestamp</th>
                      <th className="py-2 px-3 text-left">Context</th>
                    </tr>
                  </thead>
                  <tbody>
                    {typeMetrics.map(metric => (
                      <tr 
                        key={metric.id} 
                        className={`border-t hover:bg-gray-50 cursor-pointer ${
                          selectedMetricId === metric.id ? 'bg-blue-50' : ''
                        }`}
                        onClick={() => handleMetricSelect(metric.id)}
                      >
                        <td className="py-2 px-3">
                          {formatMetricValue(metric.metricType, metric.value)}
                        </td>
                        <td className="py-2 px-3 max-w-xs truncate" title={metric.location}>
                          {metric.location}
                        </td>
                        <td className="py-2 px-3">
                          {formatTime(metric.timestamp)}
                        </td>
                        <td className="py-2 px-3">
                          {metric.context ? (
                            <details>
                              <summary className="cursor-pointer">View Details</summary>
                              <pre className="text-xs mt-2 bg-gray-50 p-2 rounded overflow-x-auto">
                                {JSON.stringify(metric.context, null, 2)}
                              </pre>
                            </details>
                          ) : 'N/A'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          ))}
        </div>
      )}
      
      {selectedMetricId && (
        <MetricDetailModal 
          metricId={selectedMetricId} 
          metrics={metrics}
          onClose={() => setSelectedMetricId(null)} 
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
 * Props for the MetricDetailModal component
 */
interface MetricDetailModalProps {
  metricId: string;
  metrics: UXMetrics[];
  onClose: () => void;
}

/**
 * A modal component for displaying detailed metric information
 */
const MetricDetailModal: React.FC<MetricDetailModalProps> = ({ 
  metricId, 
  metrics, 
  onClose 
}) => {
  const metric = metrics.find(m => m.id === metricId);
  
  if (!metric) {
    return null;
  }
  
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-lg w-full max-w-2xl max-h-[80vh] overflow-y-auto">
        <div className="p-4 border-b flex justify-between items-center">
          <h3 className="text-lg font-semibold">{formatMetricType(metric.metricType)} Metric</h3>
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
              <p className="font-mono text-sm">{metric.id}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Type</p>
              <p>{formatMetricType(metric.metricType)}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Value</p>
              <p>{formatMetricValue(metric.metricType, metric.value)}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Location</p>
              <p className="break-all">{metric.location}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Session ID</p>
              <p className="font-mono text-sm">{metric.sessionId}</p>
            </div>
            <div>
              <p className="text-sm text-gray-500">Timestamp</p>
              <p>{formatTime(metric.timestamp)}</p>
            </div>
          </div>
          
          {metric.context && Object.keys(metric.context).length > 0 && (
            <div>
              <p className="text-sm text-gray-500 mb-1">Context</p>
              <div className="bg-gray-50 rounded p-2">
                <pre className="text-sm overflow-x-auto">
                  {JSON.stringify(metric.context, null, 2)}
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
 * Formats a metric type to a human-readable string
 * 
 * @param metricType - Metric type
 * @returns Formatted metric type string
 */
function formatMetricType(metricType: UXMetricType): string {
  switch (metricType) {
    case UXMetricType.TTFP:
      return 'Time to First Paint';
    case UXMetricType.TTI:
      return 'Time to Interactive';
    case UXMetricType.FID:
      return 'First Input Delay';
    case UXMetricType.CLS:
      return 'Cumulative Layout Shift';
    case UXMetricType.LCP:
      return 'Largest Contentful Paint';
    case UXMetricType.INTERACTION_TIME:
      return 'Interaction Time';
    case UXMetricType.PAGE_LOAD:
      return 'Page Load Time';
    case UXMetricType.COMPONENT_RENDER:
      return 'Component Render Time';
    case UXMetricType.API_RESPONSE:
      return 'API Response Time';
    case UXMetricType.SATISFACTION:
      return 'User Satisfaction';
    case UXMetricType.ERROR:
      return 'Error';
    case UXMetricType.CUSTOM:
      return 'Custom Metric';
    default:
      return metricType;
  }
}

/**
 * Formats a metric value based on its type
 * 
 * @param metricType - Metric type
 * @param value - Metric value
 * @returns Formatted metric value string
 */
function formatMetricValue(metricType: UXMetricType, value: number): string {
  switch (metricType) {
    case UXMetricType.TTFP:
    case UXMetricType.TTI:
    case UXMetricType.FID:
    case UXMetricType.LCP:
    case UXMetricType.INTERACTION_TIME:
    case UXMetricType.PAGE_LOAD:
    case UXMetricType.COMPONENT_RENDER:
    case UXMetricType.API_RESPONSE:
      return `${value.toFixed(1)}ms`;
    case UXMetricType.CLS:
      return value.toFixed(3);
    case UXMetricType.SATISFACTION:
      return `${(value * 100).toFixed(1)}%`;
    case UXMetricType.ERROR:
      return value === 1 ? 'Yes' : 'No';
    default:
      return value.toString();
  }
}

/**
 * Gets a color for a score
 * 
 * @param score - Score value (0-100)
 * @returns CSS color class
 */
function getScoreColor(score: number): string {
  if (score >= 90) return 'text-green-500';
  if (score >= 70) return 'text-yellow-500';
  return 'text-red-500';
}

/**
 * Gets a color for a performance bar based on metric type and value
 * 
 * @param metricType - Metric type
 * @param value - Metric value
 * @returns CSS color class
 */
function getPerformanceBarColor(metricType: UXMetricType, value: number): string {
  switch (metricType) {
    case UXMetricType.TTFP:
      return value < 500 ? 'bg-green-500' : value < 1000 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.TTI:
      return value < 1000 ? 'bg-green-500' : value < 2000 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.FID:
      return value < 100 ? 'bg-green-500' : value < 300 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.CLS:
      return value < 0.1 ? 'bg-green-500' : value < 0.25 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.LCP:
      return value < 2500 ? 'bg-green-500' : value < 4000 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.PAGE_LOAD:
      return value < 1000 ? 'bg-green-500' : value < 3000 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.COMPONENT_RENDER:
      return value < 50 ? 'bg-green-500' : value < 200 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.API_RESPONSE:
      return value < 200 ? 'bg-green-500' : value < 500 ? 'bg-yellow-500' : 'bg-red-500';
    case UXMetricType.SATISFACTION:
      return value > 0.8 ? 'bg-green-500' : value > 0.5 ? 'bg-yellow-500' : 'bg-red-500';
    default:
      return 'bg-blue-500';
  }
}

/**
 * Gets a width for a performance bar based on metric type and value
 * 
 * @param metricType - Metric type
 * @param value - Metric value
 * @returns Width percentage (0-100)
 */
function getPerformanceBarWidth(metricType: UXMetricType, value: number): number {
  switch (metricType) {
    case UXMetricType.TTFP:
      return Math.min(100, (value / 2000) * 100);
    case UXMetricType.TTI:
      return Math.min(100, (value / 3000) * 100);
    case UXMetricType.FID:
      return Math.min(100, (value / 500) * 100);
    case UXMetricType.CLS:
      return Math.min(100, (value / 0.5) * 100);
    case UXMetricType.LCP:
      return Math.min(100, (value / 5000) * 100);
    case UXMetricType.PAGE_LOAD:
      return Math.min(100, (value / 5000) * 100);
    case UXMetricType.COMPONENT_RENDER:
      return Math.min(100, (value / 300) * 100);
    case UXMetricType.API_RESPONSE:
      return Math.min(100, (value / 1000) * 100);
    case UXMetricType.SATISFACTION:
      return value * 100;
    default:
      return Math.min(100, value);
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