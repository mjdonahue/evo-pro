import React, { useState, useEffect } from 'react';
import { Wifi, WifiOff, RefreshCw, AlertCircle } from 'lucide-react';
import { offlineQueueManager } from '../../lib/api/offline';
import { syncManager, SyncStatus, syncEvents, SyncEventType, SyncEventUnion } from '../../lib/api/sync';

/**
 * Component that displays the current offline/online status and sync status
 */
export function OfflineIndicator() {
  const [isOnline, setIsOnline] = useState(offlineQueueManager.isNetworkOnline());
  const [syncState, setSyncState] = useState(syncManager.getProgress());
  const [queueSize, setQueueSize] = useState(offlineQueueManager.getQueuedOperations().length);

  // Update state when online/offline status changes
  useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);

    // Update queue size when it changes
    const handleQueueChange = () => {
      setQueueSize(offlineQueueManager.getQueuedOperations().length);
    };

    // Update sync state when sync events occur
    const handleSyncEvent = (event: SyncEventUnion) => {
      setSyncState(syncManager.getProgress());
    };

    // Add event listeners
    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    // Custom event listener for queue changes
    document.addEventListener('queue:changed', handleQueueChange);

    // Add sync event listeners using the SyncEventEmitter
    syncEvents.addEventListener(SyncEventType.PROGRESS, handleSyncEvent);
    syncEvents.addEventListener(SyncEventType.COMPLETED, handleSyncEvent);
    syncEvents.addEventListener(SyncEventType.FAILED, handleSyncEvent);
    syncEvents.addEventListener(SyncEventType.OPERATION_SYNCED, handleSyncEvent);
    syncEvents.addEventListener(SyncEventType.OPERATION_FAILED, handleSyncEvent);

    // Initial state check
    setIsOnline(offlineQueueManager.isNetworkOnline());
    setSyncState(syncManager.getProgress());
    setQueueSize(offlineQueueManager.getQueuedOperations().length);

    // Cleanup
    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
      document.removeEventListener('queue:changed', handleQueueChange);

      // Remove sync event listeners
      syncEvents.removeEventListener(SyncEventType.PROGRESS, handleSyncEvent);
      syncEvents.removeEventListener(SyncEventType.COMPLETED, handleSyncEvent);
      syncEvents.removeEventListener(SyncEventType.FAILED, handleSyncEvent);
      syncEvents.removeEventListener(SyncEventType.OPERATION_SYNCED, handleSyncEvent);
      syncEvents.removeEventListener(SyncEventType.OPERATION_FAILED, handleSyncEvent);
    };
  }, []);

  // Determine the icon and color based on the current state
  const getStatusDetails = () => {
    if (!isOnline) {
      return {
        icon: <WifiOff size={20} />,
        label: 'Offline',
        tooltip: `Working offline (${queueSize} operations queued)`,
        className: 'text-yellow-500'
      };
    }

    if (queueSize > 0) {
      if (syncState.status === SyncStatus.SYNCING) {
        return {
          icon: <RefreshCw size={20} className="animate-spin" />,
          label: 'Syncing',
          tooltip: `Syncing ${syncState.completed}/${syncState.total} operations`,
          className: 'text-blue-500'
        };
      }

      return {
        icon: <AlertCircle size={20} />,
        label: 'Pending',
        tooltip: `Online with ${queueSize} operations pending sync`,
        className: 'text-orange-500'
      };
    }

    return {
      icon: <Wifi size={20} />,
      label: 'Online',
      tooltip: 'Connected and in sync',
      className: 'text-green-500'
    };
  };

  const { icon, label, tooltip, className } = getStatusDetails();

  return (
    <div 
      className={`relative w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors group ${className}`}
      onClick={() => {
        // If we're online and have queued operations, trigger a sync
        if (isOnline && queueSize > 0 && syncState.status !== SyncStatus.SYNCING) {
          syncManager.synchronize();
        }
      }}
    >
      {icon}
      <div className="absolute left-full ml-2 px-2 py-1 bg-popover text-popover-foreground text-sm rounded-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity whitespace-nowrap">
        {tooltip}
      </div>
    </div>
  );
}
