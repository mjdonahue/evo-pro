import React, { useState, useEffect, useMemo } from 'react';
import { 
  useTechnicalDebtTracking, 
  TechnicalDebtItem, 
  TechnicalDebtStatus, 
  TechnicalDebtType, 
  TechnicalDebtStats, 
  TechnicalDebtFilter 
} from '../../utils/technicalDebtTracking';

/**
 * Props for the TechnicalDebtDashboard component
 */
interface TechnicalDebtDashboardProps {
  /** Title for the dashboard */
  title?: string;
  /** Whether to auto-refresh the dashboard */
  autoRefresh?: boolean;
  /** Refresh interval in milliseconds */
  refreshInterval?: number;
  /** Initial filter for debt items */
  initialFilter?: TechnicalDebtFilter;
  /** Whether to show the filter controls */
  showFilters?: boolean;
  /** Whether to show the statistics section */
  showStatistics?: boolean;
  /** Whether to show the debt items section */
  showDebtItems?: boolean;
  /** Maximum number of debt items to show */
  maxItems?: number;
  /** Callback when a debt item is selected */
  onItemSelect?: (itemId: string) => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * A dashboard component for monitoring technical debt
 */
export const TechnicalDebtDashboard: React.FC<TechnicalDebtDashboardProps> = ({
  title = 'Technical Debt Dashboard',
  autoRefresh = true,
  refreshInterval = 10000,
  initialFilter = {},
  showFilters = true,
  showStatistics = true,
  showDebtItems = true,
  maxItems = 50,
  onItemSelect,
  className = '',
}) => {
  // Get technical debt tracking utilities
  const { 
    debtItems, 
    stats, 
    getDebtItems, 
    addDebtItem, 
    updateDebtItem, 
    removeDebtItem,
    scanCodeForDebt,
    importFromJson,
    exportToJson
  } = useTechnicalDebtTracking();
  
  // State for the dashboard
  const [filter, setFilter] = useState<TechnicalDebtFilter>(initialFilter);
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [filteredItems, setFilteredItems] = useState<TechnicalDebtItem[]>([]);
  const [isAddingItem, setIsAddingItem] = useState(false);
  const [newItem, setNewItem] = useState<Partial<TechnicalDebtItem>>({
    title: '',
    description: '',
    location: '',
    estimatedEffort: 1,
    impact: 5,
    urgency: 5,
    status: TechnicalDebtStatus.IDENTIFIED,
    type: TechnicalDebtType.CODE,
    tags: []
  });
  const [importExportOpen, setImportExportOpen] = useState(false);
  const [importData, setImportData] = useState('');
  const [refreshKey, setRefreshKey] = useState(0);
  
  // Apply filters to debt items
  useEffect(() => {
    setFilteredItems(getDebtItems(filter).slice(0, maxItems));
  }, [filter, debtItems, getDebtItems, maxItems, refreshKey]);
  
  // Set up auto-refresh
  useEffect(() => {
    if (autoRefresh) {
      const intervalId = setInterval(() => {
        setRefreshKey(prev => prev + 1);
      }, refreshInterval);
      
      return () => {
        clearInterval(intervalId);
      };
    }
  }, [autoRefresh, refreshInterval]);
  
  // Handle item selection
  const handleItemSelect = (itemId: string) => {
    setSelectedItemId(itemId);
    onItemSelect?.(itemId);
  };
  
  // Group debt items by type
  const itemsByType = useMemo(() => {
    const grouped: Record<string, TechnicalDebtItem[]> = {};
    
    for (const item of filteredItems) {
      if (!grouped[item.type]) {
        grouped[item.type] = [];
      }
      
      grouped[item.type].push(item);
    }
    
    return grouped;
  }, [filteredItems]);
  
  // Manual refresh button handler
  const handleRefresh = () => {
    setRefreshKey(prev => prev + 1);
  };
  
  // Filter change handlers
  const handleFilterChange = (newFilter: Partial<TechnicalDebtFilter>) => {
    setFilter(prevFilter => ({
      ...prevFilter,
      ...newFilter,
    }));
  };
  
  // Handle adding a new debt item
  const handleAddItem = () => {
    if (!newItem.title) return;
    
    addDebtItem(newItem);
    
    // Reset form
    setNewItem({
      title: '',
      description: '',
      location: '',
      estimatedEffort: 1,
      impact: 5,
      urgency: 5,
      status: TechnicalDebtStatus.IDENTIFIED,
      type: TechnicalDebtType.CODE,
      tags: []
    });
    
    setIsAddingItem(false);
  };
  
  // Handle importing debt items
  const handleImport = () => {
    if (!importData) return;
    
    const count = importFromJson(importData);
    alert(`Imported ${count} debt items`);
    
    setImportData('');
    setImportExportOpen(false);
  };
  
  // Handle exporting debt items
  const handleExport = () => {
    const data = exportToJson(filter);
    setImportData(data);
  };
  
  return (
    <div className={`technical-debt-dashboard p-4 ${className}`}>
      <div className="dashboard-header flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">{title}</h2>
        <div className="flex space-x-2">
          <button 
            className="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600"
            onClick={() => setIsAddingItem(true)}
          >
            Add Debt Item
          </button>
          <button 
            className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
            onClick={() => setImportExportOpen(true)}
          >
            Import/Export
          </button>
          <button 
            className="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
            onClick={handleRefresh}
          >
            Refresh
          </button>
        </div>
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
                  status: e.target.value ? e.target.value as TechnicalDebtStatus : undefined 
                })}
              >
                <option value="">All Statuses</option>
                {Object.values(TechnicalDebtStatus).map(status => (
                  <option key={status} value={status}>{formatStatus(status)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Type</label>
              <select 
                className="w-full p-2 border rounded"
                value={filter.type as string || ''}
                onChange={(e) => handleFilterChange({ 
                  type: e.target.value ? e.target.value as TechnicalDebtType : undefined 
                })}
              >
                <option value="">All Types</option>
                {Object.values(TechnicalDebtType).map(type => (
                  <option key={type} value={type}>{formatType(type)}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Search</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={filter.searchText || ''}
                onChange={(e) => handleFilterChange({ searchText: e.target.value || undefined })}
                placeholder="Search in title, description..."
              />
            </div>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mt-3">
            <div>
              <label className="block text-sm font-medium mb-1">Location</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={filter.location || ''}
                onChange={(e) => handleFilterChange({ location: e.target.value || undefined })}
                placeholder="Filter by location..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Assignee</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={filter.assignee || ''}
                onChange={(e) => handleFilterChange({ assignee: e.target.value || undefined })}
                placeholder="Filter by assignee..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Tags</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                placeholder="Enter tags separated by commas"
                onChange={(e) => {
                  const tagsText = e.target.value;
                  if (!tagsText) {
                    handleFilterChange({ tags: undefined });
                    return;
                  }
                  
                  const tags = tagsText.split(',').map(tag => tag.trim()).filter(Boolean);
                  handleFilterChange({ tags });
                }}
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
              title="Total Debt Items" 
              value={stats.totalItems.toString()} 
              icon="📊"
            />
            <StatCard 
              title="Total Effort" 
              value={`${stats.totalEffort} days`} 
              icon="⏱️"
              color="text-blue-500"
            />
            <StatCard 
              title="Debt Score" 
              value={Math.round(stats.debtScore).toString()} 
              icon="📈"
              color={getDebtScoreColor(stats.debtScore)}
            />
            <StatCard 
              title="Avg. Impact" 
              value={stats.averageImpact.toFixed(1)} 
              icon="💥"
              color={getImpactColor(stats.averageImpact)}
            />
          </div>
          
          {/* Status Breakdown */}
          <div className="mt-4">
            <h4 className="text-md font-semibold mb-2">Status Breakdown</h4>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              {Object.entries(stats.itemsByStatus)
                .filter(([_, count]) => count > 0)
                .map(([status, count]) => (
                  <div key={status} className="bg-gray-50 p-3 rounded">
                    <div className="flex justify-between items-center">
                      <span className="font-medium">{formatStatus(status as TechnicalDebtStatus)}</span>
                      <span className="text-sm">{count} items</span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2.5">
                      <div 
                        className={`h-2.5 rounded-full ${getStatusColor(status as TechnicalDebtStatus)}`}
                        style={{ width: `${(count / stats.totalItems) * 100}%` }}
                      ></div>
                    </div>
                  </div>
                ))}
            </div>
          </div>
          
          {/* Type Breakdown */}
          <div className="mt-4">
            <h4 className="text-md font-semibold mb-2">Type Breakdown</h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
              {Object.entries(stats.itemsByType)
                .filter(([_, count]) => count > 0)
                .sort((a, b) => b[1] - a[1])
                .slice(0, 6)
                .map(([type, count]) => (
                  <div key={type} className="bg-gray-50 p-3 rounded">
                    <div className="flex justify-between items-center">
                      <span className="font-medium">{formatType(type as TechnicalDebtType)}</span>
                      <span className="text-sm">{count} items</span>
                    </div>
                    <div className="mt-1 bg-gray-200 rounded-full h-2.5">
                      <div 
                        className={`h-2.5 rounded-full ${getTypeColor(type as TechnicalDebtType)}`}
                        style={{ width: `${(count / stats.totalItems) * 100}%` }}
                      ></div>
                    </div>
                  </div>
                ))}
            </div>
          </div>
          
          {/* Trend Over Time */}
          {stats.trend.labels.length > 0 && (
            <div className="mt-4">
              <h4 className="text-md font-semibold mb-2">Trend Over Time</h4>
              <div className="bg-gray-50 p-3 rounded">
                <div className="flex justify-between items-center mb-2">
                  <span className="font-medium">Technical Debt Trend (Last 6 Months)</span>
                </div>
                <div className="relative h-40">
                  {/* X-axis labels */}
                  <div className="absolute bottom-0 left-0 right-0 flex justify-between text-xs text-gray-500">
                    {stats.trend.labels.map((label, i) => (
                      <div key={i} className="text-center" style={{ width: `${100 / stats.trend.labels.length}%` }}>
                        {label}
                      </div>
                    ))}
                  </div>
                  
                  {/* Chart area */}
                  <div className="absolute top-0 left-0 right-0 bottom-5 flex items-end">
                    {stats.trend.total.map((value, i) => {
                      const maxValue = Math.max(...stats.trend.total);
                      const height = maxValue > 0 ? (value / maxValue) * 100 : 0;
                      const identified = stats.trend.identified[i];
                      const resolved = stats.trend.resolved[i];
                      
                      return (
                        <div 
                          key={i} 
                          className="flex flex-col justify-end items-center"
                          style={{ width: `${100 / stats.trend.labels.length}%` }}
                        >
                          <div className="w-full px-1">
                            <div 
                              className="bg-blue-500 w-full" 
                              style={{ height: `${height}%` }}
                              title={`Total: ${value}, Identified: ${identified}, Resolved: ${resolved}`}
                            ></div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
                <div className="flex justify-center mt-2 text-xs">
                  <div className="flex items-center mr-4">
                    <div className="w-3 h-3 bg-blue-500 mr-1"></div>
                    <span>Total Debt</span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
      
      {showDebtItems && filteredItems.length > 0 && (
        <div className="debt-items-section bg-white p-4 rounded shadow">
          <h3 className="text-lg font-semibold mb-3">Technical Debt Items</h3>
          
          {/* Group debt items by type */}
          {Object.entries(itemsByType).map(([type, items]) => (
            <div key={type} className="mb-4">
              <h4 className="text-md font-semibold mb-2">{formatType(type as TechnicalDebtType)}</h4>
              <div className="overflow-x-auto">
                <table className="min-w-full bg-white">
                  <thead className="bg-gray-100">
                    <tr>
                      <th className="py-2 px-3 text-left">Title</th>
                      <th className="py-2 px-3 text-left">Status</th>
                      <th className="py-2 px-3 text-left">Impact</th>
                      <th className="py-2 px-3 text-left">Urgency</th>
                      <th className="py-2 px-3 text-left">Effort</th>
                      <th className="py-2 px-3 text-left">Location</th>
                      <th className="py-2 px-3 text-left">Tags</th>
                      <th className="py-2 px-3 text-left">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {items.map(item => (
                      <tr 
                        key={item.id} 
                        className={`border-t hover:bg-gray-50 cursor-pointer ${
                          selectedItemId === item.id ? 'bg-blue-50' : ''
                        }`}
                        onClick={() => handleItemSelect(item.id)}
                      >
                        <td className="py-2 px-3">
                          <div className="font-medium">{item.title}</div>
                          <div className="text-xs text-gray-500 truncate max-w-xs" title={item.description}>
                            {item.description}
                          </div>
                        </td>
                        <td className="py-2 px-3">
                          <StatusBadge status={item.status} />
                        </td>
                        <td className="py-2 px-3">
                          <ImpactBadge impact={item.impact} />
                        </td>
                        <td className="py-2 px-3">
                          <UrgencyBadge urgency={item.urgency} />
                        </td>
                        <td className="py-2 px-3">
                          {item.estimatedEffort} {item.estimatedEffort === 1 ? 'day' : 'days'}
                        </td>
                        <td className="py-2 px-3 max-w-xs truncate" title={item.location}>
                          {item.location || 'N/A'}
                        </td>
                        <td className="py-2 px-3">
                          <div className="flex flex-wrap gap-1">
                            {item.tags.slice(0, 3).map(tag => (
                              <span 
                                key={tag} 
                                className="inline-block px-2 py-1 bg-gray-100 text-gray-800 text-xs rounded-full"
                              >
                                {tag}
                              </span>
                            ))}
                            {item.tags.length > 3 && (
                              <span className="inline-block px-2 py-1 bg-gray-100 text-gray-800 text-xs rounded-full">
                                +{item.tags.length - 3}
                              </span>
                            )}
                          </div>
                        </td>
                        <td className="py-2 px-3">
                          <button 
                            className="text-red-500 hover:text-red-700 mr-2"
                            onClick={(e) => {
                              e.stopPropagation();
                              if (confirm('Are you sure you want to remove this debt item?')) {
                                removeDebtItem(item.id);
                              }
                            }}
                          >
                            Delete
                          </button>
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
      
      {showDebtItems && filteredItems.length === 0 && (
        <div className="bg-white p-4 rounded shadow text-center">
          <p className="text-gray-500">No technical debt items found matching the current filters.</p>
        </div>
      )}
      
      {selectedItemId && (
        <DebtItemDetailModal 
          itemId={selectedItemId} 
          debtItems={filteredItems}
          onClose={() => setSelectedItemId(null)}
          onUpdate={updateDebtItem}
        />
      )}
      
      {isAddingItem && (
        <AddDebtItemModal 
          item={newItem}
          onChange={setNewItem}
          onAdd={handleAddItem}
          onClose={() => setIsAddingItem(false)}
        />
      )}
      
      {importExportOpen && (
        <ImportExportModal
          importData={importData}
          onImportDataChange={setImportData}
          onImport={handleImport}
          onExport={handleExport}
          onClose={() => setImportExportOpen(false)}
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
 * Props for the StatusBadge component
 */
interface StatusBadgeProps {
  status: TechnicalDebtStatus;
}

/**
 * A badge component for displaying debt status
 */
const StatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
  return (
    <span className={`inline-block px-2 py-1 rounded-full text-xs font-semibold ${getStatusBadgeClasses(status)}`}>
      {formatStatus(status)}
    </span>
  );
};

/**
 * Props for the ImpactBadge component
 */
interface ImpactBadgeProps {
  impact: number;
}

/**
 * A badge component for displaying impact level
 */
const ImpactBadge: React.FC<ImpactBadgeProps> = ({ impact }) => {
  return (
    <span className={`inline-block px-2 py-1 rounded-full text-xs font-semibold ${getImpactBadgeClasses(impact)}`}>
      {impact}/10
    </span>
  );
};

/**
 * Props for the UrgencyBadge component
 */
interface UrgencyBadgeProps {
  urgency: number;
}

/**
 * A badge component for displaying urgency level
 */
const UrgencyBadge: React.FC<UrgencyBadgeProps> = ({ urgency }) => {
  return (
    <span className={`inline-block px-2 py-1 rounded-full text-xs font-semibold ${getUrgencyBadgeClasses(urgency)}`}>
      {urgency}/10
    </span>
  );
};

/**
 * Props for the DebtItemDetailModal component
 */
interface DebtItemDetailModalProps {
  itemId: string;
  debtItems: TechnicalDebtItem[];
  onClose: () => void;
  onUpdate: (id: string, updates: Partial<TechnicalDebtItem>) => void;
}

/**
 * A modal component for displaying and editing debt item details
 */
const DebtItemDetailModal: React.FC<DebtItemDetailModalProps> = ({ 
  itemId, 
  debtItems, 
  onClose,
  onUpdate
}) => {
  const item = debtItems.find(item => item.id === itemId);
  
  if (!item) {
    return null;
  }
  
  const [editedItem, setEditedItem] = useState<TechnicalDebtItem>(item);
  const [isEditing, setIsEditing] = useState(false);
  
  // Reset edited item when selected item changes
  useEffect(() => {
    setEditedItem(item);
    setIsEditing(false);
  }, [item]);
  
  const handleSave = () => {
    onUpdate(itemId, editedItem);
    setIsEditing(false);
  };
  
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-lg w-full max-w-3xl max-h-[80vh] overflow-y-auto">
        <div className="p-4 border-b flex justify-between items-center">
          <h3 className="text-lg font-semibold">
            {isEditing ? 'Edit Technical Debt Item' : 'Technical Debt Item Details'}
          </h3>
          <div className="flex items-center">
            {!isEditing && (
              <button 
                className="text-blue-500 hover:text-blue-700 mr-4"
                onClick={() => setIsEditing(true)}
              >
                Edit
              </button>
            )}
            <button 
              className="text-gray-500 hover:text-gray-700"
              onClick={onClose}
            >
              ✕
            </button>
          </div>
        </div>
        <div className="p-4">
          {isEditing ? (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Title</label>
                <input 
                  type="text" 
                  className="w-full p-2 border rounded"
                  value={editedItem.title}
                  onChange={(e) => setEditedItem({...editedItem, title: e.target.value})}
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Description</label>
                <textarea 
                  className="w-full p-2 border rounded"
                  rows={3}
                  value={editedItem.description}
                  onChange={(e) => setEditedItem({...editedItem, description: e.target.value})}
                />
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Status</label>
                  <select 
                    className="w-full p-2 border rounded"
                    value={editedItem.status}
                    onChange={(e) => setEditedItem({
                      ...editedItem, 
                      status: e.target.value as TechnicalDebtStatus
                    })}
                  >
                    {Object.values(TechnicalDebtStatus).map(status => (
                      <option key={status} value={status}>{formatStatus(status)}</option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Type</label>
                  <select 
                    className="w-full p-2 border rounded"
                    value={editedItem.type}
                    onChange={(e) => setEditedItem({
                      ...editedItem, 
                      type: e.target.value as TechnicalDebtType
                    })}
                  >
                    {Object.values(TechnicalDebtType).map(type => (
                      <option key={type} value={type}>{formatType(type)}</option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Impact (1-10)</label>
                  <input 
                    type="number" 
                    className="w-full p-2 border rounded"
                    min={1}
                    max={10}
                    value={editedItem.impact}
                    onChange={(e) => setEditedItem({
                      ...editedItem, 
                      impact: Math.min(10, Math.max(1, parseInt(e.target.value) || 1))
                    })}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Urgency (1-10)</label>
                  <input 
                    type="number" 
                    className="w-full p-2 border rounded"
                    min={1}
                    max={10}
                    value={editedItem.urgency}
                    onChange={(e) => setEditedItem({
                      ...editedItem, 
                      urgency: Math.min(10, Math.max(1, parseInt(e.target.value) || 1))
                    })}
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Effort (days)</label>
                  <input 
                    type="number" 
                    className="w-full p-2 border rounded"
                    min={0.1}
                    step={0.1}
                    value={editedItem.estimatedEffort}
                    onChange={(e) => setEditedItem({
                      ...editedItem, 
                      estimatedEffort: Math.max(0.1, parseFloat(e.target.value) || 0.1)
                    })}
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Location</label>
                <input 
                  type="text" 
                  className="w-full p-2 border rounded"
                  value={editedItem.location}
                  onChange={(e) => setEditedItem({...editedItem, location: e.target.value})}
                  placeholder="File path, component name, etc."
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Assignee</label>
                <input 
                  type="text" 
                  className="w-full p-2 border rounded"
                  value={editedItem.assignee || ''}
                  onChange={(e) => setEditedItem({...editedItem, assignee: e.target.value})}
                  placeholder="Person responsible for addressing this debt"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Tags (comma separated)</label>
                <input 
                  type="text" 
                  className="w-full p-2 border rounded"
                  value={editedItem.tags.join(', ')}
                  onChange={(e) => {
                    const tagsText = e.target.value;
                    const tags = tagsText.split(',').map(tag => tag.trim()).filter(Boolean);
                    setEditedItem({...editedItem, tags});
                  }}
                  placeholder="refactor, performance, etc."
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Notes</label>
                <textarea 
                  className="w-full p-2 border rounded"
                  rows={3}
                  value={editedItem.notes || ''}
                  onChange={(e) => setEditedItem({...editedItem, notes: e.target.value})}
                  placeholder="Additional context or notes"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Target Date</label>
                <input 
                  type="date" 
                  className="w-full p-2 border rounded"
                  value={editedItem.targetDate ? new Date(editedItem.targetDate).toISOString().split('T')[0] : ''}
                  onChange={(e) => {
                    const date = e.target.value ? new Date(e.target.value).getTime() : undefined;
                    setEditedItem({...editedItem, targetDate: date});
                  }}
                />
              </div>
              <div className="flex justify-end space-x-2 mt-4">
                <button 
                  className="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400"
                  onClick={() => setIsEditing(false)}
                >
                  Cancel
                </button>
                <button 
                  className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                  onClick={handleSave}
                >
                  Save Changes
                </button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <div>
                <h4 className="text-lg font-medium">{item.title}</h4>
                <p className="text-gray-700 mt-1">{item.description}</p>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-gray-500">Status</p>
                  <StatusBadge status={item.status} />
                </div>
                <div>
                  <p className="text-sm text-gray-500">Type</p>
                  <p>{formatType(item.type)}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Impact</p>
                  <ImpactBadge impact={item.impact} />
                </div>
                <div>
                  <p className="text-sm text-gray-500">Urgency</p>
                  <UrgencyBadge urgency={item.urgency} />
                </div>
                <div>
                  <p className="text-sm text-gray-500">Estimated Effort</p>
                  <p>{item.estimatedEffort} {item.estimatedEffort === 1 ? 'day' : 'days'}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Location</p>
                  <p className="break-all">{item.location || 'N/A'}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Assignee</p>
                  <p>{item.assignee || 'Unassigned'}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Target Date</p>
                  <p>{item.targetDate ? new Date(item.targetDate).toLocaleDateString() : 'Not set'}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Created</p>
                  <p>{new Date(item.createdAt).toLocaleDateString()}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Last Updated</p>
                  <p>{new Date(item.updatedAt).toLocaleDateString()}</p>
                </div>
              </div>
              <div>
                <p className="text-sm text-gray-500">Tags</p>
                <div className="flex flex-wrap gap-1 mt-1">
                  {item.tags.map(tag => (
                    <span 
                      key={tag} 
                      className="inline-block px-2 py-1 bg-gray-100 text-gray-800 text-xs rounded-full"
                    >
                      {tag}
                    </span>
                  ))}
                  {item.tags.length === 0 && <p>No tags</p>}
                </div>
              </div>
              {item.notes && (
                <div>
                  <p className="text-sm text-gray-500">Notes</p>
                  <p className="whitespace-pre-wrap">{item.notes}</p>
                </div>
              )}
              {item.relatedItems && item.relatedItems.length > 0 && (
                <div>
                  <p className="text-sm text-gray-500">Related Items</p>
                  <ul className="list-disc pl-5">
                    {item.relatedItems.map((relatedItem, index) => (
                      <li key={index}>{relatedItem}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

/**
 * Props for the AddDebtItemModal component
 */
interface AddDebtItemModalProps {
  item: Partial<TechnicalDebtItem>;
  onChange: (item: Partial<TechnicalDebtItem>) => void;
  onAdd: () => void;
  onClose: () => void;
}

/**
 * A modal component for adding a new debt item
 */
const AddDebtItemModal: React.FC<AddDebtItemModalProps> = ({ 
  item, 
  onChange, 
  onAdd, 
  onClose 
}) => {
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-lg w-full max-w-2xl max-h-[80vh] overflow-y-auto">
        <div className="p-4 border-b flex justify-between items-center">
          <h3 className="text-lg font-semibold">Add Technical Debt Item</h3>
          <button 
            className="text-gray-500 hover:text-gray-700"
            onClick={onClose}
          >
            ✕
          </button>
        </div>
        <div className="p-4">
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1">Title *</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={item.title || ''}
                onChange={(e) => onChange({...item, title: e.target.value})}
                placeholder="Brief title describing the debt"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Description</label>
              <textarea 
                className="w-full p-2 border rounded"
                rows={3}
                value={item.description || ''}
                onChange={(e) => onChange({...item, description: e.target.value})}
                placeholder="Detailed description of the technical debt"
              />
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-1">Status</label>
                <select 
                  className="w-full p-2 border rounded"
                  value={item.status || TechnicalDebtStatus.IDENTIFIED}
                  onChange={(e) => onChange({
                    ...item, 
                    status: e.target.value as TechnicalDebtStatus
                  })}
                >
                  {Object.values(TechnicalDebtStatus).map(status => (
                    <option key={status} value={status}>{formatStatus(status)}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Type</label>
                <select 
                  className="w-full p-2 border rounded"
                  value={item.type || TechnicalDebtType.CODE}
                  onChange={(e) => onChange({
                    ...item, 
                    type: e.target.value as TechnicalDebtType
                  })}
                >
                  {Object.values(TechnicalDebtType).map(type => (
                    <option key={type} value={type}>{formatType(type)}</option>
                  ))}
                </select>
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium mb-1">Impact (1-10)</label>
                <input 
                  type="number" 
                  className="w-full p-2 border rounded"
                  min={1}
                  max={10}
                  value={item.impact || 5}
                  onChange={(e) => onChange({
                    ...item, 
                    impact: Math.min(10, Math.max(1, parseInt(e.target.value) || 1))
                  })}
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Urgency (1-10)</label>
                <input 
                  type="number" 
                  className="w-full p-2 border rounded"
                  min={1}
                  max={10}
                  value={item.urgency || 5}
                  onChange={(e) => onChange({
                    ...item, 
                    urgency: Math.min(10, Math.max(1, parseInt(e.target.value) || 1))
                  })}
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Effort (days)</label>
                <input 
                  type="number" 
                  className="w-full p-2 border rounded"
                  min={0.1}
                  step={0.1}
                  value={item.estimatedEffort || 1}
                  onChange={(e) => onChange({
                    ...item, 
                    estimatedEffort: Math.max(0.1, parseFloat(e.target.value) || 0.1)
                  })}
                />
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Location</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={item.location || ''}
                onChange={(e) => onChange({...item, location: e.target.value})}
                placeholder="File path, component name, etc."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Tags (comma separated)</label>
              <input 
                type="text" 
                className="w-full p-2 border rounded"
                value={(item.tags || []).join(', ')}
                onChange={(e) => {
                  const tagsText = e.target.value;
                  const tags = tagsText.split(',').map(tag => tag.trim()).filter(Boolean);
                  onChange({...item, tags});
                }}
                placeholder="refactor, performance, etc."
              />
            </div>
            <div className="flex justify-end space-x-2 mt-4">
              <button 
                className="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400"
                onClick={onClose}
              >
                Cancel
              </button>
              <button 
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                onClick={onAdd}
                disabled={!item.title}
              >
                Add Debt Item
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

/**
 * Props for the ImportExportModal component
 */
interface ImportExportModalProps {
  importData: string;
  onImportDataChange: (data: string) => void;
  onImport: () => void;
  onExport: () => void;
  onClose: () => void;
}

/**
 * A modal component for importing and exporting debt items
 */
const ImportExportModal: React.FC<ImportExportModalProps> = ({ 
  importData, 
  onImportDataChange, 
  onImport, 
  onExport, 
  onClose 
}) => {
  const [activeTab, setActiveTab] = useState<'import' | 'export'>('import');
  
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-lg w-full max-w-2xl max-h-[80vh] overflow-y-auto">
        <div className="p-4 border-b flex justify-between items-center">
          <h3 className="text-lg font-semibold">Import/Export Technical Debt</h3>
          <button 
            className="text-gray-500 hover:text-gray-700"
            onClick={onClose}
          >
            ✕
          </button>
        </div>
        <div className="p-4">
          <div className="flex border-b mb-4">
            <button 
              className={`px-4 py-2 ${activeTab === 'import' ? 'border-b-2 border-blue-500 text-blue-500' : 'text-gray-500'}`}
              onClick={() => setActiveTab('import')}
            >
              Import
            </button>
            <button 
              className={`px-4 py-2 ${activeTab === 'export' ? 'border-b-2 border-blue-500 text-blue-500' : 'text-gray-500'}`}
              onClick={() => setActiveTab('export')}
            >
              Export
            </button>
          </div>
          
          {activeTab === 'import' ? (
            <div>
              <p className="mb-2">Paste JSON data to import technical debt items:</p>
              <textarea 
                className="w-full p-2 border rounded"
                rows={10}
                value={importData}
                onChange={(e) => onImportDataChange(e.target.value)}
                placeholder="Paste JSON data here..."
              />
              <div className="flex justify-end mt-4">
                <button 
                  className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                  onClick={onImport}
                  disabled={!importData}
                >
                  Import Data
                </button>
              </div>
            </div>
          ) : (
            <div>
              <p className="mb-2">Export technical debt items as JSON:</p>
              <div className="flex justify-end mb-2">
                <button 
                  className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                  onClick={onExport}
                >
                  Generate Export Data
                </button>
              </div>
              <textarea 
                className="w-full p-2 border rounded"
                rows={10}
                value={importData}
                readOnly
                placeholder="Click 'Generate Export Data' to see the JSON data..."
              />
              {importData && (
                <div className="flex justify-end mt-4">
                  <button 
                    className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
                    onClick={() => {
                      navigator.clipboard.writeText(importData);
                      alert('Export data copied to clipboard!');
                    }}
                  >
                    Copy to Clipboard
                  </button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

/**
 * Formats a technical debt status to a human-readable string
 * 
 * @param status - Technical debt status
 * @returns Formatted status string
 */
function formatStatus(status: TechnicalDebtStatus): string {
  switch (status) {
    case TechnicalDebtStatus.IDENTIFIED:
      return 'Identified';
    case TechnicalDebtStatus.ACKNOWLEDGED:
      return 'Acknowledged';
    case TechnicalDebtStatus.SCHEDULED:
      return 'Scheduled';
    case TechnicalDebtStatus.IN_PROGRESS:
      return 'In Progress';
    case TechnicalDebtStatus.RESOLVED:
      return 'Resolved';
    case TechnicalDebtStatus.ACCEPTED:
      return 'Accepted';
    case TechnicalDebtStatus.DEFERRED:
      return 'Deferred';
    default:
      return status;
  }
}

/**
 * Formats a technical debt type to a human-readable string
 * 
 * @param type - Technical debt type
 * @returns Formatted type string
 */
function formatType(type: TechnicalDebtType): string {
  switch (type) {
    case TechnicalDebtType.CODE:
      return 'Code';
    case TechnicalDebtType.ARCHITECTURE:
      return 'Architecture';
    case TechnicalDebtType.DOCUMENTATION:
      return 'Documentation';
    case TechnicalDebtType.TESTING:
      return 'Testing';
    case TechnicalDebtType.INFRASTRUCTURE:
      return 'Infrastructure';
    case TechnicalDebtType.DEPENDENCIES:
      return 'Dependencies';
    case TechnicalDebtType.PERFORMANCE:
      return 'Performance';
    case TechnicalDebtType.SECURITY:
      return 'Security';
    case TechnicalDebtType.ACCESSIBILITY:
      return 'Accessibility';
    case TechnicalDebtType.UX:
      return 'User Experience';
    case TechnicalDebtType.OTHER:
      return 'Other';
    default:
      return type;
  }
}

/**
 * Gets CSS classes for a status badge
 * 
 * @param status - Technical debt status
 * @returns CSS classes string
 */
function getStatusBadgeClasses(status: TechnicalDebtStatus): string {
  switch (status) {
    case TechnicalDebtStatus.IDENTIFIED:
      return 'bg-yellow-100 text-yellow-800';
    case TechnicalDebtStatus.ACKNOWLEDGED:
      return 'bg-blue-100 text-blue-800';
    case TechnicalDebtStatus.SCHEDULED:
      return 'bg-purple-100 text-purple-800';
    case TechnicalDebtStatus.IN_PROGRESS:
      return 'bg-indigo-100 text-indigo-800';
    case TechnicalDebtStatus.RESOLVED:
      return 'bg-green-100 text-green-800';
    case TechnicalDebtStatus.ACCEPTED:
      return 'bg-gray-100 text-gray-800';
    case TechnicalDebtStatus.DEFERRED:
      return 'bg-red-100 text-red-800';
    default:
      return 'bg-gray-100 text-gray-800';
  }
}

/**
 * Gets CSS classes for an impact badge
 * 
 * @param impact - Impact level (1-10)
 * @returns CSS classes string
 */
function getImpactBadgeClasses(impact: number): string {
  if (impact >= 8) return 'bg-red-100 text-red-800';
  if (impact >= 5) return 'bg-yellow-100 text-yellow-800';
  return 'bg-green-100 text-green-800';
}

/**
 * Gets CSS classes for an urgency badge
 * 
 * @param urgency - Urgency level (1-10)
 * @returns CSS classes string
 */
function getUrgencyBadgeClasses(urgency: number): string {
  if (urgency >= 8) return 'bg-red-100 text-red-800';
  if (urgency >= 5) return 'bg-yellow-100 text-yellow-800';
  return 'bg-green-100 text-green-800';
}

/**
 * Gets a color for a status bar
 * 
 * @param status - Technical debt status
 * @returns CSS color class
 */
function getStatusColor(status: TechnicalDebtStatus): string {
  switch (status) {
    case TechnicalDebtStatus.IDENTIFIED:
      return 'bg-yellow-500';
    case TechnicalDebtStatus.ACKNOWLEDGED:
      return 'bg-blue-500';
    case TechnicalDebtStatus.SCHEDULED:
      return 'bg-purple-500';
    case TechnicalDebtStatus.IN_PROGRESS:
      return 'bg-indigo-500';
    case TechnicalDebtStatus.RESOLVED:
      return 'bg-green-500';
    case TechnicalDebtStatus.ACCEPTED:
      return 'bg-gray-500';
    case TechnicalDebtStatus.DEFERRED:
      return 'bg-red-500';
    default:
      return 'bg-gray-500';
  }
}

/**
 * Gets a color for a type bar
 * 
 * @param type - Technical debt type
 * @returns CSS color class
 */
function getTypeColor(type: TechnicalDebtType): string {
  switch (type) {
    case TechnicalDebtType.CODE:
      return 'bg-blue-500';
    case TechnicalDebtType.ARCHITECTURE:
      return 'bg-purple-500';
    case TechnicalDebtType.DOCUMENTATION:
      return 'bg-green-500';
    case TechnicalDebtType.TESTING:
      return 'bg-yellow-500';
    case TechnicalDebtType.INFRASTRUCTURE:
      return 'bg-indigo-500';
    case TechnicalDebtType.DEPENDENCIES:
      return 'bg-pink-500';
    case TechnicalDebtType.PERFORMANCE:
      return 'bg-orange-500';
    case TechnicalDebtType.SECURITY:
      return 'bg-red-500';
    case TechnicalDebtType.ACCESSIBILITY:
      return 'bg-teal-500';
    case TechnicalDebtType.UX:
      return 'bg-cyan-500';
    case TechnicalDebtType.OTHER:
      return 'bg-gray-500';
    default:
      return 'bg-gray-500';
  }
}

/**
 * Gets a color for a debt score
 * 
 * @param score - Debt score
 * @returns CSS color class
 */
function getDebtScoreColor(score: number): string {
  if (score > 300) return 'text-red-500';
  if (score > 150) return 'text-yellow-500';
  return 'text-green-500';
}

/**
 * Gets a color for an impact score
 * 
 * @param impact - Impact score (1-10)
 * @returns CSS color class
 */
function getImpactColor(impact: number): string {
  if (impact >= 8) return 'text-red-500';
  if (impact >= 5) return 'text-yellow-500';
  return 'text-green-500';
}