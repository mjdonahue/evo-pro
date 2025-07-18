/**
 * Feature Usage Analytics Service
 * 
 * This module provides utilities for tracking feature usage and analyzing usage patterns.
 */

import { invoke } from '@tauri-apps/api/tauri';

/**
 * Types of analytics events
 */
export enum AnalyticsEventType {
  FeatureUsage = 'feature_usage',
  Performance = 'performance',
  Error = 'error',
  UserInterface = 'user_interface',
  Session = 'session',
  Custom = 'custom',
}

/**
 * Anonymization levels for analytics data
 */
export enum AnonymizationLevel {
  None = 'none',
  Basic = 'basic',
  Advanced = 'advanced',
  Full = 'full',
}

/**
 * Analytics event data
 */
export interface AnalyticsEvent {
  id: string;
  eventType: AnalyticsEventType;
  eventName: string;
  sessionId: string;
  timestamp: string;
  properties: Record<string, any>;
  hasConsent: boolean;
  anonymizationLevel: AnonymizationLevel;
}

/**
 * User consent preferences for analytics
 */
export interface AnalyticsConsent {
  userId: string;
  featureUsage: boolean;
  performance: boolean;
  errorReporting: boolean;
  userInterface: boolean;
  updatedAt: string;
}

/**
 * Analytics report data
 */
export interface AnalyticsReport {
  totalEvents: number;
  eventCounts: Record<string, number>;
  dailyCounts: Record<string, number>;
  reportGeneratedAt: string;
  startDate?: string;
  endDate?: string;
  eventType?: AnalyticsEventType;
}

/**
 * Options for tracking feature usage
 */
export interface TrackFeatureOptions {
  /** Additional properties to track with the event */
  properties?: Record<string, any>;
  /** User ID for consent checking */
  userId?: string;
  /** Custom session ID (uses default if not provided) */
  sessionId?: string;
}

/**
 * Options for retrieving analytics events
 */
export interface GetEventsOptions {
  /** Filter by event type */
  eventType?: AnalyticsEventType;
  /** Filter by start date */
  startDate?: Date;
  /** Filter by end date */
  endDate?: Date;
  /** Limit the number of events returned */
  limit?: number;
}

/**
 * Options for generating analytics reports
 */
export interface GenerateReportOptions {
  /** Filter by event type */
  eventType?: AnalyticsEventType;
  /** Filter by start date */
  startDate?: Date;
  /** Filter by end date */
  endDate?: Date;
}

/**
 * Feature usage analytics service
 */
export class FeatureUsageAnalytics {
  private sessionId: string;
  
  /**
   * Creates a new feature usage analytics service
   */
  constructor() {
    // Generate a random session ID
    this.sessionId = `session-${Math.random().toString(36).substring(2, 15)}`;
  }
  
  /**
   * Tracks feature usage
   * 
   * @param featureName - Name of the feature being used
   * @param action - Action performed on the feature
   * @param options - Additional tracking options
   * @returns Promise that resolves when the event is tracked
   */
  async trackFeatureUsage(
    featureName: string,
    action: string,
    options: TrackFeatureOptions = {}
  ): Promise<void> {
    const properties = {
      action,
      ...options.properties,
    };
    
    try {
      await invoke('track_analytics_event', {
        eventType: AnalyticsEventType.FeatureUsage,
        eventName: featureName,
        sessionId: options.sessionId || this.sessionId,
        properties,
        userId: options.userId,
      });
    } catch (error) {
      console.error('Failed to track feature usage:', error);
    }
  }
  
  /**
   * Gets analytics events
   * 
   * @param options - Options for retrieving events
   * @returns Promise that resolves with the events
   */
  async getEvents(options: GetEventsOptions = {}): Promise<AnalyticsEvent[]> {
    try {
      const events = await invoke<AnalyticsEvent[]>('get_analytics_events', {
        eventType: options.eventType,
        startDate: options.startDate?.toISOString(),
        endDate: options.endDate?.toISOString(),
        limit: options.limit,
      });
      
      return events;
    } catch (error) {
      console.error('Failed to get analytics events:', error);
      return [];
    }
  }
  
  /**
   * Gets feature usage events
   * 
   * @param options - Options for retrieving events
   * @returns Promise that resolves with the events
   */
  async getFeatureUsageEvents(options: Omit<GetEventsOptions, 'eventType'> = {}): Promise<AnalyticsEvent[]> {
    return this.getEvents({
      ...options,
      eventType: AnalyticsEventType.FeatureUsage,
    });
  }
  
  /**
   * Generates an analytics report
   * 
   * @param options - Options for generating the report
   * @returns Promise that resolves with the report
   */
  async generateReport(options: GenerateReportOptions = {}): Promise<AnalyticsReport> {
    try {
      const report = await invoke<AnalyticsReport>('generate_analytics_report', {
        eventType: options.eventType,
        startDate: options.startDate?.toISOString(),
        endDate: options.endDate?.toISOString(),
      });
      
      return report;
    } catch (error) {
      console.error('Failed to generate analytics report:', error);
      return {
        totalEvents: 0,
        eventCounts: {},
        dailyCounts: {},
        reportGeneratedAt: new Date().toISOString(),
      };
    }
  }
  
  /**
   * Generates a feature usage report
   * 
   * @param options - Options for generating the report
   * @returns Promise that resolves with the report
   */
  async generateFeatureUsageReport(options: Omit<GenerateReportOptions, 'eventType'> = {}): Promise<AnalyticsReport> {
    return this.generateReport({
      ...options,
      eventType: AnalyticsEventType.FeatureUsage,
    });
  }
  
  /**
   * Updates user consent preferences
   * 
   * @param userId - User ID
   * @param featureUsage - Whether the user consents to feature usage analytics
   * @param performance - Whether the user consents to performance analytics
   * @param errorReporting - Whether the user consents to error reporting
   * @param userInterface - Whether the user consents to user interface analytics
   * @returns Promise that resolves when consent is updated
   */
  async updateConsent(
    userId: string,
    featureUsage: boolean,
    performance: boolean,
    errorReporting: boolean,
    userInterface: boolean
  ): Promise<void> {
    try {
      await invoke('update_analytics_consent', {
        userId,
        featureUsage,
        performance,
        errorReporting,
        userInterface,
      });
    } catch (error) {
      console.error('Failed to update analytics consent:', error);
    }
  }
  
  /**
   * Gets user consent preferences
   * 
   * @param userId - User ID
   * @returns Promise that resolves with the consent preferences
   */
  async getConsent(userId: string): Promise<AnalyticsConsent | null> {
    try {
      const consent = await invoke<AnalyticsConsent | null>('get_analytics_consent', {
        userId,
      });
      
      return consent;
    } catch (error) {
      console.error('Failed to get analytics consent:', error);
      return null;
    }
  }
}

/**
 * Global feature usage analytics instance
 */
export const featureUsageAnalytics = new FeatureUsageAnalytics();

/**
 * Hook for using feature usage analytics in React components
 * 
 * @returns Feature usage analytics utilities
 */
export function useFeatureUsageAnalytics() {
  return {
    /**
     * Tracks feature usage
     * 
     * @param featureName - Name of the feature being used
     * @param action - Action performed on the feature
     * @param options - Additional tracking options
     */
    trackFeatureUsage: (
      featureName: string,
      action: string,
      options?: TrackFeatureOptions
    ) => featureUsageAnalytics.trackFeatureUsage(featureName, action, options),
    
    /**
     * Gets feature usage events
     * 
     * @param options - Options for retrieving events
     * @returns Promise that resolves with the events
     */
    getFeatureUsageEvents: (
      options?: Omit<GetEventsOptions, 'eventType'>
    ) => featureUsageAnalytics.getFeatureUsageEvents(options),
    
    /**
     * Generates a feature usage report
     * 
     * @param options - Options for generating the report
     * @returns Promise that resolves with the report
     */
    generateFeatureUsageReport: (
      options?: Omit<GenerateReportOptions, 'eventType'>
    ) => featureUsageAnalytics.generateFeatureUsageReport(options),
    
    /**
     * Updates user consent preferences
     * 
     * @param userId - User ID
     * @param featureUsage - Whether the user consents to feature usage analytics
     * @param performance - Whether the user consents to performance analytics
     * @param errorReporting - Whether the user consents to error reporting
     * @param userInterface - Whether the user consents to user interface analytics
     * @returns Promise that resolves when consent is updated
     */
    updateConsent: (
      userId: string,
      featureUsage: boolean,
      performance: boolean,
      errorReporting: boolean,
      userInterface: boolean
    ) => featureUsageAnalytics.updateConsent(
      userId,
      featureUsage,
      performance,
      errorReporting,
      userInterface
    ),
    
    /**
     * Gets user consent preferences
     * 
     * @param userId - User ID
     * @returns Promise that resolves with the consent preferences
     */
    getConsent: (
      userId: string
    ) => featureUsageAnalytics.getConsent(userId),
  };
}