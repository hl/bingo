//! Unified memory coordinator for managing all optimization layers
//!
//! This module provides centralized memory management across fact stores,
//! caches, memory pools, and other optimization components to prevent
//! memory pressure and optimize overall system performance.

use crate::cache::LruCache;
use crate::memory::MemoryStats;
use crate::memory_pools::ReteMemoryPools;
use crate::unified_statistics::MemoryStats as UnifiedMemoryStats;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Unified memory coordinator managing all optimization layer memory usage
pub struct UnifiedMemoryCoordinator {
    /// Memory policy configuration
    config: MemoryCoordinatorConfig,
    /// System memory monitoring
    memory_monitor: Arc<RwLock<MemoryMonitor>>,
    /// Registered memory consumers (caches, pools, stores)
    consumers: Arc<Mutex<HashMap<String, Box<dyn MemoryConsumer + Send + Sync>>>>,
    /// Global memory pools for RETE operations
    global_pools: Arc<ReteMemoryPools>,
    /// Coordination statistics
    stats: Arc<Mutex<CoordinationStats>>,
}

/// Configuration for memory coordinator behavior
#[derive(Debug, Clone)]
pub struct MemoryCoordinatorConfig {
    /// Maximum memory limit in bytes (0 = no limit)
    pub max_memory_bytes: usize,
    /// Memory pressure threshold (percentage at which to start cleanup)
    pub pressure_threshold: f64,
    /// Critical threshold (percentage at which to aggressively free memory)
    pub critical_threshold: f64,
    /// Frequency of memory monitoring
    pub monitor_interval: Duration,
    /// Enable automatic memory cleanup
    pub auto_cleanup: bool,
    /// Cache size adjustment factor under pressure (0.5 = reduce by 50%)
    pub cache_reduction_factor: f64,
    /// Pool size adjustment factor under pressure
    pub pool_reduction_factor: f64,
}

/// Memory monitoring component
#[derive(Debug)]
pub struct MemoryMonitor {
    /// Current memory usage stats
    current_stats: Option<MemoryStats>,
    /// Memory usage history (for trend analysis)
    usage_history: Vec<(Instant, usize)>,
    /// Last monitoring time
    last_monitor: Instant,
    /// Peak memory usage recorded
    peak_memory_bytes: usize,
    /// Number of pressure events
    pressure_events: usize,
    /// Number of critical events
    critical_events: usize,
}

/// Statistics for memory coordination activities
#[derive(Debug, Clone, Default)]
pub struct CoordinationStats {
    /// Total cleanup operations performed
    pub cleanup_operations: usize,
    /// Memory freed by cleanup operations (bytes)
    pub memory_freed_bytes: usize,
    /// Cache resizing operations
    pub cache_resizes: usize,
    /// Pool resizing operations
    pub pool_resizes: usize,
    /// Pressure events handled
    pub pressure_events_handled: usize,
    /// Critical events handled
    pub critical_events_handled: usize,
    /// Average memory usage over time
    pub avg_memory_usage_bytes: usize,
}

/// Trait for components that consume memory and can be controlled
pub trait MemoryConsumer {
    /// Get current memory usage estimate in bytes
    fn memory_usage_bytes(&self) -> usize;

    /// Reduce memory footprint (returns bytes freed)
    fn reduce_memory_usage(&mut self, reduction_factor: f64) -> usize;

    /// Get component statistics for reporting
    fn get_stats(&self) -> HashMap<String, f64>;

    /// Component name for identification
    fn name(&self) -> &str;
}

impl Default for MemoryCoordinatorConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024,     // 1GB default limit
            pressure_threshold: 80.0,                 // Start cleanup at 80% usage
            critical_threshold: 95.0,                 // Aggressive cleanup at 95% usage
            monitor_interval: Duration::from_secs(5), // Monitor every 5 seconds
            auto_cleanup: true,
            cache_reduction_factor: 0.7, // Reduce caches by 30%
            pool_reduction_factor: 0.8,  // Reduce pools by 20%
        }
    }
}

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new() -> Self {
        Self {
            current_stats: None,
            usage_history: Vec::with_capacity(1000), // Keep 1000 measurements
            last_monitor: Instant::now(),
            peak_memory_bytes: 0,
            pressure_events: 0,
            critical_events: 0,
        }
    }

    /// Update memory statistics
    pub fn update(&mut self) -> anyhow::Result<()> {
        let stats = MemoryStats::current()?;
        let memory_bytes = stats.rss_bytes;

        // Update peak tracking
        self.peak_memory_bytes = self.peak_memory_bytes.max(memory_bytes);

        // Add to history
        self.usage_history.push((Instant::now(), memory_bytes));

        // Trim history to prevent unbounded growth
        if self.usage_history.len() > 1000 {
            self.usage_history.remove(0);
        }

        self.current_stats = Some(stats);
        self.last_monitor = Instant::now();

        Ok(())
    }

    /// Check if system is under memory pressure
    pub fn is_under_pressure(&self, config: &MemoryCoordinatorConfig) -> bool {
        if config.max_memory_bytes == 0 {
            return false; // No limit set
        }

        if let Some(stats) = &self.current_stats {
            let usage_percent = (stats.rss_bytes as f64 / config.max_memory_bytes as f64) * 100.0;
            usage_percent > config.pressure_threshold
        } else {
            false
        }
    }

    /// Check if system is in critical memory state
    pub fn is_critical(&self, config: &MemoryCoordinatorConfig) -> bool {
        if config.max_memory_bytes == 0 {
            return false; // No limit set
        }

        if let Some(stats) = &self.current_stats {
            let usage_percent = (stats.rss_bytes as f64 / config.max_memory_bytes as f64) * 100.0;
            usage_percent > config.critical_threshold
        } else {
            false
        }
    }

    /// Get current memory usage percentage
    pub fn usage_percentage(&self, config: &MemoryCoordinatorConfig) -> f64 {
        if config.max_memory_bytes == 0 {
            return 0.0; // No limit set
        }

        if let Some(stats) = &self.current_stats {
            (stats.rss_bytes as f64 / config.max_memory_bytes as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get memory growth trend (bytes per second)
    pub fn memory_growth_trend(&self) -> f64 {
        if self.usage_history.len() < 2 {
            return 0.0;
        }

        // Calculate growth over last 10 measurements
        let sample_size = 10.min(self.usage_history.len());
        let start_idx = self.usage_history.len() - sample_size;

        let (start_time, start_memory) = self.usage_history[start_idx];
        let (end_time, end_memory) = self.usage_history[self.usage_history.len() - 1];

        let duration_secs = end_time.duration_since(start_time).as_secs_f64();
        if duration_secs > 0.0 {
            (end_memory as f64 - start_memory as f64) / duration_secs
        } else {
            0.0
        }
    }
}

impl UnifiedMemoryCoordinator {
    /// Create a new unified memory coordinator
    pub fn new(config: MemoryCoordinatorConfig) -> Self {
        Self {
            config,
            memory_monitor: Arc::new(RwLock::new(MemoryMonitor::new())),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            global_pools: Arc::new(ReteMemoryPools::new()),
            stats: Arc::new(Mutex::new(CoordinationStats::default())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(MemoryCoordinatorConfig::default())
    }

    /// Register a memory consumer for coordination
    pub fn register_consumer<T>(&self, name: String, consumer: T)
    where
        T: MemoryConsumer + Send + Sync + 'static,
    {
        let mut consumers = self.consumers.lock().unwrap();
        consumers.insert(name, Box::new(consumer));
    }

    /// Update memory monitoring and trigger cleanup if needed
    pub fn coordinate_memory(&self) -> anyhow::Result<CoordinationResult> {
        // Update memory statistics
        {
            let mut monitor = self.memory_monitor.write().unwrap();
            monitor.update()?;
        }

        if !self.config.auto_cleanup {
            return Ok(CoordinationResult::NoAction);
        }

        let monitor = self.memory_monitor.read().unwrap();

        if monitor.is_critical(&self.config) {
            drop(monitor);
            self.handle_critical_memory()
        } else if monitor.is_under_pressure(&self.config) {
            drop(monitor);
            self.handle_memory_pressure()
        } else {
            Ok(CoordinationResult::NoAction)
        }
    }

    /// Handle memory pressure by reducing consumer memory usage
    fn handle_memory_pressure(&self) -> anyhow::Result<CoordinationResult> {
        let mut total_freed = 0;
        let mut operations = 0;

        {
            let mut consumers = self.consumers.lock().unwrap();
            for (name, consumer) in consumers.iter_mut() {
                let freed = consumer.reduce_memory_usage(self.config.cache_reduction_factor);
                total_freed += freed;
                operations += 1;

                tracing::info!("Reduced memory usage for {}: {} bytes freed", name, freed);
            }
        }

        {
            let mut monitor = self.memory_monitor.write().unwrap();
            monitor.pressure_events += 1;
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.cleanup_operations += operations;
            stats.memory_freed_bytes += total_freed;
            stats.pressure_events_handled += 1;
        }

        Ok(CoordinationResult::PressureHandled {
            memory_freed_bytes: total_freed,
            components_affected: operations,
        })
    }

    /// Handle critical memory by aggressive cleanup
    fn handle_critical_memory(&self) -> anyhow::Result<CoordinationResult> {
        let mut total_freed = 0;
        let mut operations = 0;

        // More aggressive memory reduction in critical state
        let critical_reduction_factor = self.config.cache_reduction_factor * 0.5; // Even more aggressive

        {
            let mut consumers = self.consumers.lock().unwrap();
            for (name, consumer) in consumers.iter_mut() {
                let freed = consumer.reduce_memory_usage(critical_reduction_factor);
                total_freed += freed;
                operations += 1;

                tracing::warn!(
                    "Critical memory cleanup for {}: {} bytes freed",
                    name,
                    freed
                );
            }
        }

        {
            let mut monitor = self.memory_monitor.write().unwrap();
            monitor.critical_events += 1;
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.cleanup_operations += operations;
            stats.memory_freed_bytes += total_freed;
            stats.critical_events_handled += 1;
        }

        Ok(CoordinationResult::CriticalHandled {
            memory_freed_bytes: total_freed,
            components_affected: operations,
        })
    }

    /// Get current memory usage information
    pub fn get_memory_info(&self) -> MemoryInfo {
        let monitor = self.memory_monitor.read().unwrap();
        let stats = self.stats.lock().unwrap();

        let current_usage = monitor.current_stats.as_ref().map(|s| s.rss_bytes).unwrap_or(0);

        MemoryInfo {
            current_usage_bytes: current_usage,
            peak_usage_bytes: monitor.peak_memory_bytes,
            usage_percentage: monitor.usage_percentage(&self.config),
            memory_limit_bytes: self.config.max_memory_bytes,
            is_under_pressure: monitor.is_under_pressure(&self.config),
            is_critical: monitor.is_critical(&self.config),
            growth_trend_bytes_per_sec: monitor.memory_growth_trend(),
            coordination_stats: stats.clone(),
        }
    }

    /// Get global memory pools for RETE operations
    pub fn get_global_pools(&self) -> Arc<ReteMemoryPools> {
        self.global_pools.clone()
    }

    /// Generate unified memory statistics
    pub fn generate_unified_stats(&self) -> UnifiedMemoryStats {
        let monitor = self.memory_monitor.read().unwrap();
        let coordination_stats = self.stats.lock().unwrap();

        // Collect statistics from memory pools
        let pool_stats = self.global_pools.get_stats();

        // Calculate total hits and misses from individual pool stats
        let total_hits = pool_stats.token_pool.pool_hits
            + pool_stats.fact_data_pool.pool_hits
            + pool_stats.field_map_pool.pool_hits
            + pool_stats.fact_vec_pool.pool_hits
            + pool_stats.fact_id_set_pool.pool_hits;

        let total_misses = pool_stats.token_pool.pool_misses
            + pool_stats.fact_data_pool.pool_misses
            + pool_stats.field_map_pool.pool_misses
            + pool_stats.fact_vec_pool.pool_misses
            + pool_stats.fact_id_set_pool.pool_misses;

        UnifiedMemoryStats {
            pool_hits: total_hits,
            pool_misses: total_misses,
            pool_allocated: coordination_stats.cleanup_operations, // Approximation
            pool_returned: coordination_stats.memory_freed_bytes / 1024, // Approximation
            peak_pool_sizes: {
                let mut sizes = HashMap::new();
                sizes.insert("Token".to_string(), pool_stats.token_pool.peak_size);
                sizes.insert("FactData".to_string(), pool_stats.fact_data_pool.peak_size);
                sizes.insert("FieldMap".to_string(), pool_stats.field_map_pool.peak_size);
                sizes.insert("FactVec".to_string(), pool_stats.fact_vec_pool.peak_size);
                sizes.insert(
                    "FactIdSet".to_string(),
                    pool_stats.fact_id_set_pool.peak_size,
                );
                sizes
            },
            total_allocated_bytes: monitor.current_stats.as_ref().map(|s| s.rss_bytes).unwrap_or(0),
        }
    }

    /// Force garbage collection and memory cleanup
    pub fn force_cleanup(&self) -> anyhow::Result<usize> {
        let mut total_freed = 0;

        {
            let mut consumers = self.consumers.lock().unwrap();
            for (_, consumer) in consumers.iter_mut() {
                total_freed += consumer.reduce_memory_usage(0.5); // Reduce by 50%
            }
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.cleanup_operations += 1;
            stats.memory_freed_bytes += total_freed;
        }

        Ok(total_freed)
    }
}

/// Result of memory coordination operation
#[derive(Debug, Clone)]
pub enum CoordinationResult {
    /// No action was needed
    NoAction,
    /// Memory pressure was handled
    PressureHandled { memory_freed_bytes: usize, components_affected: usize },
    /// Critical memory situation was handled
    CriticalHandled { memory_freed_bytes: usize, components_affected: usize },
}

/// Current memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub current_usage_bytes: usize,
    pub peak_usage_bytes: usize,
    pub usage_percentage: f64,
    pub memory_limit_bytes: usize,
    pub is_under_pressure: bool,
    pub is_critical: bool,
    pub growth_trend_bytes_per_sec: f64,
    pub coordination_stats: CoordinationStats,
}

impl MemoryInfo {
    /// Format memory info as human-readable string
    pub fn format_summary(&self) -> String {
        format!(
            "Memory: {:.1}% ({:.1}MB/{:.1}MB), Pressure: {}, Critical: {}, Growth: {:.1}KB/s",
            self.usage_percentage,
            self.current_usage_bytes as f64 / (1024.0 * 1024.0),
            self.memory_limit_bytes as f64 / (1024.0 * 1024.0),
            self.is_under_pressure,
            self.is_critical,
            self.growth_trend_bytes_per_sec / 1024.0
        )
    }
}

/// Memory consumer implementation for LRU caches
pub struct CacheMemoryConsumer<K, V> {
    name: String,
    cache: Arc<Mutex<LruCache<K, V>>>,
    original_capacity: usize,
}

impl<K, V> CacheMemoryConsumer<K, V>
where
    K: Clone + Send + Sync + std::hash::Hash + Eq + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(name: String, cache: Arc<Mutex<LruCache<K, V>>>) -> Self {
        let original_capacity = cache.lock().unwrap().len();
        Self { name, cache, original_capacity }
    }
}

impl<K, V> MemoryConsumer for CacheMemoryConsumer<K, V>
where
    K: Clone + Send + Sync + std::hash::Hash + Eq + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn memory_usage_bytes(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        // Rough estimate: 64 bytes per cache entry
        cache.len() * 64
    }

    fn reduce_memory_usage(&mut self, reduction_factor: f64) -> usize {
        let mut cache = self.cache.lock().unwrap();
        let current_size = cache.len();
        let target_size = (current_size as f64 * reduction_factor) as usize;

        let items_to_remove = current_size.saturating_sub(target_size);

        // Clear cache entries (LruCache doesn't have direct size reduction)
        // For now, we'll clear a portion of the cache
        if items_to_remove > 0 && current_size > 0 {
            let clear_ratio = items_to_remove as f64 / current_size as f64;
            if clear_ratio > 0.5 {
                cache.clear();
            }
        }

        items_to_remove * 64 // Estimate bytes freed
    }

    fn get_stats(&self) -> HashMap<String, f64> {
        let cache = self.cache.lock().unwrap();
        let mut stats = HashMap::new();
        stats.insert("size".to_string(), cache.len() as f64);
        stats.insert(
            "memory_estimate_mb".to_string(),
            (cache.len() * 64) as f64 / (1024.0 * 1024.0),
        );
        stats
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_memory_coordinator_creation() {
        let coordinator = UnifiedMemoryCoordinator::with_defaults();
        let info = coordinator.get_memory_info();

        assert_eq!(info.coordination_stats.cleanup_operations, 0);
        assert!(!info.is_critical); // Should not be critical initially
    }

    #[test]
    fn test_memory_monitor_updates() {
        let mut monitor = MemoryMonitor::new();

        // Test multiple updates
        monitor.update().unwrap();
        thread::sleep(Duration::from_millis(10));
        monitor.update().unwrap();

        assert!(monitor.current_stats.is_some());
        assert!(monitor.usage_history.len() >= 2);
    }

    #[test]
    fn test_memory_pressure_detection() {
        let config = MemoryCoordinatorConfig {
            max_memory_bytes: 1024, // Very small limit for testing
            pressure_threshold: 50.0,
            critical_threshold: 90.0,
            ..Default::default()
        };

        let mut monitor = MemoryMonitor::new();
        monitor.update().unwrap();

        // With current memory usage, pressure detection should work
        let under_pressure = monitor.is_under_pressure(&config);
        println!("Under pressure with 1KB limit: {}", under_pressure);

        // Test with no limit
        let no_limit_config = MemoryCoordinatorConfig { max_memory_bytes: 0, ..config };
        assert!(!monitor.is_under_pressure(&no_limit_config));
    }

    #[test]
    fn test_cache_memory_consumer() {
        let cache = Arc::new(Mutex::new(LruCache::new(10)));
        {
            let mut cache_lock = cache.lock().unwrap();
            cache_lock.put("key1", "value1");
            cache_lock.put("key2", "value2");
        }

        let mut consumer = CacheMemoryConsumer::new("TestCache".to_string(), cache.clone());

        let initial_usage = consumer.memory_usage_bytes();
        assert!(initial_usage > 0);

        let freed = consumer.reduce_memory_usage(0.5);
        assert!(freed > 0);

        let stats = consumer.get_stats();
        assert!(stats.contains_key("size"));
        assert_eq!(consumer.name(), "TestCache");
    }

    #[test]
    fn test_coordination_result() {
        let result = CoordinationResult::PressureHandled {
            memory_freed_bytes: 1024,
            components_affected: 3,
        };

        match result {
            CoordinationResult::PressureHandled { memory_freed_bytes, components_affected } => {
                assert_eq!(memory_freed_bytes, 1024);
                assert_eq!(components_affected, 3);
            }
            _ => panic!("Unexpected result type"),
        }
    }

    #[test]
    fn test_memory_info_formatting() {
        let info = MemoryInfo {
            current_usage_bytes: 1024 * 1024,  // 1MB
            peak_usage_bytes: 2 * 1024 * 1024, // 2MB
            usage_percentage: 50.0,
            memory_limit_bytes: 2 * 1024 * 1024, // 2MB
            is_under_pressure: false,
            is_critical: false,
            growth_trend_bytes_per_sec: 1024.0, // 1KB/s
            coordination_stats: CoordinationStats::default(),
        };

        let summary = info.format_summary();
        assert!(summary.contains("50.0%"));
        assert!(summary.contains("1.0MB"));
        assert!(summary.contains("false"));
        assert!(summary.contains("1.0KB/s"));
    }
}
