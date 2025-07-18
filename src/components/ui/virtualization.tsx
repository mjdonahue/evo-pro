import React, { useState, useRef, useEffect, useCallback, useMemo } from 'react';

/**
 * Props for the VirtualList component
 */
export interface VirtualListProps<T> {
  /** Array of items to render */
  items: T[];
  /** Function to render an item */
  renderItem: (item: T, index: number) => React.ReactNode;
  /** Height of the container in pixels */
  height: number;
  /** Fixed height of each item in pixels (for fixed-height mode) */
  itemHeight?: number;
  /** Function to get the height of an item (for variable-height mode) */
  getItemHeight?: (item: T, index: number) => number;
  /** Number of items to render beyond the visible area (buffer) */
  overscan?: number;
  /** Additional CSS class name */
  className?: string;
  /** Additional inline styles */
  style?: React.CSSProperties;
  /** Callback when the visible range changes */
  onVisibleRangeChange?: (startIndex: number, endIndex: number) => void;
}

/**
 * A virtualized list component that efficiently renders large datasets
 * by only rendering items that are visible in the viewport.
 */
export function VirtualList<T>({
  items,
  renderItem,
  height,
  itemHeight: fixedItemHeight,
  getItemHeight,
  overscan = 5,
  className = '',
  style = {},
  onVisibleRangeChange,
}: VirtualListProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  
  // Determine if we're using fixed or variable height mode
  const isFixedHeight = fixedItemHeight !== undefined;
  const defaultItemHeight = fixedItemHeight || 50;
  
  // For variable height mode, we need to keep track of item heights and positions
  const [itemHeights, setItemHeights] = useState<number[]>([]);
  const [itemPositions, setItemPositions] = useState<number[]>([]);
  
  // Calculate item positions for variable height mode
  useEffect(() => {
    if (!isFixedHeight) {
      const heights = items.map((item, index) => 
        getItemHeight ? getItemHeight(item, index) : defaultItemHeight
      );
      
      const positions = [0];
      for (let i = 1; i < heights.length; i++) {
        positions[i] = positions[i - 1] + heights[i - 1];
      }
      
      setItemHeights(heights);
      setItemPositions(positions);
    }
  }, [items, getItemHeight, isFixedHeight, defaultItemHeight]);
  
  // Calculate the total height of all items
  const totalHeight = useMemo(() => {
    if (isFixedHeight) {
      return items.length * defaultItemHeight;
    } else {
      return itemHeights.length > 0 
        ? itemPositions[itemPositions.length - 1] + itemHeights[itemHeights.length - 1]
        : 0;
    }
  }, [items.length, defaultItemHeight, isFixedHeight, itemHeights, itemPositions]);
  
  // Calculate the visible range of items
  const { startIndex, endIndex } = useMemo(() => {
    if (items.length === 0) {
      return { startIndex: 0, endIndex: 0 };
    }
    
    let start = 0;
    let end = 0;
    
    if (isFixedHeight) {
      start = Math.floor(scrollTop / defaultItemHeight);
      end = Math.min(
        items.length - 1,
        Math.floor((scrollTop + height) / defaultItemHeight)
      );
    } else {
      // Binary search to find the first visible item
      let low = 0;
      let high = items.length - 1;
      
      while (low <= high) {
        const mid = Math.floor((low + high) / 2);
        const position = itemPositions[mid];
        
        if (position < scrollTop) {
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }
      
      start = Math.max(0, high);
      
      // Find the last visible item
      low = start;
      high = items.length - 1;
      
      while (low <= high) {
        const mid = Math.floor((low + high) / 2);
        const position = itemPositions[mid];
        
        if (position <= scrollTop + height) {
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }
      
      end = Math.min(items.length - 1, low);
    }
    
    // Apply overscan
    start = Math.max(0, start - overscan);
    end = Math.min(items.length - 1, end + overscan);
    
    return { startIndex: start, endIndex: end };
  }, [scrollTop, height, items.length, isFixedHeight, defaultItemHeight, itemPositions, overscan]);
  
  // Notify when the visible range changes
  useEffect(() => {
    onVisibleRangeChange?.(startIndex, endIndex);
  }, [startIndex, endIndex, onVisibleRangeChange]);
  
  // Handle scroll events
  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);
  
  // Render the visible items
  const visibleItems = useMemo(() => {
    return items.slice(startIndex, endIndex + 1).map((item, index) => {
      const actualIndex = startIndex + index;
      const itemStyle: React.CSSProperties = {};
      
      if (isFixedHeight) {
        itemStyle.height = `${defaultItemHeight}px`;
        itemStyle.top = `${actualIndex * defaultItemHeight}px`;
      } else {
        itemStyle.height = `${itemHeights[actualIndex]}px`;
        itemStyle.top = `${itemPositions[actualIndex]}px`;
      }
      
      return (
        <div 
          key={actualIndex} 
          className="virtual-list-item absolute left-0 right-0"
          style={itemStyle}
        >
          {renderItem(item, actualIndex)}
        </div>
      );
    });
  }, [items, startIndex, endIndex, renderItem, isFixedHeight, defaultItemHeight, itemHeights, itemPositions]);
  
  return (
    <div
      ref={containerRef}
      className={`virtual-list relative overflow-auto ${className}`}
      style={{ height: `${height}px`, ...style }}
      onScroll={handleScroll}
    >
      <div 
        className="virtual-list-inner relative"
        style={{ height: `${totalHeight}px` }}
      >
        {visibleItems}
      </div>
    </div>
  );
}

/**
 * Props for the VirtualGrid component
 */
export interface VirtualGridProps<T> {
  /** Array of items to render */
  items: T[];
  /** Function to render an item */
  renderItem: (item: T, index: number) => React.ReactNode;
  /** Height of the container in pixels */
  height: number;
  /** Width of the container in pixels (or 'auto' to use container width) */
  width?: number | 'auto';
  /** Height of each row in pixels */
  rowHeight: number;
  /** Width of each column in pixels */
  columnWidth: number;
  /** Number of columns to display */
  columnCount?: number;
  /** Number of items to render beyond the visible area (buffer) */
  overscan?: number;
  /** Additional CSS class name */
  className?: string;
  /** Additional inline styles */
  style?: React.CSSProperties;
  /** Callback when the visible range changes */
  onVisibleRangeChange?: (startIndex: number, endIndex: number) => void;
}

/**
 * A virtualized grid component that efficiently renders large datasets
 * in a grid layout by only rendering items that are visible in the viewport.
 */
export function VirtualGrid<T>({
  items,
  renderItem,
  height,
  width = 'auto',
  rowHeight,
  columnWidth,
  columnCount,
  overscan = 5,
  className = '',
  style = {},
  onVisibleRangeChange,
}: VirtualGridProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [containerWidth, setContainerWidth] = useState(0);
  
  // Calculate the number of columns based on container width if not specified
  const columns = useMemo(() => {
    if (columnCount !== undefined) {
      return columnCount;
    }
    
    if (width !== 'auto') {
      return Math.floor(width / columnWidth);
    }
    
    return Math.max(1, Math.floor(containerWidth / columnWidth));
  }, [columnCount, width, columnWidth, containerWidth]);
  
  // Calculate the number of rows
  const rowCount = Math.ceil(items.length / columns);
  
  // Calculate the total height of the grid
  const totalHeight = rowCount * rowHeight;
  
  // Calculate the visible range of rows
  const { startRow, endRow } = useMemo(() => {
    const start = Math.floor(scrollTop / rowHeight);
    const end = Math.min(
      rowCount - 1,
      Math.floor((scrollTop + height) / rowHeight)
    );
    
    // Apply overscan
    const startWithOverscan = Math.max(0, start - overscan);
    const endWithOverscan = Math.min(rowCount - 1, end + overscan);
    
    return { startRow: startWithOverscan, endRow: endWithOverscan };
  }, [scrollTop, height, rowHeight, rowCount, overscan]);
  
  // Calculate the visible range of items
  const { startIndex, endIndex } = useMemo(() => {
    const start = startRow * columns;
    const end = Math.min(items.length - 1, (endRow + 1) * columns - 1);
    
    return { startIndex: start, endIndex: end };
  }, [startRow, endRow, columns, items.length]);
  
  // Notify when the visible range changes
  useEffect(() => {
    onVisibleRangeChange?.(startIndex, endIndex);
  }, [startIndex, endIndex, onVisibleRangeChange]);
  
  // Handle scroll events
  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);
  
  // Measure the container width if needed
  useEffect(() => {
    if (width === 'auto' && containerRef.current) {
      const resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          setContainerWidth(entry.contentRect.width);
        }
      });
      
      resizeObserver.observe(containerRef.current);
      
      return () => {
        resizeObserver.disconnect();
      };
    }
  }, [width]);
  
  // Render the visible items
  const visibleItems = useMemo(() => {
    const result = [];
    
    for (let rowIndex = startRow; rowIndex <= endRow; rowIndex++) {
      const startItemIndex = rowIndex * columns;
      
      for (let colIndex = 0; colIndex < columns; colIndex++) {
        const itemIndex = startItemIndex + colIndex;
        
        if (itemIndex < items.length) {
          const item = items[itemIndex];
          const left = colIndex * columnWidth;
          const top = rowIndex * rowHeight;
          
          result.push(
            <div 
              key={itemIndex} 
              className="virtual-grid-item absolute"
              style={{
                left: `${left}px`,
                top: `${top}px`,
                width: `${columnWidth}px`,
                height: `${rowHeight}px`,
              }}
            >
              {renderItem(item, itemIndex)}
            </div>
          );
        }
      }
    }
    
    return result;
  }, [items, startRow, endRow, columns, columnWidth, rowHeight, renderItem]);
  
  return (
    <div
      ref={containerRef}
      className={`virtual-grid relative overflow-auto ${className}`}
      style={{ 
        height: `${height}px`, 
        width: width === 'auto' ? '100%' : `${width}px`,
        ...style 
      }}
      onScroll={handleScroll}
    >
      <div 
        className="virtual-grid-inner relative"
        style={{ 
          height: `${totalHeight}px`,
          width: width === 'auto' ? '100%' : `${width}px`,
        }}
      >
        {visibleItems}
      </div>
    </div>
  );
}

/**
 * Hook that provides virtualization functionality for custom implementations
 * 
 * @param options - Configuration options for the virtualization
 * @returns Virtualization state and helper functions
 */
export function useVirtualization<T>({
  items,
  height,
  itemHeight: fixedItemHeight,
  getItemHeight,
  overscan = 5,
  scrollTop = 0,
}: {
  items: T[];
  height: number;
  itemHeight?: number;
  getItemHeight?: (item: T, index: number) => number;
  overscan?: number;
  scrollTop?: number;
}) {
  // Determine if we're using fixed or variable height mode
  const isFixedHeight = fixedItemHeight !== undefined;
  const defaultItemHeight = fixedItemHeight || 50;
  
  // For variable height mode, we need to keep track of item heights and positions
  const [itemHeights, setItemHeights] = useState<number[]>([]);
  const [itemPositions, setItemPositions] = useState<number[]>([]);
  
  // Calculate item positions for variable height mode
  useEffect(() => {
    if (!isFixedHeight) {
      const heights = items.map((item, index) => 
        getItemHeight ? getItemHeight(item, index) : defaultItemHeight
      );
      
      const positions = [0];
      for (let i = 1; i < heights.length; i++) {
        positions[i] = positions[i - 1] + heights[i - 1];
      }
      
      setItemHeights(heights);
      setItemPositions(positions);
    }
  }, [items, getItemHeight, isFixedHeight, defaultItemHeight]);
  
  // Calculate the total height of all items
  const totalHeight = useMemo(() => {
    if (isFixedHeight) {
      return items.length * defaultItemHeight;
    } else {
      return itemHeights.length > 0 
        ? itemPositions[itemPositions.length - 1] + itemHeights[itemHeights.length - 1]
        : 0;
    }
  }, [items.length, defaultItemHeight, isFixedHeight, itemHeights, itemPositions]);
  
  // Calculate the visible range of items
  const { startIndex, endIndex } = useMemo(() => {
    if (items.length === 0) {
      return { startIndex: 0, endIndex: 0 };
    }
    
    let start = 0;
    let end = 0;
    
    if (isFixedHeight) {
      start = Math.floor(scrollTop / defaultItemHeight);
      end = Math.min(
        items.length - 1,
        Math.floor((scrollTop + height) / defaultItemHeight)
      );
    } else {
      // Binary search to find the first visible item
      let low = 0;
      let high = items.length - 1;
      
      while (low <= high) {
        const mid = Math.floor((low + high) / 2);
        const position = itemPositions[mid];
        
        if (position < scrollTop) {
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }
      
      start = Math.max(0, high);
      
      // Find the last visible item
      low = start;
      high = items.length - 1;
      
      while (low <= high) {
        const mid = Math.floor((low + high) / 2);
        const position = itemPositions[mid];
        
        if (position <= scrollTop + height) {
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }
      
      end = Math.min(items.length - 1, low);
    }
    
    // Apply overscan
    start = Math.max(0, start - overscan);
    end = Math.min(items.length - 1, end + overscan);
    
    return { startIndex: start, endIndex: end };
  }, [scrollTop, height, items.length, isFixedHeight, defaultItemHeight, itemPositions, overscan]);
  
  // Get the visible items
  const visibleItems = useMemo(() => {
    return items.slice(startIndex, endIndex + 1);
  }, [items, startIndex, endIndex]);
  
  // Get the position and size of an item
  const getItemStyle = useCallback((index: number): React.CSSProperties => {
    if (isFixedHeight) {
      return {
        position: 'absolute',
        top: `${index * defaultItemHeight}px`,
        height: `${defaultItemHeight}px`,
        left: 0,
        right: 0,
      };
    } else {
      return {
        position: 'absolute',
        top: `${itemPositions[index]}px`,
        height: `${itemHeights[index]}px`,
        left: 0,
        right: 0,
      };
    }
  }, [isFixedHeight, defaultItemHeight, itemPositions, itemHeights]);
  
  return {
    visibleItems,
    visibleItemsWithIndex: visibleItems.map((item, i) => ({ 
      item, 
      index: startIndex + i 
    })),
    startIndex,
    endIndex,
    totalHeight,
    getItemStyle,
    itemHeights,
    itemPositions,
  };
}