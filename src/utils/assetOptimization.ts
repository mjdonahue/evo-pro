/**
 * Utilities for optimizing various types of assets
 */

/**
 * Options for font loading optimization
 */
export interface FontLoadingOptions {
  /** Font family name */
  family: string;
  /** Font weights to load */
  weights?: number[];
  /** Font styles to load */
  styles?: ('normal' | 'italic')[];
  /** Font display strategy */
  display?: 'auto' | 'block' | 'swap' | 'fallback' | 'optional';
  /** Timeout for font loading (in milliseconds) */
  timeout?: number;
}

/**
 * Optimizes font loading using the Font Loading API
 * 
 * @param options - Font loading options
 * @returns Promise that resolves when fonts are loaded or timeout is reached
 */
export async function optimizeFontLoading(options: FontLoadingOptions): Promise<boolean> {
  const {
    family,
    weights = [400],
    styles = ['normal'],
    display = 'swap',
    timeout = 3000,
  } = options;
  
  // Check if Font Loading API is available
  if (!('FontFace' in window)) {
    console.warn('Font Loading API is not supported in this browser');
    return false;
  }
  
  try {
    // Add font-display to the document if not already present
    const styleElement = document.getElementById('font-display-style') || document.createElement('style');
    if (!document.getElementById('font-display-style')) {
      styleElement.id = 'font-display-style';
      styleElement.textContent = `
        @font-face {
          font-family: ${family};
          font-display: ${display};
        }
      `;
      document.head.appendChild(styleElement);
    }
    
    // Create a promise for each font variation
    const fontPromises = weights.flatMap(weight => 
      styles.map(style => {
        const font = new FontFace(family, `local(${family})`, {
          weight: weight.toString(),
          style,
        });
        
        // Add the font to the document fonts
        document.fonts.add(font);
        
        // Load the font with a timeout
        return Promise.race([
          font.load().then(() => true),
          new Promise<boolean>(resolve => setTimeout(() => resolve(false), timeout)),
        ]);
      })
    );
    
    // Wait for all fonts to load or timeout
    const results = await Promise.all(fontPromises);
    return results.every(result => result);
  } catch (error) {
    console.error('Error optimizing font loading:', error);
    return false;
  }
}

/**
 * Preloads critical assets to improve performance
 * 
 * @param assets - Array of asset URLs to preload
 * @param type - Type of asset (e.g., 'image', 'style', 'script', 'font')
 * @param options - Additional options for the preload link
 */
export function preloadCriticalAssets(
  assets: string[],
  type: 'image' | 'style' | 'script' | 'font' | 'fetch',
  options: { crossOrigin?: 'anonymous' | 'use-credentials'; as?: string } = {}
): void {
  assets.forEach(asset => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.href = asset;
    
    // Set appropriate 'as' attribute based on type
    if (options.as) {
      link.setAttribute('as', options.as);
    } else {
      link.setAttribute('as', type);
    }
    
    // Add crossorigin attribute if specified
    if (options.crossOrigin) {
      link.crossOrigin = options.crossOrigin;
    }
    
    // Add the preload link to the document head
    document.head.appendChild(link);
  });
}

/**
 * Prefetches assets that will be needed in the future
 * 
 * @param assets - Array of asset URLs to prefetch
 */
export function prefetchAssets(assets: string[]): void {
  assets.forEach(asset => {
    const link = document.createElement('link');
    link.rel = 'prefetch';
    link.href = asset;
    document.head.appendChild(link);
  });
}

/**
 * Preconnects to origins that will be used for asset loading
 * 
 * @param origins - Array of origin URLs to preconnect to
 * @param crossOrigin - Whether to include credentials in the preconnect
 */
export function preconnectToOrigins(
  origins: string[],
  crossOrigin?: 'anonymous' | 'use-credentials'
): void {
  origins.forEach(origin => {
    const link = document.createElement('link');
    link.rel = 'preconnect';
    link.href = origin;
    
    if (crossOrigin) {
      link.crossOrigin = crossOrigin;
    }
    
    document.head.appendChild(link);
  });
}

/**
 * Loads a script dynamically with optimized loading
 * 
 * @param src - Script URL
 * @param options - Script loading options
 * @returns Promise that resolves when the script is loaded
 */
export function loadScript(
  src: string,
  options: {
    async?: boolean;
    defer?: boolean;
    crossOrigin?: string;
    integrity?: string;
    type?: string;
    id?: string;
  } = {}
): Promise<HTMLScriptElement> {
  return new Promise((resolve, reject) => {
    // Check if script is already loaded
    const existingScript = document.querySelector(`script[src="${src}"]`);
    if (existingScript) {
      resolve(existingScript as HTMLScriptElement);
      return;
    }
    
    // Create script element
    const script = document.createElement('script');
    script.src = src;
    
    // Add attributes based on options
    if (options.async) script.async = true;
    if (options.defer) script.defer = true;
    if (options.crossOrigin) script.crossOrigin = options.crossOrigin;
    if (options.integrity) script.integrity = options.integrity;
    if (options.type) script.type = options.type;
    if (options.id) script.id = options.id;
    
    // Set up load and error handlers
    script.onload = () => resolve(script);
    script.onerror = () => reject(new Error(`Failed to load script: ${src}`));
    
    // Add script to document
    document.head.appendChild(script);
  });
}

/**
 * Loads a CSS stylesheet dynamically with optimized loading
 * 
 * @param href - Stylesheet URL
 * @param options - Stylesheet loading options
 * @returns Promise that resolves when the stylesheet is loaded
 */
export function loadStylesheet(
  href: string,
  options: {
    media?: string;
    crossOrigin?: string;
    integrity?: string;
    id?: string;
  } = {}
): Promise<HTMLLinkElement> {
  return new Promise((resolve, reject) => {
    // Check if stylesheet is already loaded
    const existingLink = document.querySelector(`link[href="${href}"]`);
    if (existingLink) {
      resolve(existingLink as HTMLLinkElement);
      return;
    }
    
    // Create link element
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = href;
    
    // Add attributes based on options
    if (options.media) link.media = options.media;
    if (options.crossOrigin) link.crossOrigin = options.crossOrigin;
    if (options.integrity) link.integrity = options.integrity;
    if (options.id) link.id = options.id;
    
    // Set up load and error handlers
    link.onload = () => resolve(link);
    link.onerror = () => reject(new Error(`Failed to load stylesheet: ${href}`));
    
    // Add link to document
    document.head.appendChild(link);
  });
}

/**
 * Optimizes CSS delivery by inlining critical CSS and loading non-critical CSS asynchronously
 * 
 * @param criticalCss - Critical CSS to inline
 * @param nonCriticalCssUrls - URLs of non-critical CSS to load asynchronously
 */
export function optimizeCssDelivery(
  criticalCss: string,
  nonCriticalCssUrls: string[] = []
): void {
  // Inline critical CSS
  const style = document.createElement('style');
  style.textContent = criticalCss;
  document.head.appendChild(style);
  
  // Load non-critical CSS asynchronously
  nonCriticalCssUrls.forEach(url => {
    const link = document.createElement('link');
    link.rel = 'preload';
    link.as = 'style';
    link.href = url;
    link.onload = function() {
      // Convert preload to stylesheet once loaded
      this.onload = null;
      this.rel = 'stylesheet';
    };
    document.head.appendChild(link);
  });
}

/**
 * Detects network connection quality and returns appropriate asset quality level
 * 
 * @returns Promise that resolves to a quality level (low, medium, high)
 */
export async function detectNetworkQuality(): Promise<'low' | 'medium' | 'high'> {
  // Check if Network Information API is available
  if ('connection' in navigator && navigator.connection) {
    const connection = navigator.connection as any;
    
    // Use effectiveType if available
    if (connection.effectiveType) {
      switch (connection.effectiveType) {
        case 'slow-2g':
        case '2g':
          return 'low';
        case '3g':
          return 'medium';
        case '4g':
          return 'high';
        default:
          return 'medium';
      }
    }
    
    // Fall back to downlink if available
    if (connection.downlink !== undefined) {
      if (connection.downlink < 1) return 'low';
      if (connection.downlink < 5) return 'medium';
      return 'high';
    }
  }
  
  // If Network Information API is not available, perform a simple speed test
  try {
    const startTime = Date.now();
    const response = await fetch('/favicon.ico', { method: 'HEAD' });
    const endTime = Date.now();
    
    if (!response.ok) return 'medium'; // Default to medium if test fails
    
    const duration = endTime - startTime;
    
    if (duration > 500) return 'low';
    if (duration > 100) return 'medium';
    return 'high';
  } catch (error) {
    console.warn('Network quality detection failed:', error);
    return 'medium'; // Default to medium quality
  }
}