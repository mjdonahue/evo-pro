/**
 * Synchronization Status Indicators
 * 
 * This module provides UI components for displaying synchronization status
 * to users, including progress indicators, status badges, and notifications.
 */

import React, { useState, useEffect, useCallback } from 'react';
import { SyncStatus, SyncProgress, SyncResult, SyncSession } from '../api/crossDeviceSync';
import { Conflict } from '../api/conflict';
import { CrossDeviceConflict } from '../api/crossDeviceConflict';

/**
 * Props for the SyncStatusBadge component
 */
interface SyncStatusBadgeProps {
  /** Current synchronization status */
  status: SyncStatus;
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
  /** Whether to show the status text */
  showText?: boolean;
  /** Optional size variant */
  size?: 'small' | 'medium' | 'large';
}

/**
 * A badge that displays the current synchronization status
 */
export const SyncStatusBadge: React.FC<SyncStatusBadgeProps> = ({
  status,
  className = '',
  style = {},
  showText = true,
  size = 'medium'
}) => {
  // Map status to color and text
  const getStatusInfo = () => {
    switch (status) {
      case SyncStatus.IDLE:
        return { color: 'bg-gray-400', text: 'Idle' };
      case SyncStatus.SYNCING:
        return { color: 'bg-blue-500 animate-pulse', text: 'Syncing' };
      case SyncStatus.COMPLETED:
        return { color: 'bg-green-500', text: 'Synced' };
      case SyncStatus.FAILED:
        return { color: 'bg-red-500', text: 'Failed' };
      case SyncStatus.PARTIALLY_COMPLETED:
        return { color: 'bg-yellow-500', text: 'Partial' };
      default:
        return { color: 'bg-gray-400', text: 'Unknown' };
    }
  };

  const { color, text } = getStatusInfo();
  
  // Map size to CSS classes
  const sizeClasses = {
    small: 'h-2 w-2',
    medium: 'h-3 w-3',
    large: 'h-4 w-4'
  };
  
  const textSizeClasses = {
    small: 'text-xs',
    medium: 'text-sm',
    large: 'text-base'
  };

  return (
    <div className={`flex items-center ${className}`} style={style}>
      <div className={`rounded-full ${color} ${sizeClasses[size]}`} />
      {showText && (
        <span className={`ml-2 ${textSizeClasses[size]}`}>{text}</span>
      )}
    </div>
  );
};

/**
 * Props for the SyncProgressBar component
 */
interface SyncProgressBarProps {
  /** Current synchronization progress */
  progress: SyncProgress;
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
  /** Whether to show the percentage text */
  showPercentage?: boolean;
  /** Whether to show detailed stats */
  showDetails?: boolean;
  /** Optional height of the progress bar */
  height?: number;
}

/**
 * A progress bar that displays synchronization progress
 */
export const SyncProgressBar: React.FC<SyncProgressBarProps> = ({
  progress,
  className = '',
  style = {},
  showPercentage = true,
  showDetails = false,
  height = 8
}) => {
  // Calculate percentage
  const percentage = progress.total > 0 
    ? Math.round((progress.completed / progress.total) * 100) 
    : 0;
  
  // Format time remaining
  const formatTimeRemaining = () => {
    if (!progress.startTime || progress.total === 0 || progress.completed === 0) {
      return 'Calculating...';
    }
    
    const elapsedMs = Date.now() - progress.startTime;
    const itemsPerMs = progress.completed / elapsedMs;
    const remainingItems = progress.total - progress.completed;
    const estimatedRemainingMs = remainingItems / itemsPerMs;
    
    if (estimatedRemainingMs < 1000) {
      return 'Less than a second';
    } else if (estimatedRemainingMs < 60000) {
      return `${Math.round(estimatedRemainingMs / 1000)} seconds`;
    } else if (estimatedRemainingMs < 3600000) {
      return `${Math.round(estimatedRemainingMs / 60000)} minutes`;
    } else {
      return `${Math.round(estimatedRemainingMs / 3600000)} hours`;
    }
  };

  return (
    <div className={`w-full ${className}`} style={style}>
      <div className="w-full bg-gray-200 rounded-full" style={{ height }}>
        <div 
          className="bg-blue-600 rounded-full" 
          style={{ 
            width: `${percentage}%`, 
            height: '100%',
            transition: 'width 0.3s ease-in-out'
          }}
        />
      </div>
      
      {showPercentage && (
        <div className="mt-1 text-sm text-gray-600">
          {percentage}% complete
        </div>
      )}
      
      {showDetails && (
        <div className="mt-2 text-xs text-gray-500 grid grid-cols-2 gap-2">
          <div>Completed: {progress.completed} / {progress.total}</div>
          <div>Failed: {progress.failed}</div>
          <div>Status: {progress.status}</div>
          <div>Remaining: {formatTimeRemaining()}</div>
        </div>
      )}
    </div>
  );
};

/**
 * Props for the SyncStatusSummary component
 */
interface SyncStatusSummaryProps {
  /** Synchronization result */
  result: SyncResult;
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
  /** Whether to show detailed information */
  detailed?: boolean;
  /** Optional callback when retry is clicked */
  onRetry?: () => void;
}

/**
 * A summary of synchronization results
 */
export const SyncStatusSummary: React.FC<SyncStatusSummaryProps> = ({
  result,
  className = '',
  style = {},
  detailed = false,
  onRetry
}) => {
  return (
    <div className={`p-4 border rounded-lg ${className}`} style={style}>
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-lg font-medium">
          Synchronization {result.success ? 'Completed' : 'Failed'}
        </h3>
        <SyncStatusBadge 
          status={result.success ? SyncStatus.COMPLETED : SyncStatus.FAILED} 
          size="small"
        />
      </div>
      
      <div className="text-sm text-gray-600 mb-3">
        {result.success 
          ? `Successfully synchronized ${result.completed} items in ${(result.duration / 1000).toFixed(1)}s`
          : `Synchronization failed after ${(result.duration / 1000).toFixed(1)}s`
        }
      </div>
      
      {detailed && (
        <div className="grid grid-cols-2 gap-2 text-sm text-gray-500 mb-3">
          <div>Total items: {result.total}</div>
          <div>Completed: {result.completed}</div>
          <div>Failed: {result.failed}</div>
          <div>Skipped: {result.skipped}</div>
          <div>Conflicts: {result.conflicts}</div>
          <div>Duration: {(result.duration / 1000).toFixed(2)}s</div>
        </div>
      )}
      
      {!result.success && result.error && (
        <div className="text-sm text-red-500 mb-3">
          Error: {result.error.message}
        </div>
      )}
      
      {!result.success && onRetry && (
        <button 
          className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-sm"
          onClick={onRetry}
        >
          Retry Synchronization
        </button>
      )}
    </div>
  );
};

/**
 * Props for the ConflictIndicator component
 */
interface ConflictIndicatorProps {
  /** Conflicts to display */
  conflicts: Conflict[] | CrossDeviceConflict[];
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
  /** Optional callback when resolve is clicked */
  onResolve?: (conflict: Conflict | CrossDeviceConflict) => void;
  /** Optional callback when ignore is clicked */
  onIgnore?: (conflict: Conflict | CrossDeviceConflict) => void;
}

/**
 * A component that displays synchronization conflicts
 */
export const ConflictIndicator: React.FC<ConflictIndicatorProps> = ({
  conflicts,
  className = '',
  style = {},
  onResolve,
  onIgnore
}) => {
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  
  // Toggle expansion state for a conflict
  const toggleExpand = (id: string) => {
    setExpanded(prev => ({
      ...prev,
      [id]: !prev[id]
    }));
  };
  
  // Get a short description of the conflict
  const getConflictDescription = (conflict: Conflict | CrossDeviceConflict) => {
    const entityType = conflict.operation?.entityType || 'Unknown';
    const entityId = conflict.operation?.entityId || 'Unknown';
    
    switch (conflict.type) {
      case 'update_update':
        return `Concurrent updates to ${entityType} (${entityId})`;
      case 'update_delete':
        return `Update conflicts with deletion of ${entityType} (${entityId})`;
      case 'delete_update':
        return `Deletion conflicts with update of ${entityType} (${entityId})`;
      case 'create_create':
        return `Duplicate creation of ${entityType} (${entityId})`;
      default:
        return `Conflict with ${entityType} (${entityId})`;
    }
  };
  
  // Get device information if available (for cross-device conflicts)
  const getDeviceInfo = (conflict: Conflict | CrossDeviceConflict) => {
    if ('sourceDevice' in conflict && conflict.sourceDevice) {
      return (
        <div className="text-xs text-gray-500 mt-1">
          Between {conflict.sourceDevice.name} and {
            'targetDevice' in conflict && conflict.targetDevice 
              ? conflict.targetDevice.name 
              : 'another device'
          }
        </div>
      );
    }
    return null;
  };

  if (conflicts.length === 0) {
    return null;
  }

  return (
    <div className={`border rounded-lg overflow-hidden ${className}`} style={style}>
      <div className="bg-yellow-100 px-4 py-2 border-b">
        <div className="font-medium text-yellow-800">
          {conflicts.length} Synchronization {conflicts.length === 1 ? 'Conflict' : 'Conflicts'} Detected
        </div>
      </div>
      
      <div className="divide-y">
        {conflicts.map((conflict, index) => (
          <div key={conflict.timestamp + index} className="p-3">
            <div 
              className="flex justify-between items-center cursor-pointer"
              onClick={() => toggleExpand(`${conflict.timestamp}-${index}`)}
            >
              <div>
                <div className="font-medium text-sm">
                  {getConflictDescription(conflict)}
                </div>
                {getDeviceInfo(conflict)}
              </div>
              <div className="text-gray-400">
                {expanded[`${conflict.timestamp}-${index}`] ? '▼' : '▶'}
              </div>
            </div>
            
            {expanded[`${conflict.timestamp}-${index}`] && (
              <div className="mt-2">
                <div className="text-xs text-gray-500 mb-2">
                  {new Date(conflict.timestamp).toLocaleString()}
                </div>
                
                <div className="grid grid-cols-2 gap-2 text-xs mb-3">
                  <div>
                    <div className="font-medium mb-1">Local Version</div>
                    <pre className="bg-gray-100 p-2 rounded overflow-auto max-h-32">
                      {JSON.stringify(conflict.localState, null, 2)}
                    </pre>
                  </div>
                  <div>
                    <div className="font-medium mb-1">Remote Version</div>
                    <pre className="bg-gray-100 p-2 rounded overflow-auto max-h-32">
                      {JSON.stringify(conflict.serverState, null, 2)}
                    </pre>
                  </div>
                </div>
                
                <div className="flex space-x-2">
                  {onResolve && !conflict.resolved && (
                    <button 
                      className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-xs"
                      onClick={() => onResolve(conflict)}
                    >
                      Resolve Conflict
                    </button>
                  )}
                  
                  {onIgnore && !conflict.resolved && (
                    <button 
                      className="px-3 py-1 bg-gray-300 text-gray-700 rounded hover:bg-gray-400 text-xs"
                      onClick={() => onIgnore(conflict)}
                    >
                      Ignore
                    </button>
                  )}
                  
                  {conflict.resolved && (
                    <div className="text-xs text-green-500">
                      Resolved using {conflict.resolution?.strategy} strategy
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

/**
 * Props for the DeviceSyncStatus component
 */
interface DeviceSyncStatusProps {
  /** Device ID */
  deviceId: string;
  /** Last sync time */
  lastSyncTime?: number;
  /** Sync status */
  status?: SyncStatus;
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
  /** Optional callback when sync is clicked */
  onSyncClick?: () => void;
}

/**
 * A component that displays the sync status for a specific device
 */
export const DeviceSyncStatus: React.FC<DeviceSyncStatusProps> = ({
  deviceId,
  lastSyncTime,
  status = SyncStatus.IDLE,
  className = '',
  style = {},
  onSyncClick
}) => {
  // Format the last sync time
  const formatLastSync = () => {
    if (!lastSyncTime) {
      return 'Never synced';
    }
    
    const now = Date.now();
    const diffMs = now - lastSyncTime;
    
    if (diffMs < 60000) { // Less than a minute
      return 'Just now';
    } else if (diffMs < 3600000) { // Less than an hour
      const minutes = Math.floor(diffMs / 60000);
      return `${minutes} ${minutes === 1 ? 'minute' : 'minutes'} ago`;
    } else if (diffMs < 86400000) { // Less than a day
      const hours = Math.floor(diffMs / 3600000);
      return `${hours} ${hours === 1 ? 'hour' : 'hours'} ago`;
    } else { // More than a day
      return new Date(lastSyncTime).toLocaleDateString();
    }
  };

  return (
    <div className={`flex items-center justify-between p-3 border rounded ${className}`} style={style}>
      <div>
        <div className="font-medium">{deviceId}</div>
        <div className="text-xs text-gray-500">
          Last synced: {formatLastSync()}
        </div>
      </div>
      
      <div className="flex items-center space-x-3">
        <SyncStatusBadge status={status} size="small" />
        
        {onSyncClick && status !== SyncStatus.SYNCING && (
          <button 
            className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 text-xs"
            onClick={onSyncClick}
            disabled={status === SyncStatus.SYNCING}
          >
            Sync Now
          </button>
        )}
        
        {status === SyncStatus.SYNCING && (
          <div className="w-16 h-4">
            <div className="w-full bg-gray-200 rounded-full h-1">
              <div className="bg-blue-600 h-1 rounded-full animate-pulse" style={{ width: '100%' }} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

/**
 * Props for the SyncNotification component
 */
interface SyncNotificationProps {
  /** Notification message */
  message: string;
  /** Notification type */
  type: 'info' | 'success' | 'warning' | 'error';
  /** Whether the notification is visible */
  visible: boolean;
  /** Optional duration in milliseconds */
  duration?: number;
  /** Optional callback when notification is closed */
  onClose?: () => void;
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
}

/**
 * A notification component for synchronization events
 */
export const SyncNotification: React.FC<SyncNotificationProps> = ({
  message,
  type,
  visible,
  duration = 5000,
  onClose,
  className = '',
  style = {}
}) => {
  useEffect(() => {
    if (visible && duration > 0) {
      const timer = setTimeout(() => {
        if (onClose) onClose();
      }, duration);
      
      return () => clearTimeout(timer);
    }
  }, [visible, duration, onClose]);
  
  if (!visible) {
    return null;
  }
  
  // Map type to color
  const typeToColor = {
    info: 'bg-blue-100 text-blue-800 border-blue-200',
    success: 'bg-green-100 text-green-800 border-green-200',
    warning: 'bg-yellow-100 text-yellow-800 border-yellow-200',
    error: 'bg-red-100 text-red-800 border-red-200'
  };
  
  // Map type to icon
  const typeToIcon = {
    info: '📋',
    success: '✅',
    warning: '⚠️',
    error: '❌'
  };

  return (
    <div 
      className={`p-3 rounded border shadow-sm flex items-start ${typeToColor[type]} ${className}`}
      style={{
        animation: 'fadeIn 0.3s ease-out',
        ...style
      }}
    >
      <div className="mr-2">{typeToIcon[type]}</div>
      <div className="flex-1">{message}</div>
      {onClose && (
        <button 
          className="ml-2 text-gray-500 hover:text-gray-700"
          onClick={onClose}
        >
          ×
        </button>
      )}
    </div>
  );
};

/**
 * Hook for managing synchronization notifications
 */
export function useSyncNotifications() {
  const [notifications, setNotifications] = useState<Array<{
    id: string;
    message: string;
    type: 'info' | 'success' | 'warning' | 'error';
    visible: boolean;
  }>>([]);
  
  // Add a notification
  const addNotification = useCallback((
    message: string, 
    type: 'info' | 'success' | 'warning' | 'error' = 'info'
  ) => {
    const id = Date.now().toString();
    setNotifications(prev => [...prev, { id, message, type, visible: true }]);
    return id;
  }, []);
  
  // Remove a notification
  const removeNotification = useCallback((id: string) => {
    setNotifications(prev => 
      prev.map(n => n.id === id ? { ...n, visible: false } : n)
    );
    
    // Remove from array after animation
    setTimeout(() => {
      setNotifications(prev => prev.filter(n => n.id !== id));
    }, 300);
  }, []);
  
  // Clear all notifications
  const clearNotifications = useCallback(() => {
    setNotifications([]);
  }, []);
  
  return {
    notifications,
    addNotification,
    removeNotification,
    clearNotifications
  };
}

/**
 * Props for the SyncStatusOverview component
 */
interface SyncStatusOverviewProps {
  /** Active synchronization sessions */
  sessions: SyncSession[];
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
  /** Optional callback when a session is clicked */
  onSessionClick?: (session: SyncSession) => void;
}

/**
 * A component that provides an overview of all synchronization sessions
 */
export const SyncStatusOverview: React.FC<SyncStatusOverviewProps> = ({
  sessions,
  className = '',
  style = {},
  onSessionClick
}) => {
  // Count sessions by status
  const countByStatus = sessions.reduce((acc, session) => {
    acc[session.status] = (acc[session.status] || 0) + 1;
    return acc;
  }, {} as Record<SyncStatus, number>);
  
  // Get the overall status
  const getOverallStatus = (): SyncStatus => {
    if (sessions.length === 0) {
      return SyncStatus.IDLE;
    }
    
    if (sessions.some(s => s.status === SyncStatus.SYNCING)) {
      return SyncStatus.SYNCING;
    }
    
    if (sessions.some(s => s.status === SyncStatus.FAILED)) {
      return SyncStatus.PARTIALLY_COMPLETED;
    }
    
    return SyncStatus.COMPLETED;
  };

  return (
    <div className={`border rounded-lg overflow-hidden ${className}`} style={style}>
      <div className="bg-gray-100 px-4 py-3 border-b flex justify-between items-center">
        <h3 className="font-medium">Synchronization Status</h3>
        <SyncStatusBadge status={getOverallStatus()} size="small" />
      </div>
      
      <div className="p-4">
        {sessions.length === 0 ? (
          <div className="text-center text-gray-500 py-4">
            No active synchronization sessions
          </div>
        ) : (
          <>
            <div className="grid grid-cols-4 gap-2 mb-4">
              <div className="text-center p-2 bg-gray-100 rounded">
                <div className="text-lg font-medium">{sessions.length}</div>
                <div className="text-xs text-gray-500">Total</div>
              </div>
              <div className="text-center p-2 bg-blue-100 rounded">
                <div className="text-lg font-medium">{countByStatus[SyncStatus.SYNCING] || 0}</div>
                <div className="text-xs text-gray-500">Syncing</div>
              </div>
              <div className="text-center p-2 bg-green-100 rounded">
                <div className="text-lg font-medium">{countByStatus[SyncStatus.COMPLETED] || 0}</div>
                <div className="text-xs text-gray-500">Completed</div>
              </div>
              <div className="text-center p-2 bg-red-100 rounded">
                <div className="text-lg font-medium">{countByStatus[SyncStatus.FAILED] || 0}</div>
                <div className="text-xs text-gray-500">Failed</div>
              </div>
            </div>
            
            <div className="space-y-2">
              {sessions.map(session => (
                <div 
                  key={session.id}
                  className={`p-2 border rounded flex justify-between items-center ${
                    onSessionClick ? 'cursor-pointer hover:bg-gray-50' : ''
                  }`}
                  onClick={() => onSessionClick && onSessionClick(session)}
                >
                  <div>
                    <div className="font-medium text-sm">
                      {session.sourceDevice.name} → {session.targetDevice.name}
                    </div>
                    <div className="text-xs text-gray-500">
                      {new Date(session.startTime).toLocaleTimeString()}
                      {session.endTime && ` - ${new Date(session.endTime).toLocaleTimeString()}`}
                    </div>
                  </div>
                  <SyncStatusBadge status={session.status} size="small" />
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
};

/**
 * Hook for tracking synchronization status
 */
export function useSyncStatus() {
  const [sessions, setSessions] = useState<SyncSession[]>([]);
  const [currentSession, setCurrentSession] = useState<SyncSession | null>(null);
  const [conflicts, setConflicts] = useState<Conflict[]>([]);
  
  // Update a session
  const updateSession = useCallback((updatedSession: SyncSession) => {
    setSessions(prev => {
      const index = prev.findIndex(s => s.id === updatedSession.id);
      if (index >= 0) {
        const newSessions = [...prev];
        newSessions[index] = updatedSession;
        return newSessions;
      }
      return [...prev, updatedSession];
    });
    
    if (currentSession?.id === updatedSession.id) {
      setCurrentSession(updatedSession);
    }
  }, [currentSession]);
  
  // Add a new session
  const addSession = useCallback((session: SyncSession) => {
    setSessions(prev => [...prev, session]);
    return session;
  }, []);
  
  // Remove a session
  const removeSession = useCallback((sessionId: string) => {
    setSessions(prev => prev.filter(s => s.id !== sessionId));
    if (currentSession?.id === sessionId) {
      setCurrentSession(null);
    }
  }, [currentSession]);
  
  // Set the current active session
  const setActiveSession = useCallback((sessionId: string | null) => {
    if (sessionId === null) {
      setCurrentSession(null);
    } else {
      const session = sessions.find(s => s.id === sessionId);
      if (session) {
        setCurrentSession(session);
      }
    }
  }, [sessions]);
  
  // Add a conflict
  const addConflict = useCallback((conflict: Conflict) => {
    setConflicts(prev => [...prev, conflict]);
  }, []);
  
  // Update a conflict
  const updateConflict = useCallback((updatedConflict: Conflict) => {
    setConflicts(prev => {
      const index = prev.findIndex(c => 
        c.timestamp === updatedConflict.timestamp && 
        c.operation?.id === updatedConflict.operation?.id
      );
      if (index >= 0) {
        const newConflicts = [...prev];
        newConflicts[index] = updatedConflict;
        return newConflicts;
      }
      return prev;
    });
  }, []);
  
  // Remove a conflict
  const removeConflict = useCallback((conflict: Conflict) => {
    setConflicts(prev => prev.filter(c => 
      !(c.timestamp === conflict.timestamp && c.operation?.id === conflict.operation?.id)
    ));
  }, []);
  
  // Clear all conflicts
  const clearConflicts = useCallback(() => {
    setConflicts([]);
  }, []);
  
  return {
    sessions,
    currentSession,
    conflicts,
    updateSession,
    addSession,
    removeSession,
    setActiveSession,
    addConflict,
    updateConflict,
    removeConflict,
    clearConflicts
  };
}

/**
 * Props for the SelectiveSyncOptions component
 */
interface SelectiveSyncOptionsProps {
  /** Available entity types */
  entityTypes: string[];
  /** Selected entity types */
  selectedTypes: string[];
  /** Callback when selection changes */
  onChange: (selected: string[]) => void;
  /** Optional CSS class name */
  className?: string;
  /** Optional inline style */
  style?: React.CSSProperties;
}

/**
 * A component for selecting which entity types to synchronize
 */
export const SelectiveSyncOptions: React.FC<SelectiveSyncOptionsProps> = ({
  entityTypes,
  selectedTypes,
  onChange,
  className = '',
  style = {}
}) => {
  // Toggle a single entity type
  const toggleEntityType = (type: string) => {
    if (selectedTypes.includes(type)) {
      onChange(selectedTypes.filter(t => t !== type));
    } else {
      onChange([...selectedTypes, type]);
    }
  };
  
  // Select all entity types
  const selectAll = () => {
    onChange([...entityTypes]);
  };
  
  // Deselect all entity types
  const deselectAll = () => {
    onChange([]);
  };

  return (
    <div className={`border rounded-lg overflow-hidden ${className}`} style={style}>
      <div className="bg-gray-100 px-4 py-3 border-b flex justify-between items-center">
        <h3 className="font-medium">Selective Synchronization</h3>
        <div className="space-x-2">
          <button 
            className="px-2 py-1 text-xs bg-blue-500 text-white rounded hover:bg-blue-600"
            onClick={selectAll}
          >
            Select All
          </button>
          <button 
            className="px-2 py-1 text-xs bg-gray-300 text-gray-700 rounded hover:bg-gray-400"
            onClick={deselectAll}
          >
            Deselect All
          </button>
        </div>
      </div>
      
      <div className="p-4">
        <div className="text-sm text-gray-600 mb-3">
          Select which data types to synchronize:
        </div>
        
        <div className="space-y-2">
          {entityTypes.map(type => (
            <div key={type} className="flex items-center">
              <input 
                type="checkbox" 
                id={`entity-${type}`}
                checked={selectedTypes.includes(type)}
                onChange={() => toggleEntityType(type)}
                className="mr-2"
              />
              <label htmlFor={`entity-${type}`} className="text-sm">
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </label>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

/**
 * Hook for managing selective synchronization options
 */
export function useSelectiveSync(availableEntityTypes: string[]) {
  const [selectedEntityTypes, setSelectedEntityTypes] = useState<string[]>(availableEntityTypes);
  
  // Select specific entity types
  const selectEntityTypes = useCallback((types: string[]) => {
    setSelectedEntityTypes(types);
  }, []);
  
  // Select all entity types
  const selectAllEntityTypes = useCallback(() => {
    setSelectedEntityTypes([...availableEntityTypes]);
  }, [availableEntityTypes]);
  
  // Deselect all entity types
  const deselectAllEntityTypes = useCallback(() => {
    setSelectedEntityTypes([]);
  }, []);
  
  // Toggle a single entity type
  const toggleEntityType = useCallback((type: string) => {
    setSelectedEntityTypes(prev => {
      if (prev.includes(type)) {
        return prev.filter(t => t !== type);
      } else {
        return [...prev, type];
      }
    });
  }, []);
  
  return {
    selectedEntityTypes,
    selectEntityTypes,
    selectAllEntityTypes,
    deselectAllEntityTypes,
    toggleEntityType
  };
}