//! Resource detection functionality
//!
//! This module provides functionality for detecting system resources such as
//! CPU, memory, disk space, and network capabilities.

use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::thread;

use serde::{Serialize, Deserialize};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt, ProcessExt};
use tracing::{debug, error, info, warn};

use crate::error::Result;

/// System resources information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// Timestamp when the resources were detected
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// CPU information
    pub cpu: CpuInfo,
    
    /// Memory information
    pub memory: MemoryInfo,
    
    /// Disk information
    pub disk: DiskInfo,
    
    /// Network information
    pub network: NetworkInfo,
    
    /// Battery information
    pub battery: Option<BatteryInfo>,
    
    /// System load average (1, 5, 15 minutes)
    pub load_avg: Option<(f64, f64, f64)>,
    
    /// System uptime in seconds
    pub uptime: u64,
    
    /// Operating system information
    pub os_info: OsInfo,
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// Number of physical cores
    pub physical_cores: usize,
    
    /// Number of logical cores (including hyperthreading)
    pub logical_cores: usize,
    
    /// CPU brand string
    pub brand: String,
    
    /// CPU frequency in MHz
    pub frequency: u64,
    
    /// CPU usage percentage (0-100)
    pub usage: f32,
    
    /// CPU usage per core
    pub core_usage: Vec<f32>,
    
    /// CPU architecture
    pub architecture: String,
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total physical memory in bytes
    pub total: u64,
    
    /// Used physical memory in bytes
    pub used: u64,
    
    /// Free physical memory in bytes
    pub free: u64,
    
    /// Available physical memory in bytes
    pub available: u64,
    
    /// Total swap memory in bytes
    pub swap_total: u64,
    
    /// Used swap memory in bytes
    pub swap_used: u64,
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Total disk space in bytes
    pub total: u64,
    
    /// Used disk space in bytes
    pub used: u64,
    
    /// Free disk space in bytes
    pub free: u64,
    
    /// Disk usage percentage (0-100)
    pub usage: f32,
    
    /// Disk read bytes per second
    pub read_bytes_per_sec: Option<u64>,
    
    /// Disk write bytes per second
    pub write_bytes_per_sec: Option<u64>,
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
    
    /// Total received bytes
    pub total_received: u64,
    
    /// Total transmitted bytes
    pub total_transmitted: u64,
    
    /// Received bytes per second
    pub received_per_sec: Option<u64>,
    
    /// Transmitted bytes per second
    pub transmitted_per_sec: Option<u64>,
    
    /// Network connectivity status
    pub connectivity: NetworkConnectivity,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    
    /// MAC address
    pub mac_address: Option<String>,
    
    /// IP addresses
    pub ip_addresses: Vec<String>,
    
    /// Received bytes
    pub received: u64,
    
    /// Transmitted bytes
    pub transmitted: u64,
}

/// Network connectivity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkConnectivity {
    /// No network connectivity
    None,
    
    /// Limited network connectivity
    Limited,
    
    /// Full network connectivity
    Full,
    
    /// Unknown network connectivity
    Unknown,
}

/// Battery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    /// Battery percentage (0-100)
    pub percentage: f32,
    
    /// Battery state
    pub state: BatteryState,
    
    /// Remaining time in seconds
    pub remaining_time: Option<u64>,
}

/// Battery state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryState {
    /// Battery is charging
    Charging,
    
    /// Battery is discharging
    Discharging,
    
    /// Battery is fully charged
    Full,
    
    /// Battery state is unknown
    Unknown,
}

/// Operating system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    /// Operating system name
    pub name: String,
    
    /// Operating system version
    pub version: String,
    
    /// Operating system architecture
    pub architecture: String,
    
    /// Operating system hostname
    pub hostname: String,
}

/// Resource detector for gathering system information
pub struct ResourceDetector {
    /// System information
    system: System,
    
    /// Last update time
    last_update: Instant,
    
    /// Previous network received bytes
    prev_received: u64,
    
    /// Previous network transmitted bytes
    prev_transmitted: u64,
    
    /// Previous disk read bytes
    prev_read_bytes: u64,
    
    /// Previous disk write bytes
    prev_write_bytes: u64,
    
    /// Last network update time
    last_network_update: Instant,
    
    /// Last disk update time
    last_disk_update: Instant,
    
    /// Network bytes per second
    network_bytes_per_sec: (Option<u64>, Option<u64>),
    
    /// Disk bytes per second
    disk_bytes_per_sec: (Option<u64>, Option<u64>),
}

impl ResourceDetector {
    /// Create a new resource detector
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        let now = Instant::now();
        let prev_received = system.networks().iter().map(|(_, network)| network.total_received()).sum();
        let prev_transmitted = system.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
        
        // Calculate disk I/O if available
        let (prev_read_bytes, prev_write_bytes) = Self::calculate_disk_io(&system);
        
        Self {
            system,
            last_update: now,
            prev_received,
            prev_transmitted,
            prev_read_bytes,
            prev_write_bytes,
            last_network_update: now,
            last_disk_update: now,
            network_bytes_per_sec: (None, None),
            disk_bytes_per_sec: (None, None),
        }
    }
    
    /// Calculate disk I/O
    fn calculate_disk_io(system: &System) -> (u64, u64) {
        let mut total_read = 0;
        let mut total_write = 0;
        
        // This is a simplified approach; in a real implementation, you would track per-disk I/O
        for disk in system.disks() {
            // sysinfo doesn't provide direct disk I/O metrics, so this is a placeholder
            // In a real implementation, you would use platform-specific APIs
            total_read += 0;
            total_write += 0;
        }
        
        (total_read, total_write)
    }
    
    /// Refresh system information
    pub fn refresh(&mut self) {
        self.system.refresh_all();
        
        let now = Instant::now();
        
        // Update network bytes per second
        let total_received = self.system.networks().iter().map(|(_, network)| network.total_received()).sum();
        let total_transmitted = self.system.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
        
        let network_elapsed = now.duration_since(self.last_network_update).as_secs_f64();
        if network_elapsed >= 1.0 {
            let received_per_sec = ((total_received - self.prev_received) as f64 / network_elapsed) as u64;
            let transmitted_per_sec = ((total_transmitted - self.prev_transmitted) as f64 / network_elapsed) as u64;
            
            self.network_bytes_per_sec = (Some(received_per_sec), Some(transmitted_per_sec));
            self.prev_received = total_received;
            self.prev_transmitted = total_transmitted;
            self.last_network_update = now;
        }
        
        // Update disk bytes per second
        let (total_read_bytes, total_write_bytes) = Self::calculate_disk_io(&self.system);
        
        let disk_elapsed = now.duration_since(self.last_disk_update).as_secs_f64();
        if disk_elapsed >= 1.0 {
            let read_per_sec = ((total_read_bytes - self.prev_read_bytes) as f64 / disk_elapsed) as u64;
            let write_per_sec = ((total_write_bytes - self.prev_write_bytes) as f64 / disk_elapsed) as u64;
            
            self.disk_bytes_per_sec = (Some(read_per_sec), Some(write_per_sec));
            self.prev_read_bytes = total_read_bytes;
            self.prev_write_bytes = total_write_bytes;
            self.last_disk_update = now;
        }
        
        self.last_update = now;
    }
    
    /// Get system resources
    pub fn get_resources(&mut self) -> SystemResources {
        // Refresh system information if it's been more than 1 second
        if self.last_update.elapsed() > Duration::from_secs(1) {
            self.refresh();
        }
        
        // CPU information
        let cpu_info = CpuInfo {
            physical_cores: self.system.physical_core_count().unwrap_or(0),
            logical_cores: self.system.cpus().len(),
            brand: self.system.cpus().first().map(|cpu| cpu.brand().to_string()).unwrap_or_default(),
            frequency: self.system.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0),
            usage: self.system.global_cpu_info().cpu_usage(),
            core_usage: self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
            architecture: std::env::consts::ARCH.to_string(),
        };
        
        // Memory information
        let memory_info = MemoryInfo {
            total: self.system.total_memory(),
            used: self.system.used_memory(),
            free: self.system.free_memory(),
            available: self.system.available_memory(),
            swap_total: self.system.total_swap(),
            swap_used: self.system.used_swap(),
        };
        
        // Disk information
        let total_disk_space = self.system.disks().iter().map(|disk| disk.total_space()).sum();
        let used_disk_space = self.system.disks().iter().map(|disk| disk.total_space() - disk.available_space()).sum();
        let free_disk_space = self.system.disks().iter().map(|disk| disk.available_space()).sum();
        let disk_usage = if total_disk_space > 0 {
            (used_disk_space as f32 / total_disk_space as f32) * 100.0
        } else {
            0.0
        };
        
        let disk_info = DiskInfo {
            total: total_disk_space,
            used: used_disk_space,
            free: free_disk_space,
            usage: disk_usage,
            read_bytes_per_sec: self.disk_bytes_per_sec.0,
            write_bytes_per_sec: self.disk_bytes_per_sec.1,
        };
        
        // Network information
        let mut interfaces = Vec::new();
        for (name, network) in self.system.networks() {
            interfaces.push(NetworkInterface {
                name: name.to_string(),
                mac_address: None, // sysinfo doesn't provide MAC addresses
                ip_addresses: Vec::new(), // sysinfo doesn't provide IP addresses
                received: network.total_received(),
                transmitted: network.total_transmitted(),
            });
        }
        
        let total_received = self.system.networks().iter().map(|(_, network)| network.total_received()).sum();
        let total_transmitted = self.system.networks().iter().map(|(_, network)| network.total_transmitted()).sum();
        
        let network_info = NetworkInfo {
            interfaces,
            total_received,
            total_transmitted,
            received_per_sec: self.network_bytes_per_sec.0,
            transmitted_per_sec: self.network_bytes_per_sec.1,
            connectivity: NetworkConnectivity::Unknown, // Determine connectivity status
        };
        
        // Battery information (if available)
        let battery_info = None; // sysinfo doesn't provide battery information
        
        // OS information
        let os_info = OsInfo {
            name: self.system.name().unwrap_or_default(),
            version: self.system.os_version().unwrap_or_default(),
            architecture: std::env::consts::ARCH.to_string(),
            hostname: self.system.host_name().unwrap_or_default(),
        };
        
        SystemResources {
            timestamp: chrono::Utc::now(),
            cpu: cpu_info,
            memory: memory_info,
            disk: disk_info,
            network: network_info,
            battery: battery_info,
            load_avg: self.system.load_average().map(|load| (load.one, load.five, load.fifteen)),
            uptime: self.system.uptime(),
            os_info,
        }
    }
}

/// Global resource detector instance
lazy_static::lazy_static! {
    static ref GLOBAL_DETECTOR: Arc<Mutex<ResourceDetector>> = Arc::new(Mutex::new(ResourceDetector::new()));
}

/// Get the global resource detector instance
pub fn global_detector() -> Arc<Mutex<ResourceDetector>> {
    GLOBAL_DETECTOR.clone()
}

/// Get current system resources
pub fn get_system_resources() -> Result<SystemResources> {
    let mut detector = GLOBAL_DETECTOR.lock().map_err(|e| {
        error!("Failed to lock global resource detector: {}", e);
        crate::error::AppError::internal("Failed to access system resources")
    })?;
    
    Ok(detector.get_resources())
}

/// Background resource monitoring thread
pub fn start_background_monitoring(interval: Duration) -> std::thread::JoinHandle<()> {
    let detector = GLOBAL_DETECTOR.clone();
    
    thread::spawn(move || {
        info!("Starting background resource monitoring with interval {:?}", interval);
        
        loop {
            thread::sleep(interval);
            
            // Refresh the detector
            if let Ok(mut detector) = detector.lock() {
                detector.refresh();
                
                // Log resource usage
                let resources = detector.get_resources();
                debug!(
                    "System resources: CPU: {:.1}%, Memory: {:.1}%, Disk: {:.1}%",
                    resources.cpu.usage,
                    (resources.memory.used as f32 / resources.memory.total as f32) * 100.0,
                    resources.disk.usage
                );
            }
        }
    })
}