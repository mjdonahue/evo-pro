import React, { useState, useEffect, useMemo } from 'react';
import { useFeatureUsageAnalytics, AnalyticsEvent, AnalyticsReport } from '../../utils/featureUsageAnalytics';

/**
 * Props for the FeatureUsageDashboard component
 */
interface FeatureUsageDashboardProps {
  /** Title for the dashboard */
  title?: string;
  /** Whether to auto-refresh the dashboard */
  autoRefresh?: boolean;
  /** Refresh interval in milliseconds */
  refreshInterval?: number;
  /** Number of days to show in the report */
  days?: number;
  /** Maximum number of features to show in the top features list */
  maxFeatures?: number;
  /** User ID for consent management */
  userId?: string;
  /** Custom CSS class */
  className?: string;
}

/**
 * A dashboard component for visualizing feature usage analytics
 */
export const FeatureUsageDashboard: React.FC<FeatureUsageDashboardProps> = ({
  title = 'Feature Usage Analytics',
  autoRefresh = true,
  refreshInterval = 60000, // 1 minute
  days = 30,
  maxFeatures = 10,
  userId,
  className = '',
}) => {
  // Get feature usage analytics utilities
  const {
    getFeatureUsageEvents,
    generateFeatureUsageReport,
    getConsent,
    updateConsent,
  } = useFeatureUsageAnalytics();
  
  // State for the dashboard
  const [events, setEvents] = useState<AnalyticsEvent[]>([]);
  const [report, setReport] = useState<AnalyticsReport | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [consent, setConsent] = useState<{
    featureUsage: boolean;
    performance: boolean;
    errorReporting: boolean;
    userInterface: boolean;
  } | null>(null);
  
  // Calculate date range
  const endDate = useMemo(() => new Date(), []);
  const startDate = useMemo(() => {
    const date = new Date();
    date.setDate(date.getDate() - days);
    return date;
  }, [days]);
  
  // Load data
  const loadData = async () => {
    setLoading(true);
    setError(null);
    
    try {
      // Get feature usage events
      const eventsData = await getFeatureUsageEvents({
        startDate,
        endDate,
      });
      setEvents(eventsData);
      
      // Generate feature usage report
      const reportData = await generateFeatureUsageReport({
        startDate,
        endDate,
      });
      setReport(reportData);
      
      // Get consent if userId is provided
      if (userId) {
        const consentData = await getConsent(userId);
        if (consentData) {
          setConsent({
            featureUsage: consentData.featureUsage,
            performance: consentData.performance,
            errorReporting: consentData.errorReporting,
            userInterface: consentData.userInterface,
          });
        }
      }
    } catch (err) {
      setError('Failed to load feature usage data');
      console.error('Error loading feature usage data:', err);
    } finally {
      setLoading(false);
    }
  };
  
  // Load data on mount and when dependencies change
  useEffect(() => {
    loadData();
    
    // Set up auto-refresh
    if (autoRefresh) {
      const intervalId = setInterval(loadData, refreshInterval);
      return () => clearInterval(intervalId);
    }
  }, [startDate, endDate, userId, autoRefresh, refreshInterval]);
  
  // Handle consent update
  const handleConsentUpdate = async (field: keyof typeof consent, value: boolean) => {
    if (!userId || !consent) return;
    
    const updatedConsent = {
      ...consent,
      [field]: value,
    };
    
    try {
      await updateConsent(
        userId,
        updatedConsent.featureUsage,
        updatedConsent.performance,
        updatedConsent.errorReporting,
        updatedConsent.userInterface
      );
      setConsent(updatedConsent);
    } catch (err) {
      console.error('Error updating consent:', err);
    }
  };
  
  // Calculate top features
  const topFeatures = useMemo(() => {
    if (!report || !report.eventCounts) return [];
    
    return Object.entries(report.eventCounts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, maxFeatures)
      .map(([name, count]) => ({ name, count }));
  }, [report, maxFeatures]);
  
  // Calculate daily usage data for chart
  const dailyUsageData = useMemo(() => {
    if (!report || !report.dailyCounts) return [];
    
    // Sort dates
    return Object.entries(report.dailyCounts)
      .sort((a, b) => new Date(a[0]).getTime() - new Date(b[0]).getTime())
      .map(([date, count]) => ({
        date: new Date(date).toLocaleDateString(),
        count,
      }));
  }, [report]);
  
  // Calculate feature usage by action
  const actionCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    
    events.forEach(event => {
      const action = event.properties.action;
      if (action && typeof action === 'string') {
        counts[action] = (counts[action] || 0) + 1;
      }
    });
    
    return Object.entries(counts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, maxFeatures)
      .map(([action, count]) => ({ action, count }));
  }, [events, maxFeatures]);
  
  return (
    <div className={`feature-usage-dashboard p-4 ${className}`}>
      <div className="dashboard-header flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">{title}</h2>
        <div className="flex items-center">
          <span className="text-sm text-gray-500 mr-2">
            {startDate.toLocaleDateString()} - {endDate.toLocaleDateString()}
          </span>
          <button 
            className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
            onClick={loadData}
            disabled={loading}
          >
            {loading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>
      
      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
          {error}
        </div>
      )}
      
      {userId && consent && (
        <div className="consent-section bg-gray-100 p-3 rounded mb-4">
          <h3 className="text-lg font-semibold mb-2">Analytics Consent</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <div className="flex items-center">
              <input
                type="checkbox"
                id="feature-usage-consent"
                checked={consent.featureUsage}
                onChange={(e) => handleConsentUpdate('featureUsage', e.target.checked)}
                className="mr-2"
              />
              <label htmlFor="feature-usage-consent">Feature Usage</label>
            </div>
            <div className="flex items-center">
              <input
                type="checkbox"
                id="performance-consent"
                checked={consent.performance}
                onChange={(e) => handleConsentUpdate('performance', e.target.checked)}
                className="mr-2"
              />
              <label htmlFor="performance-consent">Performance</label>
            </div>
            <div className="flex items-center">
              <input
                type="checkbox"
                id="error-reporting-consent"
                checked={consent.errorReporting}
                onChange={(e) => handleConsentUpdate('errorReporting', e.target.checked)}
                className="mr-2"
              />
              <label htmlFor="error-reporting-consent">Error Reporting</label>
            </div>
            <div className="flex items-center">
              <input
                type="checkbox"
                id="user-interface-consent"
                checked={consent.userInterface}
                onChange={(e) => handleConsentUpdate('userInterface', e.target.checked)}
                className="mr-2"
              />
              <label htmlFor="user-interface-consent">User Interface</label>
            </div>
          </div>
        </div>
      )}
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
        <div className="bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Overview</h3>
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-gray-50 p-3 rounded">
              <h4 className="text-sm font-medium text-gray-500">Total Events</h4>
              <p className="text-2xl font-bold">{report?.totalEvents || 0}</p>
            </div>
            <div className="bg-gray-50 p-3 rounded">
              <h4 className="text-sm font-medium text-gray-500">Unique Features</h4>
              <p className="text-2xl font-bold">{Object.keys(report?.eventCounts || {}).length}</p>
            </div>
            <div className="bg-gray-50 p-3 rounded">
              <h4 className="text-sm font-medium text-gray-500">Daily Average</h4>
              <p className="text-2xl font-bold">
                {report?.totalEvents && dailyUsageData.length
                  ? Math.round(report.totalEvents / dailyUsageData.length)
                  : 0}
              </p>
            </div>
            <div className="bg-gray-50 p-3 rounded">
              <h4 className="text-sm font-medium text-gray-500">Unique Actions</h4>
              <p className="text-2xl font-bold">{actionCounts.length}</p>
            </div>
          </div>
        </div>
        
        <div className="bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Top Features</h3>
          {topFeatures.length > 0 ? (
            <div className="overflow-hidden">
              {topFeatures.map(({ name, count }, index) => (
                <div key={name} className="mb-2">
                  <div className="flex justify-between mb-1">
                    <span className="text-sm font-medium">{name}</span>
                    <span className="text-sm text-gray-500">{count}</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full"
                      style={{
                        width: `${(count / topFeatures[0].count) * 100}%`,
                      }}
                    ></div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500">No feature usage data available</p>
          )}
        </div>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
        <div className="bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Daily Usage</h3>
          {dailyUsageData.length > 0 ? (
            <div className="h-64">
              <div className="flex h-full items-end">
                {dailyUsageData.map(({ date, count }) => {
                  const maxCount = Math.max(...dailyUsageData.map(d => d.count));
                  const height = maxCount > 0 ? (count / maxCount) * 100 : 0;
                  
                  return (
                    <div
                      key={date}
                      className="flex-1 flex flex-col items-center mx-1"
                    >
                      <div
                        className="w-full bg-blue-500 rounded-t"
                        style={{ height: `${height}%` }}
                      ></div>
                      <div className="text-xs mt-1 transform -rotate-45 origin-top-left">
                        {date}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          ) : (
            <p className="text-gray-500">No daily usage data available</p>
          )}
        </div>
        
        <div className="bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Top Actions</h3>
          {actionCounts.length > 0 ? (
            <div className="overflow-hidden">
              {actionCounts.map(({ action, count }, index) => (
                <div key={action} className="mb-2">
                  <div className="flex justify-between mb-1">
                    <span className="text-sm font-medium">{action}</span>
                    <span className="text-sm text-gray-500">{count}</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-green-500 h-2 rounded-full"
                      style={{
                        width: `${(count / actionCounts[0].count) * 100}%`,
                      }}
                    ></div>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-gray-500">No action data available</p>
          )}
        </div>
      </div>
      
      <div className="bg-white p-4 rounded shadow">
        <h3 className="text-lg font-semibold mb-3">Recent Feature Usage</h3>
        {events.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full bg-white">
              <thead className="bg-gray-100">
                <tr>
                  <th className="py-2 px-3 text-left">Feature</th>
                  <th className="py-2 px-3 text-left">Action</th>
                  <th className="py-2 px-3 text-left">Timestamp</th>
                  <th className="py-2 px-3 text-left">Session</th>
                </tr>
              </thead>
              <tbody>
                {events.slice(0, 10).map((event) => (
                  <tr key={event.id} className="border-t hover:bg-gray-50">
                    <td className="py-2 px-3">{event.eventName}</td>
                    <td className="py-2 px-3">{event.properties.action || 'N/A'}</td>
                    <td className="py-2 px-3">
                      {new Date(event.timestamp).toLocaleString()}
                    </td>
                    <td className="py-2 px-3">
                      <span className="text-xs">{event.sessionId.substring(0, 8)}...</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-gray-500">No recent feature usage data available</p>
        )}
      </div>
    </div>
  );
};