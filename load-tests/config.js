/**
 * k6 Load Testing Configuration
 * 
 * This file contains the default configuration for k6 load tests.
 * It defines common settings like virtual users, duration, and thresholds.
 */

export const baseConfig = {
  // Base URL for the application
  baseUrl: 'http://localhost:1420',
  
  // Default headers for all requests
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  },
};

/**
 * Load test scenarios
 * 
 * These scenarios define different load patterns for testing.
 */
export const scenarios = {
  // Smoke test with minimal load
  smoke: {
    executor: 'constant-vus',
    vus: 1,
    duration: '30s',
    tags: { test_type: 'smoke' },
  },
  
  // Load test with moderate load
  load: {
    executor: 'ramping-vus',
    startVUs: 1,
    stages: [
      { duration: '30s', target: 5 },
      { duration: '1m', target: 10 },
      { duration: '30s', target: 0 },
    ],
    tags: { test_type: 'load' },
  },
  
  // Stress test with heavy load
  stress: {
    executor: 'ramping-vus',
    startVUs: 5,
    stages: [
      { duration: '30s', target: 10 },
      { duration: '1m', target: 20 },
      { duration: '2m', target: 30 },
      { duration: '1m', target: 0 },
    ],
    tags: { test_type: 'stress' },
  },
  
  // Spike test with sudden surge in traffic
  spike: {
    executor: 'ramping-vus',
    startVUs: 1,
    stages: [
      { duration: '10s', target: 1 },
      { duration: '10s', target: 30 },
      { duration: '30s', target: 30 },
      { duration: '10s', target: 1 },
    ],
    tags: { test_type: 'spike' },
  },
  
  // Soak test for long-duration testing
  soak: {
    executor: 'ramping-vus',
    startVUs: 1,
    stages: [
      { duration: '1m', target: 5 },
      { duration: '5m', target: 5 },
      { duration: '1m', target: 0 },
    ],
    tags: { test_type: 'soak' },
  },
};

/**
 * Performance thresholds
 * 
 * These thresholds define the acceptable performance criteria.
 */
export const thresholds = {
  // Response time thresholds
  http_req_duration: [
    // 95% of requests should be below 500ms
    { threshold: 'p(95)<500', abortOnFail: false },
    // 99% of requests should be below 1s
    { threshold: 'p(99)<1000', abortOnFail: false },
  ],
  
  // Error rate thresholds
  http_req_failed: [
    // Error rate should be below 1%
    { threshold: 'rate<0.01', abortOnFail: false },
  ],
  
  // Custom metric thresholds can be added here
};

/**
 * Creates a complete k6 options object
 * 
 * @param {string} scenarioName - The name of the scenario to use
 * @param {object} customThresholds - Custom thresholds to merge with defaults
 * @returns {object} k6 options object
 */
export function createOptions(scenarioName = 'load', customThresholds = {}) {
  const scenario = scenarios[scenarioName] || scenarios.load;
  
  return {
    scenarios: {
      [scenarioName]: scenario,
    },
    thresholds: {
      ...thresholds,
      ...customThresholds,
    },
    // Output results in various formats
    summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  };
}