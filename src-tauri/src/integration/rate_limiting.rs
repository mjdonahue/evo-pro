//! Rate limiting and quota management for external services
//!
//! This module provides mechanisms to manage rate limits and quotas
//! when integrating with external systems and services.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use tokio::time::sleep;
use crate::error::{Error, ErrorKind, Result};

/// Rate limit configuration for an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed in the time window
    pub max_requests: u32,

    /// Time window in seconds
    pub window_seconds: u32,

    /// Whether to wait for rate limit reset or fail immediately
    pub wait_for_reset: bool,

    /// Maximum time to wait for rate limit reset (in seconds)
    pub max_wait_seconds: Option<u32>,

    /// Whether to distribute requests evenly across the window
    pub distribute_evenly: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,
            window_seconds: 60,
            wait_for_reset: true,
            max_wait_seconds: Some(300), // 5 minutes
            distribute_evenly: false,
        }
    }
}

/// Rate limit state for tracking request history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitState {
    /// Request timestamps within the current window
    pub request_timestamps: VecDeque<DateTime<Utc>>,

    /// When the rate limit was last reset
    pub last_reset: DateTime<Utc>,

    /// Number of requests made in the current window
    pub request_count: u32,

    /// Whether the rate limit is currently exceeded
    pub is_exceeded: bool,

    /// When the rate limit will reset
    pub reset_at: DateTime<Utc>,
}

impl RateLimitState {
    /// Create a new rate limit state
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            request_timestamps: VecDeque::new(),
            last_reset: now,
            request_count: 0,
            is_exceeded: false,
            reset_at: now,
        }
    }

    /// Update the state with a new request
    pub fn record_request(&mut self, config: &RateLimitConfig) {
        let now = Utc::now();

        // Remove timestamps outside the current window
        let window_duration = Duration::seconds(config.window_seconds as i64);
        let window_start = now - window_duration;

        while let Some(timestamp) = self.request_timestamps.front() {
            if *timestamp < window_start {
                self.request_timestamps.pop_front();
            } else {
                break;
            }
        }

        // Add the new request
        self.request_timestamps.push_back(now);
        self.request_count = self.request_timestamps.len() as u32;

        // Check if rate limit is exceeded
        self.is_exceeded = self.request_count >= config.max_requests;

        // Calculate reset time
        if let Some(oldest) = self.request_timestamps.front() {
            self.reset_at = *oldest + window_duration;
        } else {
            self.reset_at = now + window_duration;
        }
    }

    /// Get the time until the rate limit resets
    pub fn time_until_reset(&self) -> Duration {
        let now = Utc::now();
        if now >= self.reset_at {
            Duration::zero()
        } else {
            self.reset_at - now
        }
    }

    /// Get the number of requests remaining in the current window
    pub fn requests_remaining(&self, config: &RateLimitConfig) -> u32 {
        if self.request_count >= config.max_requests {
            0
        } else {
            config.max_requests - self.request_count
        }
    }
}

/// Trait for rate limiting strategies
#[async_trait]
pub trait RateLimitStrategy: Send + Sync {
    /// Check if a request can be made
    async fn check_rate_limit(&self, service_id: &str) -> Result<bool>;

    /// Record that a request was made
    async fn record_request(&self, service_id: &str) -> Result<()>;

    /// Wait until a request can be made
    async fn wait_for_rate_limit(&self, service_id: &str) -> Result<()>;

    /// Get the current rate limit state
    async fn get_rate_limit_state(&self, service_id: &str) -> Result<RateLimitState>;

    /// Get the rate limit configuration
    fn get_rate_limit_config(&self, service_id: &str) -> Result<RateLimitConfig>;

    /// Set the rate limit configuration
    fn set_rate_limit_config(&self, service_id: &str, config: RateLimitConfig) -> Result<()>;
}

/// Fixed window rate limiting strategy
pub struct FixedWindowRateLimiter {
    /// Rate limit configurations by service ID
    configs: RwLock<HashMap<String, RateLimitConfig>>,

    /// Rate limit states by service ID
    states: RwLock<HashMap<String, RateLimitState>>,
}

impl FixedWindowRateLimiter {
    /// Create a new fixed window rate limiter
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize a service with a rate limit configuration
    pub fn init_service(&self, service_id: &str, config: RateLimitConfig) -> Result<()> {
        let mut configs = self.configs.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on rate limit configs"
            )
        })?;

        let mut states = self.states.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on rate limit states"
            )
        })?;

        configs.insert(service_id.to_string(), config);
        states.insert(service_id.to_string(), RateLimitState::new());

        Ok(())
    }
}

#[async_trait]
impl RateLimitStrategy for FixedWindowRateLimiter {
    async fn check_rate_limit(&self, service_id: &str) -> Result<bool> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit configs"
            )
        })?;

        let states = self.states.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit states"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit configuration not found for service: {}", service_id)
            )
        })?;

        let state = states.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit state not found for service: {}", service_id)
            )
        })?;

        Ok(!state.is_exceeded)
    }

    async fn record_request(&self, service_id: &str) -> Result<()> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit configs"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit configuration not found for service: {}", service_id)
            )
        })?;

        let mut states = self.states.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on rate limit states"
            )
        })?;

        let state = states.entry(service_id.to_string()).or_insert_with(RateLimitState::new);
        state.record_request(config);

        Ok(())
    }

    async fn wait_for_rate_limit(&self, service_id: &str) -> Result<()> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit configs"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit configuration not found for service: {}", service_id)
            )
        })?;

        if !config.wait_for_reset {
            return Err(Error::new(
                ErrorKind::RateLimit,
                &format!("Rate limit exceeded for service: {}", service_id)
            ));
        }

        let states = self.states.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit states"
            )
        })?;

        let state = states.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit state not found for service: {}", service_id)
            )
        })?;

        if !state.is_exceeded {
            return Ok(());
        }

        let wait_duration = state.time_until_reset();
        let wait_seconds = wait_duration.num_seconds() as u32;

        if let Some(max_wait) = config.max_wait_seconds {
            if wait_seconds > max_wait {
                return Err(Error::new(
                    ErrorKind::RateLimit,
                    &format!(
                        "Rate limit exceeded for service: {}. Wait time ({} seconds) exceeds maximum ({} seconds)",
                        service_id, wait_seconds, max_wait
                    )
                ));
            }
        }

        // Convert chrono::Duration to tokio::time::Duration
        let wait_millis = wait_duration.num_milliseconds() as u64;
        sleep(tokio::time::Duration::from_millis(wait_millis)).await;

        Ok(())
    }

    async fn get_rate_limit_state(&self, service_id: &str) -> Result<RateLimitState> {
        let states = self.states.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit states"
            )
        })?;

        let state = states.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit state not found for service: {}", service_id)
            )
        })?;

        Ok(state.clone())
    }

    fn get_rate_limit_config(&self, service_id: &str) -> Result<RateLimitConfig> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on rate limit configs"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Rate limit configuration not found for service: {}", service_id)
            )
        })?;

        Ok(config.clone())
    }

    fn set_rate_limit_config(&self, service_id: &str, config: RateLimitConfig) -> Result<()> {
        let mut configs = self.configs.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on rate limit configs"
            )
        })?;

        configs.insert(service_id.to_string(), config);

        Ok(())
    }
}

/// Sliding window rate limiting strategy
pub struct SlidingWindowRateLimiter {
    /// Inner fixed window rate limiter
    inner: FixedWindowRateLimiter,
}

impl SlidingWindowRateLimiter {
    /// Create a new sliding window rate limiter
    pub fn new() -> Self {
        Self {
            inner: FixedWindowRateLimiter::new(),
        }
    }

    /// Initialize a service with a rate limit configuration
    pub fn init_service(&self, service_id: &str, config: RateLimitConfig) -> Result<()> {
        self.inner.init_service(service_id, config)
    }
}

#[async_trait]
impl RateLimitStrategy for SlidingWindowRateLimiter {
    async fn check_rate_limit(&self, service_id: &str) -> Result<bool> {
        self.inner.check_rate_limit(service_id).await
    }

    async fn record_request(&self, service_id: &str) -> Result<()> {
        self.inner.record_request(service_id).await
    }

    async fn wait_for_rate_limit(&self, service_id: &str) -> Result<()> {
        self.inner.wait_for_rate_limit(service_id).await
    }

    async fn get_rate_limit_state(&self, service_id: &str) -> Result<RateLimitState> {
        self.inner.get_rate_limit_state(service_id).await
    }

    fn get_rate_limit_config(&self, service_id: &str) -> Result<RateLimitConfig> {
        self.inner.get_rate_limit_config(service_id)
    }

    fn set_rate_limit_config(&self, service_id: &str, config: RateLimitConfig) -> Result<()> {
        self.inner.set_rate_limit_config(service_id, config)
    }
}

/// Quota configuration for an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    /// Maximum number of requests allowed in the quota period
    pub max_requests: u32,

    /// Quota period in seconds
    pub period_seconds: u32,

    /// Whether to wait for quota reset or fail immediately
    pub wait_for_reset: bool,

    /// Maximum time to wait for quota reset (in seconds)
    pub max_wait_seconds: Option<u32>,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            max_requests: 10000,
            period_seconds: 86400, // 24 hours
            wait_for_reset: false,
            max_wait_seconds: None,
        }
    }
}

/// Quota state for tracking usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaState {
    /// Number of requests made in the current period
    pub request_count: u32,

    /// When the quota period started
    pub period_start: DateTime<Utc>,

    /// When the quota will reset
    pub reset_at: DateTime<Utc>,

    /// Whether the quota is currently exceeded
    pub is_exceeded: bool,
}

impl QuotaState {
    /// Create a new quota state
    pub fn new(config: &QuotaConfig) -> Self {
        let now = Utc::now();
        let period_duration = Duration::seconds(config.period_seconds as i64);
        let reset_at = now + period_duration;

        Self {
            request_count: 0,
            period_start: now,
            reset_at,
            is_exceeded: false,
        }
    }

    /// Update the state with a new request
    pub fn record_request(&mut self, config: &QuotaConfig) {
        let now = Utc::now();

        // Check if we need to reset the period
        if now >= self.reset_at {
            let period_duration = Duration::seconds(config.period_seconds as i64);
            self.period_start = now;
            self.reset_at = now + period_duration;
            self.request_count = 0;
            self.is_exceeded = false;
        }

        // Increment the request count
        self.request_count += 1;

        // Check if quota is exceeded
        self.is_exceeded = self.request_count > config.max_requests;
    }

    /// Get the time until the quota resets
    pub fn time_until_reset(&self) -> Duration {
        let now = Utc::now();
        if now >= self.reset_at {
            Duration::zero()
        } else {
            self.reset_at - now
        }
    }

    /// Get the number of requests remaining in the current period
    pub fn requests_remaining(&self, config: &QuotaConfig) -> u32 {
        if self.request_count >= config.max_requests {
            0
        } else {
            config.max_requests - self.request_count
        }
    }
}

/// Trait for quota management strategies
#[async_trait]
pub trait QuotaStrategy: Send + Sync {
    /// Check if a request can be made within the quota
    async fn check_quota(&self, service_id: &str) -> Result<bool>;

    /// Record that a request was made
    async fn record_request(&self, service_id: &str) -> Result<()>;

    /// Wait until a request can be made within the quota
    async fn wait_for_quota(&self, service_id: &str) -> Result<()>;

    /// Get the current quota state
    async fn get_quota_state(&self, service_id: &str) -> Result<QuotaState>;

    /// Get the quota configuration
    fn get_quota_config(&self, service_id: &str) -> Result<QuotaConfig>;

    /// Set the quota configuration
    fn set_quota_config(&self, service_id: &str, config: QuotaConfig) -> Result<()>;
}

/// Simple quota management strategy
pub struct SimpleQuotaManager {
    /// Quota configurations by service ID
    configs: RwLock<HashMap<String, QuotaConfig>>,

    /// Quota states by service ID
    states: RwLock<HashMap<String, QuotaState>>,
}

impl SimpleQuotaManager {
    /// Create a new simple quota manager
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize a service with a quota configuration
    pub fn init_service(&self, service_id: &str, config: QuotaConfig) -> Result<()> {
        let mut configs = self.configs.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on quota configs"
            )
        })?;

        let mut states = self.states.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on quota states"
            )
        })?;

        configs.insert(service_id.to_string(), config.clone());
        states.insert(service_id.to_string(), QuotaState::new(&config));

        Ok(())
    }
}

#[async_trait]
impl QuotaStrategy for SimpleQuotaManager {
    async fn check_quota(&self, service_id: &str) -> Result<bool> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota configs"
            )
        })?;

        let states = self.states.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota states"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota configuration not found for service: {}", service_id)
            )
        })?;

        let state = states.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota state not found for service: {}", service_id)
            )
        })?;

        Ok(!state.is_exceeded)
    }

    async fn record_request(&self, service_id: &str) -> Result<()> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota configs"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota configuration not found for service: {}", service_id)
            )
        })?;

        let mut states = self.states.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on quota states"
            )
        })?;

        let state = states.entry(service_id.to_string()).or_insert_with(|| QuotaState::new(config));
        state.record_request(config);

        Ok(())
    }

    async fn wait_for_quota(&self, service_id: &str) -> Result<()> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota configs"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota configuration not found for service: {}", service_id)
            )
        })?;

        if !config.wait_for_reset {
            return Err(Error::new(
                ErrorKind::QuotaExceeded,
                &format!("Quota exceeded for service: {}", service_id)
            ));
        }

        let states = self.states.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota states"
            )
        })?;

        let state = states.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota state not found for service: {}", service_id)
            )
        })?;

        if !state.is_exceeded {
            return Ok(());
        }

        let wait_duration = state.time_until_reset();
        let wait_seconds = wait_duration.num_seconds() as u32;

        if let Some(max_wait) = config.max_wait_seconds {
            if wait_seconds > max_wait {
                return Err(Error::new(
                    ErrorKind::QuotaExceeded,
                    &format!(
                        "Quota exceeded for service: {}. Wait time ({} seconds) exceeds maximum ({} seconds)",
                        service_id, wait_seconds, max_wait
                    )
                ));
            }
        }

        // Convert chrono::Duration to tokio::time::Duration
        let wait_millis = wait_duration.num_milliseconds() as u64;
        sleep(tokio::time::Duration::from_millis(wait_millis)).await;

        Ok(())
    }

    async fn get_quota_state(&self, service_id: &str) -> Result<QuotaState> {
        let states = self.states.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota states"
            )
        })?;

        let state = states.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota state not found for service: {}", service_id)
            )
        })?;

        Ok(state.clone())
    }

    fn get_quota_config(&self, service_id: &str) -> Result<QuotaConfig> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on quota configs"
            )
        })?;

        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Quota configuration not found for service: {}", service_id)
            )
        })?;

        Ok(config.clone())
    }

    fn set_quota_config(&self, service_id: &str, config: QuotaConfig) -> Result<()> {
        let mut configs = self.configs.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on quota configs"
            )
        })?;

        configs.insert(service_id.to_string(), config);

        Ok(())
    }
}

/// Combined rate limiting and quota management service
pub struct RateLimitService {
    /// Rate limiting strategy
    rate_limiter: Arc<dyn RateLimitStrategy>,

    /// Quota management strategy
    quota_manager: Arc<dyn QuotaStrategy>,
}

/// Rate limited service wrapper
pub struct RateLimitedService<T: crate::integration::interfaces::ExternalService> {
    /// The wrapped service
    service: T,

    /// Rate limit service
    rate_limit_service: Arc<RateLimitService>,

    /// Service ID for rate limiting
    service_id: String,
}

impl<T: crate::integration::interfaces::ExternalService> RateLimitedService<T> {
    /// Create a new rate limited service
    pub fn new(
        service: T,
        rate_limit_service: Arc<RateLimitService>,
        service_id: String,
    ) -> Self {
        Self {
            service,
            rate_limit_service,
            service_id,
        }
    }

    /// Get the wrapped service
    pub fn service(&self) -> &T {
        &self.service
    }

    /// Get the current rate limit state
    pub async fn get_rate_limit_state(&self) -> Result<RateLimitState> {
        self.rate_limit_service.get_rate_limit_state(&self.service_id).await
    }

    /// Get the current quota state
    pub async fn get_quota_state(&self) -> Result<QuotaState> {
        self.rate_limit_service.get_quota_state(&self.service_id).await
    }

    /// Execute a function with rate limiting and quota management
    pub async fn execute<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send,
    {
        self.rate_limit_service.execute(&self.service_id, f).await
    }

    /// Check if a request can be made (both rate limit and quota)
    pub async fn can_request(&self) -> Result<bool> {
        self.rate_limit_service.check_request(&self.service_id).await
    }

    /// Wait until a request can be made (both rate limit and quota)
    pub async fn wait_for_request(&self) -> Result<()> {
        self.rate_limit_service.wait_for_request(&self.service_id).await
    }
}

#[async_trait]
impl<T: crate::integration::interfaces::ExternalService + Send + Sync> crate::integration::interfaces::ExternalService for RateLimitedService<T> {
    fn id(&self) -> &str {
        self.service.id()
    }

    fn name(&self) -> &str {
        self.service.name()
    }

    async fn capabilities(&self) -> Result<crate::integration::interfaces::ServiceCapabilities> {
        // Execute with rate limiting
        self.execute(|| self.service.capabilities()).await
    }

    async fn status(&self) -> Result<crate::integration::interfaces::ServiceStatus> {
        // For status checks, we don't want to apply rate limiting
        // as this could prevent us from checking the service status
        // when we need it most (during high load)
        self.service.status().await
    }

    async fn initialize(&self) -> Result<()> {
        // Execute with rate limiting
        self.execute(|| self.service.initialize()).await
    }

    async fn terminate(&self) -> Result<()> {
        // For termination, we don't want to apply rate limiting
        // as this could prevent proper shutdown
        self.service.terminate().await
    }
}

impl RateLimitService {
    /// Create a new rate limit service
    pub fn new(
        rate_limiter: Arc<dyn RateLimitStrategy>,
        quota_manager: Arc<dyn QuotaStrategy>,
    ) -> Self {
        Self {
            rate_limiter,
            quota_manager,
        }
    }

    /// Initialize a service with rate limit and quota configurations
    pub fn init_service(
        &self,
        service_id: &str,
        rate_limit_config: RateLimitConfig,
        quota_config: QuotaConfig,
    ) -> Result<()> {
        if let Some(limiter) = Arc::get_mut(&mut self.rate_limiter.clone()) {
            if let Some(fixed) = limiter.downcast_mut::<FixedWindowRateLimiter>() {
                fixed.init_service(service_id, rate_limit_config)?;
            } else if let Some(sliding) = limiter.downcast_mut::<SlidingWindowRateLimiter>() {
                sliding.init_service(service_id, rate_limit_config)?;
            }
        }

        if let Some(quota) = Arc::get_mut(&mut self.quota_manager.clone()) {
            if let Some(simple) = quota.downcast_mut::<SimpleQuotaManager>() {
                simple.init_service(service_id, quota_config)?;
            }
        }

        Ok(())
    }

    /// Check if a request can be made (both rate limit and quota)
    pub async fn check_request(&self, service_id: &str) -> Result<bool> {
        let rate_limit_ok = self.rate_limiter.check_rate_limit(service_id).await?;
        let quota_ok = self.quota_manager.check_quota(service_id).await?;

        Ok(rate_limit_ok && quota_ok)
    }

    /// Record that a request was made
    pub async fn record_request(&self, service_id: &str) -> Result<()> {
        self.rate_limiter.record_request(service_id).await?;
        self.quota_manager.record_request(service_id).await?;

        Ok(())
    }

    /// Wait until a request can be made (both rate limit and quota)
    pub async fn wait_for_request(&self, service_id: &str) -> Result<()> {
        self.rate_limiter.wait_for_rate_limit(service_id).await?;
        self.quota_manager.wait_for_quota(service_id).await?;

        Ok(())
    }

    /// Execute a function with rate limiting and quota management
    pub async fn execute<F, T>(&self, service_id: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send,
    {
        // Check if we can make the request
        if !self.check_request(service_id).await? {
            // Wait for rate limit and quota to allow the request
            self.wait_for_request(service_id).await?;
        }

        // Record the request
        self.record_request(service_id).await?;

        // Execute the function
        let result = f();

        // Return the result
        result
    }

    /// Get the current rate limit state
    pub async fn get_rate_limit_state(&self, service_id: &str) -> Result<RateLimitState> {
        self.rate_limiter.get_rate_limit_state(service_id).await
    }

    /// Get the current quota state
    pub async fn get_quota_state(&self, service_id: &str) -> Result<QuotaState> {
        self.quota_manager.get_quota_state(service_id).await
    }

    /// Get the rate limit configuration
    pub fn get_rate_limit_config(&self, service_id: &str) -> Result<RateLimitConfig> {
        self.rate_limiter.get_rate_limit_config(service_id)
    }

    /// Get the quota configuration
    pub fn get_quota_config(&self, service_id: &str) -> Result<QuotaConfig> {
        self.quota_manager.get_quota_config(service_id)
    }

    /// Set the rate limit configuration
    pub fn set_rate_limit_config(&self, service_id: &str, config: RateLimitConfig) -> Result<()> {
        self.rate_limiter.set_rate_limit_config(service_id, config)
    }

    /// Set the quota configuration
    pub fn set_quota_config(&self, service_id: &str, config: QuotaConfig) -> Result<()> {
        self.quota_manager.set_quota_config(service_id, config)
    }

    /// Create a rate-limited service wrapper
    pub fn create_rate_limited_service<T: crate::integration::interfaces::ExternalService>(
        &self,
        service: T,
        service_id: &str,
    ) -> RateLimitedService<T> {
        RateLimitedService::new(
            service,
            Arc::new(self.clone()),
            service_id.to_string(),
        )
    }
}

impl Clone for RateLimitService {
    fn clone(&self) -> Self {
        // Note: This is a shallow clone that shares the same rate limiter and quota manager
        // This is intentional, as these are meant to be shared
        Self {
            rate_limiter: self.rate_limiter.clone(),
            quota_manager: self.quota_manager.clone(),
        }
    }
}
