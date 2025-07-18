# Adaptive Performance System

## Overview

The Adaptive Performance System is a framework for detecting system resources and adapting application behavior based on available resources. It allows the application to provide an optimal experience across a wide range of devices with different capabilities.

## Components

The system consists of two main components:

1. **Resource Detection**: Monitors system resources such as CPU, memory, disk, and network.
2. **Resource Adaptation**: Adjusts application behavior based on detected resources.

## Resource Detection

The resource detection component monitors the following system resources:

- **CPU**: Number of cores, usage, frequency
- **Memory**: Total, used, available
- **Disk**: Total space, used space, I/O rates
- **Network**: Connectivity, bandwidth
- **Battery**: Status, percentage (when available)
- **Operating System**: Name, version, architecture

Resource detection is performed by the `ResourceDetector` class, which uses the `sysinfo` crate to gather system information.

## Resource Profiles

The system defines several resource profiles that represent different device capabilities:

- **HighEnd**: Systems with abundant resources (8+ cores, 16+ GB RAM)
- **MidRange**: Systems with moderate resources (4+ cores, 8+ GB RAM)
- **LowEnd**: Systems with limited resources (2+ cores, 4+ GB RAM)
- **Mobile**: Systems with very limited resources (< 2 cores, < 4 GB RAM)
- **BatteryPowered**: Systems running on battery power
- **LimitedConnectivity**: Systems with limited network connectivity

Each profile has recommended settings for various performance parameters:

- Batch size for processing operations
- Cache size
- Number of worker threads
- Polling interval
- Compression level

## Adaptation Strategies

Adaptation strategies define how the application should behave based on the current resource profile. Each strategy includes:

- Batch size for processing operations
- Cache size in megabytes
- Number of worker threads
- Polling interval for background operations
- Compression level
- Flags for enabling/disabling features like background processing, prefetching, and caching
- Custom parameters for specific adaptations

## Resource Manager

The `ResourceManager` coordinates resource detection and adaptation. It:

1. Periodically checks system resources
2. Determines the appropriate resource profile
3. Applies the corresponding adaptation strategy
4. Notifies listeners of changes

## Usage

### Getting the Current Strategy

```rust
use crate::resources::adaptation::{get_current_strategy, AdaptationStrategy};

fn example() -> Result<()> {
    let strategy = get_current_strategy()?;
    
    // Use strategy parameters to adapt behavior
    let batch_size = strategy.batch_size;
    let worker_threads = strategy.worker_threads;
    
    // Enable/disable features based on strategy
    if strategy.enable_prefetching {
        // Enable prefetching
    }
    
    Ok(())
}
```

### Adding an Adaptation Listener

```rust
use crate::resources::adaptation::{add_adaptation_listener, AdaptationStrategy};

fn setup_adaptation() {
    add_adaptation_listener(|strategy| {
        println!("Adaptation strategy changed to: {}", strategy.name);
        
        // Adapt application behavior based on the new strategy
        update_batch_size(strategy.batch_size);
        update_worker_threads(strategy.worker_threads);
        update_cache_size(strategy.cache_size_mb);
    });
}
```

### Forcing a Specific Profile

```rust
use crate::resources::adaptation::{force_profile, ResourceProfile};

fn force_low_end_profile() -> Result<()> {
    force_profile(ResourceProfile::LowEnd)?;
    Ok(())
}
```

## Implementation Details

The system is implemented in the following files:

- `src-tauri/src/resources/mod.rs`: Main module definition
- `src-tauri/src/resources/detection.rs`: Resource detection functionality
- `src-tauri/src/resources/adaptation.rs`: Resource adaptation functionality

The system is initialized in the application's startup sequence in `src-tauri/src/lib.rs`.

## Future Enhancements

Potential future enhancements include:

1. **More Granular Profiles**: Additional profiles for specific use cases
2. **Dynamic Adaptation**: Adjusting parameters in real-time based on current resource usage
3. **User Preferences**: Allowing users to override automatic adaptation
4. **Telemetry**: Collecting anonymous usage data to improve adaptation strategies
5. **Machine Learning**: Using ML to optimize adaptation strategies based on usage patterns