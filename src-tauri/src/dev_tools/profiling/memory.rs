//! Memory Profiling Utilities
//!
//! This module provides tools for profiling memory usage in the application.
//! It includes utilities for tracking memory allocations, detecting memory leaks,
//! and analyzing memory usage patterns.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

/// Memory snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Timestamp when the snapshot was taken
    pub timestamp: Instant,
    /// Total memory usage in bytes
    pub total_bytes: usize,
    /// Memory usage by category in bytes
    pub category_bytes: HashMap<String, usize>,
    /// Number of allocations
    pub allocation_count: usize,
    /// Allocation sizes
    pub allocation_sizes: HashMap<usize, usize>, // Size -> Count
    /// Additional context
    pub context: HashMap<String, String>,
}

impl MemorySnapshot {
    /// Create a new memory snapshot
    pub fn new() -> Self {
        Self {
            timestamp: Instant::now(),
            total_bytes: 0,
            category_bytes: HashMap::new(),
            allocation_count: 0,
            allocation_sizes: HashMap::new(),
            context: HashMap::new(),
        }
    }
    
    /// Set the total memory usage
    pub fn with_total_bytes(mut self, bytes: usize) -> Self {
        self.total_bytes = bytes;
        self
    }
    
    /// Add memory usage for a category
    pub fn with_category_bytes(mut self, category: impl Into<String>, bytes: usize) -> Self {
        self.category_bytes.insert(category.into(), bytes);
        self
    }
    
    /// Set the allocation count
    pub fn with_allocation_count(mut self, count: usize) -> Self {
        self.allocation_count = count;
        self
    }
    
    /// Add allocation size information
    pub fn with_allocation_size(mut self, size: usize, count: usize) -> Self {
        self.allocation_sizes.insert(size, count);
        self
    }
    
    /// Add context to the snapshot
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Memory allocation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    /// Allocation ID
    pub id: String,
    /// Size in bytes
    pub size: usize,
    /// Allocation category
    pub category: String,
    /// Timestamp when the allocation was made
    pub timestamp: Instant,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
    /// Whether the allocation has been freed
    pub freed: bool,
    /// Timestamp when the allocation was freed (if applicable)
    pub freed_at: Option<Instant>,
}

impl MemoryAllocation {
    /// Create a new memory allocation
    pub fn new(size: usize, category: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            size,
            category: category.into(),
            timestamp: Instant::now(),
            stack_trace: None,
            freed: false,
            freed_at: None,
        }
    }
    
    /// Set the stack trace
    pub fn with_stack_trace(mut self, stack_trace: impl Into<String>) -> Self {
        self.stack_trace = Some(stack_trace.into());
        self
    }
    
    /// Mark the allocation as freed
    pub fn mark_freed(&mut self) {
        self.freed = true;
        self.freed_at = Some(Instant::now());
    }
}

/// Memory profiler
#[derive(Debug)]
pub struct MemoryProfiler {
    /// Memory snapshots
    snapshots: Vec<MemorySnapshot>,
    /// Memory allocations
    allocations: HashMap<String, MemoryAllocation>,
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
    /// Whether to track individual allocations
    track_allocations: bool,
    /// Whether to capture stack traces for allocations
    capture_stack_traces: bool,
}

impl MemoryProfiler {
    /// Create a new memory profiler
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            allocations: HashMap::new(),
            max_snapshots: 100,
            track_allocations: false,
            capture_stack_traces: false,
        }
    }
    
    /// Set the maximum number of snapshots to keep
    pub fn with_max_snapshots(mut self, max: usize) -> Self {
        self.max_snapshots = max;
        self
    }
    
    /// Set whether to track individual allocations
    pub fn with_track_allocations(mut self, track: bool) -> Self {
        self.track_allocations = track;
        self
    }
    
    /// Set whether to capture stack traces for allocations
    pub fn with_capture_stack_traces(mut self, capture: bool) -> Self {
        self.capture_stack_traces = capture;
        self
    }
    
    /// Take a memory snapshot
    pub fn take_snapshot(&mut self) -> MemorySnapshot {
        // In a real implementation, this would gather actual memory usage data
        // For now, we'll create a simulated snapshot
        let mut snapshot = MemorySnapshot::new();
        
        // Calculate total memory usage from tracked allocations
        let total_bytes: usize = self.allocations
            .values()
            .filter(|a| !a.freed)
            .map(|a| a.size)
            .sum();
        
        snapshot = snapshot.with_total_bytes(total_bytes);
        
        // Calculate memory usage by category
        let mut category_bytes = HashMap::new();
        for allocation in self.allocations.values() {
            if allocation.freed {
                continue;
            }
            
            *category_bytes.entry(allocation.category.clone()).or_insert(0) += allocation.size;
        }
        
        for (category, bytes) in category_bytes {
            snapshot = snapshot.with_category_bytes(category, bytes);
        }
        
        // Calculate allocation counts and sizes
        let allocation_count = self.allocations.values().filter(|a| !a.freed).count();
        snapshot = snapshot.with_allocation_count(allocation_count);
        
        // Group allocations by size
        let mut allocation_sizes = HashMap::new();
        for allocation in self.allocations.values() {
            if allocation.freed {
                continue;
            }
            
            *allocation_sizes.entry(allocation.size).or_insert(0) += 1;
        }
        
        for (size, count) in allocation_sizes {
            snapshot = snapshot.with_allocation_size(size, count);
        }
        
        // Add the snapshot to our history
        self.snapshots.push(snapshot.clone());
        
        // Trim history if needed
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
        
        snapshot
    }
    
    /// Track a memory allocation
    pub fn track_allocation(&mut self, size: usize, category: impl Into<String>) -> String {
        if !self.track_allocations {
            return String::new();
        }
        
        let mut allocation = MemoryAllocation::new(size, category);
        
        if self.capture_stack_traces {
            // In a real implementation, this would capture a stack trace
            // For now, we'll just use a placeholder
            allocation = allocation.with_stack_trace("Stack trace not available in this implementation");
        }
        
        let id = allocation.id.clone();
        self.allocations.insert(id.clone(), allocation);
        
        id
    }
    
    /// Mark a memory allocation as freed
    pub fn free_allocation(&mut self, id: &str) {
        if !self.track_allocations {
            return;
        }
        
        if let Some(allocation) = self.allocations.get_mut(id) {
            allocation.mark_freed();
        }
    }
    
    /// Get all memory snapshots
    pub fn get_snapshots(&self) -> &[MemorySnapshot] {
        &self.snapshots
    }
    
    /// Get the latest memory snapshot
    pub fn get_latest_snapshot(&self) -> Option<&MemorySnapshot> {
        self.snapshots.last()
    }
    
    /// Get all memory allocations
    pub fn get_allocations(&self) -> Vec<&MemoryAllocation> {
        self.allocations.values().collect()
    }
    
    /// Get active (not freed) memory allocations
    pub fn get_active_allocations(&self) -> Vec<&MemoryAllocation> {
        self.allocations
            .values()
            .filter(|a| !a.freed)
            .collect()
    }
    
    /// Get potential memory leaks (allocations that have been active for a long time)
    pub fn get_potential_leaks(&self, threshold: Duration) -> Vec<&MemoryAllocation> {
        let now = Instant::now();
        
        self.allocations
            .values()
            .filter(|a| !a.freed && now.duration_since(a.timestamp) > threshold)
            .collect()
    }
    
    /// Calculate memory usage statistics
    pub fn calculate_stats(&self) -> MemoryStats {
        let mut stats = MemoryStats {
            current_bytes: 0,
            peak_bytes: 0,
            total_allocations: 0,
            active_allocations: 0,
            freed_allocations: 0,
            allocation_rate: 0.0,
            category_bytes: HashMap::new(),
            size_distribution: HashMap::new(),
        };
        
        // Calculate current memory usage
        stats.current_bytes = self.allocations
            .values()
            .filter(|a| !a.freed)
            .map(|a| a.size)
            .sum();
        
        // Calculate peak memory usage from snapshots
        stats.peak_bytes = self.snapshots
            .iter()
            .map(|s| s.total_bytes)
            .max()
            .unwrap_or(0);
        
        // Calculate allocation counts
        stats.total_allocations = self.allocations.len();
        stats.active_allocations = self.allocations.values().filter(|a| !a.freed).count();
        stats.freed_allocations = self.allocations.values().filter(|a| a.freed).count();
        
        // Calculate allocation rate (allocations per second)
        if let (Some(first), Some(last)) = (self.snapshots.first(), self.snapshots.last()) {
            let duration = last.timestamp.duration_since(first.timestamp);
            if duration.as_secs() > 0 {
                stats.allocation_rate = stats.total_allocations as f64 / duration.as_secs_f64();
            }
        }
        
        // Calculate memory usage by category
        for allocation in self.allocations.values() {
            if allocation.freed {
                continue;
            }
            
            *stats.category_bytes.entry(allocation.category.clone()).or_insert(0) += allocation.size;
        }
        
        // Calculate size distribution
        for allocation in self.allocations.values() {
            if allocation.freed {
                continue;
            }
            
            // Group by size range
            let size_range = match allocation.size {
                0..=64 => "0-64",
                65..=256 => "65-256",
                257..=1024 => "257-1024",
                1025..=4096 => "1025-4096",
                4097..=16384 => "4097-16384",
                16385..=65536 => "16385-65536",
                _ => ">65536",
            };
            
            *stats.size_distribution.entry(size_range.to_string()).or_insert(0) += 1;
        }
        
        stats
    }
    
    /// Clear all snapshots and allocations
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.allocations.clear();
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_bytes: usize,
    /// Total number of allocations
    pub total_allocations: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Number of freed allocations
    pub freed_allocations: usize,
    /// Allocation rate (allocations per second)
    pub allocation_rate: f64,
    /// Memory usage by category in bytes
    pub category_bytes: HashMap<String, usize>,
    /// Allocation size distribution
    pub size_distribution: HashMap<String, usize>,
}

/// Global memory profiler instance
lazy_static::lazy_static! {
    static ref MEMORY_PROFILER: Arc<Mutex<MemoryProfiler>> = Arc::new(Mutex::new(MemoryProfiler::new()));
}

/// Get the global memory profiler instance
pub fn get_memory_profiler() -> Arc<Mutex<MemoryProfiler>> {
    MEMORY_PROFILER.clone()
}

/// Take a memory snapshot
pub fn take_snapshot() -> MemorySnapshot {
    let mut profiler = MEMORY_PROFILER.lock().unwrap();
    profiler.take_snapshot()
}

/// Track a memory allocation
pub fn track_allocation(size: usize, category: impl Into<String>) -> String {
    let mut profiler = MEMORY_PROFILER.lock().unwrap();
    profiler.track_allocation(size, category)
}

/// Mark a memory allocation as freed
pub fn free_allocation(id: &str) {
    let mut profiler = MEMORY_PROFILER.lock().unwrap();
    profiler.free_allocation(id);
}

/// Get memory usage statistics
pub fn get_memory_stats() -> MemoryStats {
    let profiler = MEMORY_PROFILER.lock().unwrap();
    profiler.calculate_stats()
}

/// Get potential memory leaks
pub fn get_potential_leaks(threshold_secs: u64) -> Vec<MemoryAllocation> {
    let profiler = MEMORY_PROFILER.lock().unwrap();
    profiler
        .get_potential_leaks(Duration::from_secs(threshold_secs))
        .into_iter()
        .cloned()
        .collect()
}

/// Clear all memory profiling data
pub fn clear_memory_profiling_data() {
    let mut profiler = MEMORY_PROFILER.lock().unwrap();
    profiler.clear();
}

/// Configure the memory profiler
pub fn configure_memory_profiler(
    max_snapshots: usize,
    track_allocations: bool,
    capture_stack_traces: bool,
) {
    let mut profiler = MEMORY_PROFILER.lock().unwrap();
    *profiler = MemoryProfiler::new()
        .with_max_snapshots(max_snapshots)
        .with_track_allocations(track_allocations)
        .with_capture_stack_traces(capture_stack_traces);
}

/// Initialize the memory profiler
pub fn init() {
    info!("Initializing memory profiler");
    
    // Configure with default settings
    configure_memory_profiler(100, true, false);
}

/// Export memory profiling data as JSON
pub fn export_memory_profiling_data() -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct ExportData {
        stats: MemoryStats,
        snapshots: Vec<MemorySnapshotExport>,
        leaks: Vec<MemoryAllocation>,
    }
    
    #[derive(Serialize)]
    struct MemorySnapshotExport {
        timestamp_ms: u64,
        total_bytes: usize,
        category_bytes: HashMap<String, usize>,
        allocation_count: usize,
    }
    
    let profiler = MEMORY_PROFILER.lock().unwrap();
    let stats = profiler.calculate_stats();
    
    // Export snapshots
    let now = Instant::now();
    let snapshots: Vec<MemorySnapshotExport> = profiler
        .get_snapshots()
        .iter()
        .map(|snapshot| {
            MemorySnapshotExport {
                timestamp_ms: now.duration_since(snapshot.timestamp).as_millis() as u64,
                total_bytes: snapshot.total_bytes,
                category_bytes: snapshot.category_bytes.clone(),
                allocation_count: snapshot.allocation_count,
            }
        })
        .collect();
    
    // Export potential leaks
    let leaks = profiler
        .get_potential_leaks(Duration::from_secs(60))
        .into_iter()
        .cloned()
        .collect();
    
    let export_data = ExportData {
        stats,
        snapshots,
        leaks,
    };
    
    serde_json::to_string_pretty(&export_data)
}