//! Tests for the resources module

#[cfg(test)]
mod tests {
    use std::time::Duration;
    
    use crate::error::Result;
    use crate::resources::detection::{get_system_resources, ResourceDetector};
    use crate::resources::adaptation::{ResourceProfile, AdaptationStrategy, ResourceManager};
    
    #[tokio::test]
    async fn test_resource_detection() -> Result<()> {
        // Create a resource detector
        let mut detector = ResourceDetector::new();
        
        // Get system resources
        let resources = detector.get_resources();
        
        // Verify that we got valid resource information
        assert!(resources.cpu.logical_cores > 0, "Should have at least one CPU core");
        assert!(resources.memory.total > 0, "Should have some memory");
        assert!(resources.disk.total > 0, "Should have some disk space");
        
        // Print resource information for debugging
        println!("CPU cores: {}", resources.cpu.logical_cores);
        println!("CPU usage: {:.1}%", resources.cpu.usage);
        println!("Memory: {:.1} GB", resources.memory.total as f64 / 1024.0 / 1024.0 / 1024.0);
        println!("Disk: {:.1} GB", resources.disk.total as f64 / 1024.0 / 1024.0 / 1024.0);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_resource_adaptation() -> Result<()> {
        // Get the global resource manager
        let manager = ResourceManager::global();
        
        // Force a specific profile
        manager.force_profile(ResourceProfile::HighEnd)?;
        
        // Verify that the profile was set
        assert_eq!(manager.current_profile(), ResourceProfile::HighEnd);
        
        // Get the current strategy
        let strategy = manager.current_strategy();
        
        // Verify that the strategy has the expected parameters
        assert_eq!(strategy.profile, ResourceProfile::HighEnd);
        assert_eq!(strategy.batch_size, ResourceProfile::HighEnd.recommended_batch_size());
        assert_eq!(strategy.worker_threads, ResourceProfile::HighEnd.recommended_worker_threads());
        
        // Test with a different profile
        manager.force_profile(ResourceProfile::LowEnd)?;
        
        // Verify that the profile was set
        assert_eq!(manager.current_profile(), ResourceProfile::LowEnd);
        
        // Get the current strategy
        let strategy = manager.current_strategy();
        
        // Verify that the strategy has the expected parameters
        assert_eq!(strategy.profile, ResourceProfile::LowEnd);
        assert_eq!(strategy.batch_size, ResourceProfile::LowEnd.recommended_batch_size());
        assert_eq!(strategy.worker_threads, ResourceProfile::LowEnd.recommended_worker_threads());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_adaptation_listener() -> Result<()> {
        // Get the global resource manager
        let manager = ResourceManager::global();
        
        // Create a flag to track if the listener was called
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();
        
        // Add a listener
        manager.add_listener(move |strategy| {
            println!("Adaptation strategy changed to: {}", strategy.name);
            called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        
        // Force a profile change to trigger the listener
        manager.force_profile(ResourceProfile::Mobile)?;
        
        // Verify that the listener was called
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_custom_strategy() -> Result<()> {
        // Create a custom strategy
        let custom_strategy = AdaptationStrategy::new(ResourceProfile::HighEnd)
            .with_batch_size(2000)
            .with_worker_threads(16)
            .with_cache_size_mb(2048)
            .with_compression_level(0)
            .with_polling_interval(Duration::from_secs(30))
            .with_background_processing(true)
            .with_prefetching(true)
            .with_caching(true)
            .with_param("custom_param", "custom_value");
        
        // Get the global resource manager
        let manager = ResourceManager::global();
        
        // Register the custom strategy
        manager.register_strategy(ResourceProfile::HighEnd, custom_strategy.clone());
        
        // Force the profile to use our custom strategy
        manager.force_profile(ResourceProfile::HighEnd)?;
        
        // Get the current strategy
        let strategy = manager.current_strategy();
        
        // Verify that our custom strategy was applied
        assert_eq!(strategy.batch_size, 2000);
        assert_eq!(strategy.worker_threads, 16);
        assert_eq!(strategy.cache_size_mb, 2048);
        assert_eq!(strategy.compression_level, 0);
        assert_eq!(strategy.polling_interval, Duration::from_secs(30));
        assert!(strategy.enable_background_processing);
        assert!(strategy.enable_prefetching);
        assert!(strategy.enable_caching);
        assert_eq!(strategy.custom_params.get("custom_param"), Some(&"custom_value".to_string()));
        
        Ok(())
    }
}