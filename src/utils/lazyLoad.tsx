import React, { lazy, Suspense } from 'react';
import { LoadingSpinner } from '../components/ui/LoadingSpinner';

// Default fallback component to show while loading
const DefaultFallback = () => (
  <div className="w-full h-full flex items-center justify-center">
    <LoadingSpinner size="large" />
  </div>
);

/**
 * Creates a lazy-loaded component with a suspense wrapper and fallback UI
 * 
 * @param importFunc - Dynamic import function that returns the component
 * @param fallback - Optional custom fallback component to show while loading
 * @returns A wrapped component that will be lazy loaded
 */
export function lazyLoad(
  importFunc: () => Promise<{ default: React.ComponentType<any> }>,
  fallback: React.ReactNode = <DefaultFallback />
) {
  const LazyComponent = lazy(importFunc);
  
  return (props: any) => (
    <Suspense fallback={fallback}>
      <LazyComponent {...props} />
    </Suspense>
  );
}