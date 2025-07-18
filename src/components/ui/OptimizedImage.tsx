import React, { useState, useEffect, useRef } from 'react';
import { useVisibilityQuery } from '../../lib/api/lazyData';

/**
 * Props for the OptimizedImage component
 */
export interface OptimizedImageProps extends Omit<React.ImgHTMLAttributes<HTMLImageElement>, 'src'> {
  /** Source URL of the image */
  src: string;
  /** Alternative text for the image */
  alt: string;
  /** Whether to lazy load the image (default: true) */
  lazyLoad?: boolean;
  /** Placeholder to show while the image is loading */
  placeholder?: React.ReactNode;
  /** Fallback to show if the image fails to load */
  fallback?: React.ReactNode;
  /** Array of source objects for responsive images */
  sources?: Array<{
    /** Source URL */
    src: string;
    /** Media query */
    media?: string;
    /** Image format (e.g., 'image/webp') */
    type?: string;
  }>;
  /** Function to call when the image is loaded */
  onLoad?: () => void;
  /** Function to call if the image fails to load */
  onError?: () => void;
  /** Root margin for intersection observer (for lazy loading) */
  rootMargin?: string;
  /** Whether to use blur-up technique for loading */
  blurUp?: boolean;
  /** Low-quality image placeholder URL (for blur-up technique) */
  lqip?: string;
  /** Whether to use WebP format if supported by the browser */
  useWebP?: boolean;
  /** Whether to use AVIF format if supported by the browser */
  useAVIF?: boolean;
}

/**
 * A component that optimizes image loading with features like:
 * - Lazy loading
 * - Responsive images
 * - Modern format support (WebP, AVIF)
 * - Blur-up loading technique
 * - Fallback and placeholder support
 */
export const OptimizedImage: React.FC<OptimizedImageProps> = ({
  src,
  alt,
  lazyLoad = true,
  placeholder,
  fallback,
  sources = [],
  onLoad,
  onError,
  rootMargin = '200px',
  blurUp = false,
  lqip,
  useWebP = true,
  useAVIF = true,
  className = '',
  style = {},
  ...imgProps
}) => {
  const imgRef = useRef<HTMLImageElement>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [hasError, setHasError] = useState(false);
  const [supportsWebP, setSupportsWebP] = useState(false);
  const [supportsAVIF, setSupportsAVIF] = useState(false);
  
  // Check for WebP and AVIF support
  useEffect(() => {
    // Check WebP support
    const webpImage = new Image();
    webpImage.onload = () => setSupportsWebP(true);
    webpImage.onerror = () => setSupportsWebP(false);
    webpImage.src = 'data:image/webp;base64,UklGRiQAAABXRUJQVlA4IBgAAAAwAQCdASoBAAEAAwA0JaQAA3AA/vuUAAA=';
    
    // Check AVIF support
    const avifImage = new Image();
    avifImage.onload = () => setSupportsAVIF(true);
    avifImage.onerror = () => setSupportsAVIF(false);
    avifImage.src = 'data:image/avif;base64,AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A=';
  }, []);
  
  // Use visibility query for lazy loading
  const { isVisible } = useVisibilityQuery(
    async () => true,
    imgRef,
    [],
    {
      enabled: lazyLoad,
      rootMargin,
      keepData: true,
    }
  );
  
  // Determine if we should load the image
  const shouldLoad = !lazyLoad || isVisible;
  
  // Handle image load event
  const handleLoad = () => {
    setIsLoaded(true);
    onLoad?.();
  };
  
  // Handle image error event
  const handleError = () => {
    setHasError(true);
    onError?.();
  };
  
  // Generate srcSet for responsive images
  const getSrcSet = () => {
    if (!imgProps.srcSet && sources.length > 0) {
      return sources
        .filter(source => !source.type) // Filter out sources with type (they go in <source> elements)
        .map(source => `${source.src} ${source.media?.replace(/[^\d]/g, '') || ''}w`)
        .join(', ');
    }
    return imgProps.srcSet;
  };
  
  // Get best source based on browser support
  const getBestSource = () => {
    if (useAVIF && supportsAVIF) {
      const avifSource = sources.find(s => s.type === 'image/avif');
      if (avifSource) return avifSource.src;
    }
    
    if (useWebP && supportsWebP) {
      const webpSource = sources.find(s => s.type === 'image/webp');
      if (webpSource) return webpSource.src;
    }
    
    return src;
  };
  
  // Apply blur-up effect styles
  const getBlurStyles = (): React.CSSProperties => {
    if (blurUp && !isLoaded && lqip) {
      return {
        filter: 'blur(20px)',
        transition: 'filter 0.3s ease-out',
      };
    }
    
    if (blurUp && isLoaded) {
      return {
        filter: 'blur(0)',
        transition: 'filter 0.3s ease-out',
      };
    }
    
    return {};
  };
  
  // Render placeholder while loading
  if (shouldLoad && !isLoaded && !hasError && placeholder) {
    return (
      <div className={`optimized-image-placeholder ${className}`} style={style}>
        {placeholder}
      </div>
    );
  }
  
  // Render fallback if error
  if (hasError && fallback) {
    return (
      <div className={`optimized-image-fallback ${className}`} style={style}>
        {fallback}
      </div>
    );
  }
  
  // Render the image with picture element for format support
  return (
    <picture>
      {shouldLoad && useAVIF && supportsAVIF && sources.some(s => s.type === 'image/avif') && (
        sources
          .filter(s => s.type === 'image/avif')
          .map((source, index) => (
            <source 
              key={`avif-${index}`} 
              srcSet={source.src} 
              type="image/avif" 
              media={source.media}
            />
          ))
      )}
      
      {shouldLoad && useWebP && supportsWebP && sources.some(s => s.type === 'image/webp') && (
        sources
          .filter(s => s.type === 'image/webp')
          .map((source, index) => (
            <source 
              key={`webp-${index}`} 
              srcSet={source.src} 
              type="image/webp" 
              media={source.media}
            />
          ))
      )}
      
      {shouldLoad && sources.some(s => s.media && !s.type) && (
        sources
          .filter(s => s.media && !s.type)
          .map((source, index) => (
            <source 
              key={`responsive-${index}`} 
              srcSet={source.src} 
              media={source.media}
            />
          ))
      )}
      
      <img
        ref={imgRef}
        src={shouldLoad ? (blurUp && !isLoaded && lqip ? lqip : getBestSource()) : ''}
        alt={alt}
        onLoad={handleLoad}
        onError={handleError}
        className={className}
        style={{
          ...style,
          ...getBlurStyles(),
          opacity: shouldLoad ? 1 : 0,
        }}
        srcSet={shouldLoad ? getSrcSet() : undefined}
        {...imgProps}
      />
    </picture>
  );
};

/**
 * Hook to preload an image
 * 
 * @param src - Source URL of the image to preload
 * @returns Object with loading state
 */
export function useImagePreload(src: string) {
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState(false);
  
  useEffect(() => {
    const img = new Image();
    
    img.onload = () => {
      setLoaded(true);
      setError(false);
    };
    
    img.onerror = () => {
      setLoaded(false);
      setError(true);
    };
    
    img.src = src;
    
    return () => {
      img.onload = null;
      img.onerror = null;
    };
  }, [src]);
  
  return { loaded, error };
}

/**
 * Utility to generate a low-quality image placeholder (LQIP) URL
 * 
 * @param src - Original image URL
 * @param width - Width of the LQIP
 * @returns LQIP URL
 */
export function generateLQIP(src: string, width = 20): string {
  // This is a simplified example. In a real application, you would use a server-side
  // image processing service or a CDN that supports on-the-fly transformations.
  if (src.includes('?')) {
    return `${src}&w=${width}&q=10&blur=10`;
  }
  return `${src}?w=${width}&q=10&blur=10`;
}

/**
 * Utility to generate responsive image sources
 * 
 * @param src - Base image URL
 * @param widths - Array of widths for responsive images
 * @param formats - Array of formats to generate (e.g., ['webp', 'jpg'])
 * @returns Array of source objects
 */
export function generateResponsiveSources(
  src: string,
  widths: number[] = [640, 768, 1024, 1366, 1600, 1920],
  formats: string[] = ['webp', 'avif', 'jpg']
): Array<{ src: string; media?: string; type?: string }> {
  const sources: Array<{ src: string; media?: string; type?: string }> = [];
  
  // Generate sources for each format
  formats.forEach(format => {
    // This is a simplified example. In a real application, you would use a server-side
    // image processing service or a CDN that supports on-the-fly transformations.
    const formatSrc = src.replace(/\.[^/.]+$/, `.${format}`);
    const mimeType = `image/${format}`;
    
    // Add a source for each width
    widths.forEach(width => {
      const mediaQuery = `(max-width: ${width}px)`;
      const sizeParam = src.includes('?') ? `&w=${width}` : `?w=${width}`;
      
      sources.push({
        src: `${formatSrc}${sizeParam}`,
        media: mediaQuery,
        type: mimeType,
      });
    });
    
    // Add a full-size source for each format
    sources.push({
      src: formatSrc,
      type: mimeType,
    });
  });
  
  return sources;
}