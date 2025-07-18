/**
 * Device Capability Detection and Adaptation
 * 
 * This module provides functionality for detecting device capabilities
 * and adapting application behavior based on those capabilities.
 */

import { SyncProtocol } from './crossDeviceSync';

/**
 * Device type enumeration
 */
export enum DeviceType {
  DESKTOP = 'desktop',
  LAPTOP = 'laptop',
  TABLET = 'tablet',
  MOBILE = 'mobile',
  WEB = 'web',
  OTHER = 'other'
}

/**
 * Network type enumeration
 */
export enum NetworkType {
  WIFI = 'wifi',
  CELLULAR = 'cellular',
  ETHERNET = 'ethernet',
  OFFLINE = 'offline',
  UNKNOWN = 'unknown'
}

/**
 * Network quality enumeration
 */
export enum NetworkQuality {
  EXCELLENT = 'excellent',
  GOOD = 'good',
  FAIR = 'fair',
  POOR = 'poor',
  UNKNOWN = 'unknown'
}

/**
 * Device capabilities interface
 */
export interface DeviceCapabilities {
  /** Device type */
  deviceType: DeviceType;
  /** Operating system */
  os: string;
  /** Operating system version */
  osVersion: string;
  /** Application version */
  appVersion: string;
  /** Available storage space in bytes */
  storageBytes?: number;
  /** Available memory in bytes */
  memoryBytes?: number;
  /** Number of CPU cores */
  cpuCores?: number;
  /** CPU architecture */
  cpuArchitecture?: string;
  /** Network connection type */
  networkType?: NetworkType;
  /** Network connection quality */
  networkQuality?: NetworkQuality;
  /** Network bandwidth in bits per second */
  networkBandwidth?: number;
  /** Network latency in milliseconds */
  networkLatency?: number;
  /** Battery level (0-100) */
  batteryLevel?: number;
  /** Whether the device is charging */
  isCharging?: boolean;
  /** Screen width in pixels */
  screenWidth?: number;
  /** Screen height in pixels */
  screenHeight?: number;
  /** Device pixel ratio */
  pixelRatio?: number;
  /** Whether the device supports touch input */
  touchSupport?: boolean;
  /** Maximum supported sync protocol */
  maxSupportedProtocol?: SyncProtocol;
  /** Whether the device supports delta sync */
  supportsDeltaSync?: boolean;
  /** Whether the device supports CRDTs */
  supportsCRDT?: boolean;
  /** Whether the device supports operational transforms */
  supportsOT?: boolean;
  /** Whether the device supports hardware acceleration */
  supportsHardwareAcceleration?: boolean;
  /** Whether the device supports WebGL */
  supportsWebGL?: boolean;
  /** Whether the device supports WebAssembly */
  supportsWasm?: boolean;
  /** Whether the device supports IndexedDB */
  supportsIndexedDB?: boolean;
  /** Whether the device supports service workers */
  supportsServiceWorker?: boolean;
  /** Whether the device supports push notifications */
  supportsPushNotifications?: boolean;
  /** Whether the device supports background sync */
  supportsBackgroundSync?: boolean;
  /** Custom capabilities */
  custom?: Record<string, any>;
}

/**
 * Device capability tier
 */
export enum CapabilityTier {
  /** High-end device with excellent capabilities */
  HIGH = 'high',
  /** Mid-range device with good capabilities */
  MEDIUM = 'medium',
  /** Low-end device with limited capabilities */
  LOW = 'low',
  /** Very limited device */
  MINIMAL = 'minimal'
}

/**
 * Adaptation strategy for a specific capability tier
 */
export interface AdaptationStrategy {
  /** Tier this strategy applies to */
  tier: CapabilityTier;
  /** Description of the strategy */
  description: string;
  /** Maximum batch size for operations */
  maxBatchSize: number;
  /** Maximum cache size in bytes */
  maxCacheBytes: number;
  /** Number of worker threads to use */
  workerThreads: number;
  /** Whether to enable background processing */
  enableBackgroundProcessing: boolean;
  /** Whether to enable prefetching */
  enablePrefetching: boolean;
  /** Whether to enable high-resolution assets */
  enableHighResAssets: boolean;
  /** Whether to enable advanced visual effects */
  enableAdvancedVisuals: boolean;
  /** Whether to enable real-time updates */
  enableRealTimeUpdates: boolean;
  /** Polling interval in milliseconds */
  pollingIntervalMs: number;
  /** Compression level (0-9) */
  compressionLevel: number;
  /** Synchronization protocol to use */
  syncProtocol: SyncProtocol;
  /** Custom parameters */
  params: Record<string, any>;
}

/**
 * Default adaptation strategies for different capability tiers
 */
export const DEFAULT_ADAPTATION_STRATEGIES: Record<CapabilityTier, AdaptationStrategy> = {
  [CapabilityTier.HIGH]: {
    tier: CapabilityTier.HIGH,
    description: 'Strategy for high-end devices',
    maxBatchSize: 1000,
    maxCacheBytes: 1024 * 1024 * 1024, // 1 GB
    workerThreads: 8,
    enableBackgroundProcessing: true,
    enablePrefetching: true,
    enableHighResAssets: true,
    enableAdvancedVisuals: true,
    enableRealTimeUpdates: true,
    pollingIntervalMs: 1000,
    compressionLevel: 1, // Low compression, faster processing
    syncProtocol: SyncProtocol.VECTOR_CLOCK_SYNC,
    params: {}
  },
  [CapabilityTier.MEDIUM]: {
    tier: CapabilityTier.MEDIUM,
    description: 'Strategy for mid-range devices',
    maxBatchSize: 500,
    maxCacheBytes: 512 * 1024 * 1024, // 512 MB
    workerThreads: 4,
    enableBackgroundProcessing: true,
    enablePrefetching: true,
    enableHighResAssets: true,
    enableAdvancedVisuals: false,
    enableRealTimeUpdates: true,
    pollingIntervalMs: 2000,
    compressionLevel: 4, // Balanced compression
    syncProtocol: SyncProtocol.DELTA_SYNC,
    params: {}
  },
  [CapabilityTier.LOW]: {
    tier: CapabilityTier.LOW,
    description: 'Strategy for low-end devices',
    maxBatchSize: 100,
    maxCacheBytes: 128 * 1024 * 1024, // 128 MB
    workerThreads: 2,
    enableBackgroundProcessing: false,
    enablePrefetching: false,
    enableHighResAssets: false,
    enableAdvancedVisuals: false,
    enableRealTimeUpdates: false,
    pollingIntervalMs: 5000,
    compressionLevel: 7, // Higher compression, slower processing
    syncProtocol: SyncProtocol.INCREMENTAL_SYNC,
    params: {}
  },
  [CapabilityTier.MINIMAL]: {
    tier: CapabilityTier.MINIMAL,
    description: 'Strategy for very limited devices',
    maxBatchSize: 50,
    maxCacheBytes: 32 * 1024 * 1024, // 32 MB
    workerThreads: 1,
    enableBackgroundProcessing: false,
    enablePrefetching: false,
    enableHighResAssets: false,
    enableAdvancedVisuals: false,
    enableRealTimeUpdates: false,
    pollingIntervalMs: 10000,
    compressionLevel: 9, // Maximum compression
    syncProtocol: SyncProtocol.FULL_SYNC,
    params: {}
  }
};

/**
 * Options for device capability detection
 */
export interface CapabilityDetectionOptions {
  /** Whether to detect storage capabilities */
  detectStorage?: boolean;
  /** Whether to detect memory capabilities */
  detectMemory?: boolean;
  /** Whether to detect CPU capabilities */
  detectCPU?: boolean;
  /** Whether to detect network capabilities */
  detectNetwork?: boolean;
  /** Whether to detect battery status */
  detectBattery?: boolean;
  /** Whether to detect screen capabilities */
  detectScreen?: boolean;
  /** Whether to detect feature support */
  detectFeatureSupport?: boolean;
  /** Whether to periodically update capabilities */
  enablePeriodicUpdates?: boolean;
  /** Interval for periodic updates in milliseconds */
  updateIntervalMs?: number;
  /** Callback when capabilities change */
  onCapabilitiesChanged?: (capabilities: DeviceCapabilities) => void;
}

/**
 * Default options for capability detection
 */
export const DEFAULT_DETECTION_OPTIONS: Required<CapabilityDetectionOptions> = {
  detectStorage: true,
  detectMemory: true,
  detectCPU: true,
  detectNetwork: true,
  detectBattery: true,
  detectScreen: true,
  detectFeatureSupport: true,
  enablePeriodicUpdates: true,
  updateIntervalMs: 60000, // 1 minute
  onCapabilitiesChanged: () => {}
};

/**
 * Device capability detector
 */
export class DeviceCapabilityDetector {
  private options: Required<CapabilityDetectionOptions>;
  private capabilities: DeviceCapabilities;
  private updateInterval: number | null = null;

  /**
   * Creates a new device capability detector
   * @param options - Detection options
   */
  constructor(options: CapabilityDetectionOptions = {}) {
    this.options = { ...DEFAULT_DETECTION_OPTIONS, ...options };
    
    // Initialize with basic capabilities
    this.capabilities = {
      deviceType: DeviceType.OTHER,
      os: 'unknown',
      osVersion: 'unknown',
      appVersion: 'unknown'
    };
  }

  /**
   * Detects device capabilities
   * @returns Promise that resolves with the detected capabilities
   */
  async detectCapabilities(): Promise<DeviceCapabilities> {
    // Detect basic device information
    this.detectDeviceType();
    this.detectOSInfo();
    
    // Detect specific capabilities based on options
    if (this.options.detectStorage) {
      await this.detectStorageCapabilities();
    }
    
    if (this.options.detectMemory) {
      await this.detectMemoryCapabilities();
    }
    
    if (this.options.detectCPU) {
      await this.detectCPUCapabilities();
    }
    
    if (this.options.detectNetwork) {
      await this.detectNetworkCapabilities();
    }
    
    if (this.options.detectBattery) {
      await this.detectBatteryStatus();
    }
    
    if (this.options.detectScreen) {
      this.detectScreenCapabilities();
    }
    
    if (this.options.detectFeatureSupport) {
      this.detectFeatureSupport();
    }
    
    // Notify listeners
    this.options.onCapabilitiesChanged(this.capabilities);
    
    return { ...this.capabilities };
  }

  /**
   * Starts periodic capability updates
   */
  startPeriodicUpdates(): void {
    if (this.options.enablePeriodicUpdates && !this.updateInterval) {
      this.updateInterval = window.setInterval(
        () => this.detectCapabilities(),
        this.options.updateIntervalMs
      );
    }
  }

  /**
   * Stops periodic capability updates
   */
  stopPeriodicUpdates(): void {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
    }
  }

  /**
   * Gets the current device capabilities
   * @returns The current capabilities
   */
  getCapabilities(): DeviceCapabilities {
    return { ...this.capabilities };
  }

  /**
   * Determines the capability tier based on current capabilities
   * @returns The capability tier
   */
  getCapabilityTier(): CapabilityTier {
    const caps = this.capabilities;
    
    // Check for high-end device
    if (
      (caps.cpuCores && caps.cpuCores >= 8) &&
      (caps.memoryBytes && caps.memoryBytes >= 8 * 1024 * 1024 * 1024) && // 8 GB
      (caps.networkQuality === NetworkQuality.EXCELLENT || caps.networkQuality === NetworkQuality.GOOD)
    ) {
      return CapabilityTier.HIGH;
    }
    
    // Check for mid-range device
    if (
      (caps.cpuCores && caps.cpuCores >= 4) &&
      (caps.memoryBytes && caps.memoryBytes >= 4 * 1024 * 1024 * 1024) && // 4 GB
      (caps.networkQuality !== NetworkQuality.POOR)
    ) {
      return CapabilityTier.MEDIUM;
    }
    
    // Check for low-end device
    if (
      (caps.cpuCores && caps.cpuCores >= 2) &&
      (caps.memoryBytes && caps.memoryBytes >= 2 * 1024 * 1024 * 1024) // 2 GB
    ) {
      return CapabilityTier.LOW;
    }
    
    // Default to minimal
    return CapabilityTier.MINIMAL;
  }

  /**
   * Gets the adaptation strategy for the current device
   * @returns The adaptation strategy
   */
  getAdaptationStrategy(): AdaptationStrategy {
    const tier = this.getCapabilityTier();
    return { ...DEFAULT_ADAPTATION_STRATEGIES[tier] };
  }

  /**
   * Detects the device type
   */
  private detectDeviceType(): void {
    // Use User-Agent and screen size to determine device type
    const ua = navigator.userAgent;
    const width = window.innerWidth;
    const height = window.innerHeight;
    
    if (/Mobi|Android|iPhone|iPad|iPod/i.test(ua)) {
      // Mobile or tablet
      if (Math.max(width, height) >= 768) {
        this.capabilities.deviceType = DeviceType.TABLET;
      } else {
        this.capabilities.deviceType = DeviceType.MOBILE;
      }
    } else {
      // Desktop or laptop
      if (
        /Macintosh|MacIntel|MacPPC|Mac68K/i.test(ua) ||
        /Win32|Win64|Windows|WinCE/i.test(ua) ||
        /Linux/i.test(ua)
      ) {
        // Assume laptop if screen is smaller than typical desktop
        if (width < 1440 || height < 900) {
          this.capabilities.deviceType = DeviceType.LAPTOP;
        } else {
          this.capabilities.deviceType = DeviceType.DESKTOP;
        }
      } else {
        this.capabilities.deviceType = DeviceType.OTHER;
      }
    }
  }

  /**
   * Detects OS information
   */
  private detectOSInfo(): void {
    const ua = navigator.userAgent;
    
    // Detect OS
    if (/Windows/i.test(ua)) {
      this.capabilities.os = 'Windows';
      
      // Detect Windows version
      if (/Windows NT 10.0/i.test(ua)) {
        this.capabilities.osVersion = '10';
      } else if (/Windows NT 6.3/i.test(ua)) {
        this.capabilities.osVersion = '8.1';
      } else if (/Windows NT 6.2/i.test(ua)) {
        this.capabilities.osVersion = '8';
      } else if (/Windows NT 6.1/i.test(ua)) {
        this.capabilities.osVersion = '7';
      } else {
        this.capabilities.osVersion = 'Unknown';
      }
    } else if (/Macintosh|MacIntel|MacPPC|Mac68K/i.test(ua)) {
      this.capabilities.os = 'macOS';
      
      // macOS version is harder to detect from UA
      this.capabilities.osVersion = 'Unknown';
    } else if (/Android/i.test(ua)) {
      this.capabilities.os = 'Android';
      
      // Extract Android version
      const match = ua.match(/Android (\d+(\.\d+)*)/i);
      this.capabilities.osVersion = match ? match[1] : 'Unknown';
    } else if (/iPhone|iPad|iPod/i.test(ua)) {
      this.capabilities.os = 'iOS';
      
      // Extract iOS version
      const match = ua.match(/OS (\d+(_\d+)*)/i);
      this.capabilities.osVersion = match ? match[1].replace(/_/g, '.') : 'Unknown';
    } else if (/Linux/i.test(ua)) {
      this.capabilities.os = 'Linux';
      this.capabilities.osVersion = 'Unknown';
    } else {
      this.capabilities.os = 'Unknown';
      this.capabilities.osVersion = 'Unknown';
    }
    
    // Set app version (in a real app, this would be the actual app version)
    this.capabilities.appVersion = '1.0.0';
  }

  /**
   * Detects storage capabilities
   */
  private async detectStorageCapabilities(): Promise<void> {
    try {
      if (navigator.storage && navigator.storage.estimate) {
        const estimate = await navigator.storage.estimate();
        this.capabilities.storageBytes = estimate.quota;
      }
    } catch (error) {
      console.warn('Failed to detect storage capabilities:', error);
    }
  }

  /**
   * Detects memory capabilities
   */
  private async detectMemoryCapabilities(): Promise<void> {
    try {
      // @ts-ignore - deviceMemory is not in the standard TypeScript navigator type
      if (navigator.deviceMemory) {
        // deviceMemory is in GB
        // @ts-ignore
        this.capabilities.memoryBytes = navigator.deviceMemory * 1024 * 1024 * 1024;
      }
    } catch (error) {
      console.warn('Failed to detect memory capabilities:', error);
    }
  }

  /**
   * Detects CPU capabilities
   */
  private async detectCPUCapabilities(): Promise<void> {
    try {
      // @ts-ignore - hardwareConcurrency is not in the standard TypeScript navigator type
      if (navigator.hardwareConcurrency) {
        // @ts-ignore
        this.capabilities.cpuCores = navigator.hardwareConcurrency;
      }
      
      // Detect CPU architecture
      const ua = navigator.userAgent;
      if (/x86_64|x86-64|x64|amd64/i.test(ua)) {
        this.capabilities.cpuArchitecture = 'x86_64';
      } else if (/x86|i386|i686/i.test(ua)) {
        this.capabilities.cpuArchitecture = 'x86';
      } else if (/arm64|aarch64/i.test(ua)) {
        this.capabilities.cpuArchitecture = 'arm64';
      } else if (/arm/i.test(ua)) {
        this.capabilities.cpuArchitecture = 'arm';
      } else {
        this.capabilities.cpuArchitecture = 'unknown';
      }
    } catch (error) {
      console.warn('Failed to detect CPU capabilities:', error);
    }
  }

  /**
   * Detects network capabilities
   */
  private async detectNetworkCapabilities(): Promise<void> {
    try {
      // Detect network type
      // @ts-ignore - connection is not in the standard TypeScript navigator type
      const connection = navigator.connection || navigator.mozConnection || navigator.webkitConnection;
      
      if (connection) {
        // Detect network type
        switch (connection.type) {
          case 'wifi':
            this.capabilities.networkType = NetworkType.WIFI;
            break;
          case 'cellular':
            this.capabilities.networkType = NetworkType.CELLULAR;
            break;
          case 'ethernet':
            this.capabilities.networkType = NetworkType.ETHERNET;
            break;
          case 'none':
            this.capabilities.networkType = NetworkType.OFFLINE;
            break;
          default:
            this.capabilities.networkType = NetworkType.UNKNOWN;
        }
        
        // Detect network quality based on effective type
        switch (connection.effectiveType) {
          case '4g':
            this.capabilities.networkQuality = NetworkQuality.EXCELLENT;
            break;
          case '3g':
            this.capabilities.networkQuality = NetworkQuality.GOOD;
            break;
          case '2g':
            this.capabilities.networkQuality = NetworkQuality.FAIR;
            break;
          case 'slow-2g':
            this.capabilities.networkQuality = NetworkQuality.POOR;
            break;
          default:
            this.capabilities.networkQuality = NetworkQuality.UNKNOWN;
        }
        
        // Get bandwidth if available
        if (connection.downlink) {
          // downlink is in Mbps, convert to bps
          this.capabilities.networkBandwidth = connection.downlink * 1024 * 1024;
        }
        
        // Get latency if available (rtt is in ms)
        if (connection.rtt) {
          this.capabilities.networkLatency = connection.rtt;
        }
      } else {
        // Fallback: check if online
        if (navigator.onLine) {
          this.capabilities.networkType = NetworkType.UNKNOWN;
          this.capabilities.networkQuality = NetworkQuality.UNKNOWN;
        } else {
          this.capabilities.networkType = NetworkType.OFFLINE;
          this.capabilities.networkQuality = NetworkQuality.POOR;
        }
      }
      
      // Add event listeners for network changes
      window.addEventListener('online', () => {
        this.capabilities.networkType = NetworkType.UNKNOWN;
        this.detectNetworkCapabilities();
      });
      
      window.addEventListener('offline', () => {
        this.capabilities.networkType = NetworkType.OFFLINE;
        this.capabilities.networkQuality = NetworkQuality.POOR;
        this.options.onCapabilitiesChanged(this.capabilities);
      });
      
      // Add connection change listener if available
      if (connection && connection.addEventListener) {
        connection.addEventListener('change', () => {
          this.detectNetworkCapabilities();
        });
      }
    } catch (error) {
      console.warn('Failed to detect network capabilities:', error);
    }
  }

  /**
   * Detects battery status
   */
  private async detectBatteryStatus(): Promise<void> {
    try {
      // @ts-ignore - getBattery is not in the standard TypeScript navigator type
      if (navigator.getBattery) {
        // @ts-ignore
        const battery = await navigator.getBattery();
        
        this.capabilities.batteryLevel = battery.level * 100;
        this.capabilities.isCharging = battery.charging;
        
        // Add event listeners for battery changes
        battery.addEventListener('levelchange', () => {
          this.capabilities.batteryLevel = battery.level * 100;
          this.options.onCapabilitiesChanged(this.capabilities);
        });
        
        battery.addEventListener('chargingchange', () => {
          this.capabilities.isCharging = battery.charging;
          this.options.onCapabilitiesChanged(this.capabilities);
        });
      }
    } catch (error) {
      console.warn('Failed to detect battery status:', error);
    }
  }

  /**
   * Detects screen capabilities
   */
  private detectScreenCapabilities(): void {
    try {
      this.capabilities.screenWidth = window.screen.width;
      this.capabilities.screenHeight = window.screen.height;
      this.capabilities.pixelRatio = window.devicePixelRatio || 1;
      
      // Detect touch support
      this.capabilities.touchSupport = 'ontouchstart' in window || 
        navigator.maxTouchPoints > 0 ||
        // @ts-ignore - msMaxTouchPoints is not in the standard TypeScript navigator type
        navigator.msMaxTouchPoints > 0;
      
      // Add resize listener
      window.addEventListener('resize', () => {
        this.capabilities.screenWidth = window.screen.width;
        this.capabilities.screenHeight = window.screen.height;
        this.options.onCapabilitiesChanged(this.capabilities);
      });
    } catch (error) {
      console.warn('Failed to detect screen capabilities:', error);
    }
  }

  /**
   * Detects feature support
   */
  private detectFeatureSupport(): void {
    try {
      // Detect WebGL support
      const canvas = document.createElement('canvas');
      this.capabilities.supportsWebGL = !!(
        window.WebGLRenderingContext &&
        (canvas.getContext('webgl') || canvas.getContext('experimental-webgl'))
      );
      
      // Detect WebAssembly support
      this.capabilities.supportsWasm = typeof WebAssembly === 'object' &&
        typeof WebAssembly.compile === 'function';
      
      // Detect IndexedDB support
      this.capabilities.supportsIndexedDB = !!window.indexedDB;
      
      // Detect Service Worker support
      this.capabilities.supportsServiceWorker = 'serviceWorker' in navigator;
      
      // Detect Push API support
      this.capabilities.supportsPushNotifications = 'PushManager' in window;
      
      // Detect Background Sync support
      this.capabilities.supportsBackgroundSync = 'serviceWorker' in navigator &&
        'SyncManager' in window;
      
      // Detect hardware acceleration
      this.capabilities.supportsHardwareAcceleration = this.capabilities.supportsWebGL ||
        // @ts-ignore - chrome is not in the standard TypeScript window type
        (window.chrome && window.chrome.app && window.chrome.app.isInstalled);
      
      // Set sync protocol support based on capabilities
      if (this.capabilities.supportsWasm && this.capabilities.supportsIndexedDB) {
        this.capabilities.supportsCRDT = true;
        this.capabilities.supportsOT = true;
        this.capabilities.supportsDeltaSync = true;
        this.capabilities.maxSupportedProtocol = SyncProtocol.CRDT_SYNC;
      } else if (this.capabilities.supportsIndexedDB) {
        this.capabilities.supportsCRDT = false;
        this.capabilities.supportsOT = true;
        this.capabilities.supportsDeltaSync = true;
        this.capabilities.maxSupportedProtocol = SyncProtocol.OPERATIONAL_TRANSFORM;
      } else {
        this.capabilities.supportsCRDT = false;
        this.capabilities.supportsOT = false;
        this.capabilities.supportsDeltaSync = false;
        this.capabilities.maxSupportedProtocol = SyncProtocol.INCREMENTAL_SYNC;
      }
    } catch (error) {
      console.warn('Failed to detect feature support:', error);
    }
  }
}

/**
 * Device capability adapter
 */
export class DeviceCapabilityAdapter {
  private detector: DeviceCapabilityDetector;
  private currentStrategy: AdaptationStrategy;
  private listeners: Array<(strategy: AdaptationStrategy) => void> = [];

  /**
   * Creates a new device capability adapter
   * @param detector - The capability detector to use
   */
  constructor(detector: DeviceCapabilityDetector) {
    this.detector = detector;
    
    // Initialize with default strategy for medium tier
    this.currentStrategy = DEFAULT_ADAPTATION_STRATEGIES[CapabilityTier.MEDIUM];
    
    // Update strategy when capabilities change
    detector.detectCapabilities().then(() => {
      this.updateStrategy();
    });
    
    // Listen for capability changes
    const options = detector.getCapabilities();
    if (options.onCapabilitiesChanged) {
      const originalCallback = options.onCapabilitiesChanged;
      options.onCapabilitiesChanged = (capabilities) => {
        originalCallback(capabilities);
        this.updateStrategy();
      };
    }
  }

  /**
   * Updates the adaptation strategy based on current capabilities
   */
  private updateStrategy(): void {
    const tier = this.detector.getCapabilityTier();
    const newStrategy = DEFAULT_ADAPTATION_STRATEGIES[tier];
    
    // Check if strategy has changed
    if (this.currentStrategy.tier !== newStrategy.tier) {
      this.currentStrategy = { ...newStrategy };
      
      // Notify listeners
      this.notifyListeners();
    }
  }

  /**
   * Notifies listeners of strategy changes
   */
  private notifyListeners(): void {
    for (const listener of this.listeners) {
      listener(this.currentStrategy);
    }
  }

  /**
   * Gets the current adaptation strategy
   * @returns The current strategy
   */
  getStrategy(): AdaptationStrategy {
    return { ...this.currentStrategy };
  }

  /**
   * Adds a listener for strategy changes
   * @param listener - The listener function
   */
  addListener(listener: (strategy: AdaptationStrategy) => void): void {
    this.listeners.push(listener);
  }

  /**
   * Removes a listener
   * @param listener - The listener function to remove
   */
  removeListener(listener: (strategy: AdaptationStrategy) => void): void {
    const index = this.listeners.indexOf(listener);
    if (index !== -1) {
      this.listeners.splice(index, 1);
    }
  }

  /**
   * Checks if a feature is enabled in the current strategy
   * @param feature - The feature to check
   * @returns Whether the feature is enabled
   */
  isFeatureEnabled(feature: 'backgroundProcessing' | 'prefetching' | 'highResAssets' | 'advancedVisuals' | 'realTimeUpdates'): boolean {
    switch (feature) {
      case 'backgroundProcessing':
        return this.currentStrategy.enableBackgroundProcessing;
      case 'prefetching':
        return this.currentStrategy.enablePrefetching;
      case 'highResAssets':
        return this.currentStrategy.enableHighResAssets;
      case 'advancedVisuals':
        return this.currentStrategy.enableAdvancedVisuals;
      case 'realTimeUpdates':
        return this.currentStrategy.enableRealTimeUpdates;
      default:
        return false;
    }
  }

  /**
   * Gets a parameter value from the current strategy
   * @param param - The parameter name
   * @returns The parameter value
   */
  getParameter<T>(param: keyof AdaptationStrategy): T {
    return this.currentStrategy[param] as unknown as T;
  }

  /**
   * Forces a specific capability tier
   * @param tier - The tier to force
   */
  forceTier(tier: CapabilityTier): void {
    this.currentStrategy = { ...DEFAULT_ADAPTATION_STRATEGIES[tier] };
    this.notifyListeners();
  }
}

// Create singleton instances
let detector: DeviceCapabilityDetector | null = null;
let adapter: DeviceCapabilityAdapter | null = null;

/**
 * Initializes the device capability system
 * @param options - Detection options
 * @returns The capability adapter
 */
export async function initializeDeviceCapabilities(
  options: CapabilityDetectionOptions = {}
): Promise<DeviceCapabilityAdapter> {
  if (!detector) {
    detector = new DeviceCapabilityDetector(options);
    await detector.detectCapabilities();
    
    if (options.enablePeriodicUpdates !== false) {
      detector.startPeriodicUpdates();
    }
  }
  
  if (!adapter) {
    adapter = new DeviceCapabilityAdapter(detector);
  }
  
  return adapter;
}

/**
 * Gets the device capability detector
 * @returns The detector, or null if not initialized
 */
export function getDeviceCapabilityDetector(): DeviceCapabilityDetector | null {
  return detector;
}

/**
 * Gets the device capability adapter
 * @returns The adapter, or null if not initialized
 */
export function getDeviceCapabilityAdapter(): DeviceCapabilityAdapter | null {
  return adapter;
}

/**
 * Gets the current device capabilities
 * @returns The current capabilities, or null if not initialized
 */
export function getCurrentDeviceCapabilities(): DeviceCapabilities | null {
  return detector ? detector.getCapabilities() : null;
}

/**
 * Gets the current adaptation strategy
 * @returns The current strategy, or null if not initialized
 */
export function getCurrentAdaptationStrategy(): AdaptationStrategy | null {
  return adapter ? adapter.getStrategy() : null;
}

/**
 * Checks if a feature is enabled based on current device capabilities
 * @param feature - The feature to check
 * @returns Whether the feature is enabled, or false if not initialized
 */
export function isFeatureEnabled(
  feature: 'backgroundProcessing' | 'prefetching' | 'highResAssets' | 'advancedVisuals' | 'realTimeUpdates'
): boolean {
  return adapter ? adapter.isFeatureEnabled(feature) : false;
}