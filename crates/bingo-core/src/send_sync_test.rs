//! Test file to verify Send + Sync bounds for threading
//! This file tests that all key components can be safely shared across threads

use crate::alpha_memory::AlphaMemoryManager;
use crate::beta_network::BetaNetworkManager;
use crate::fact_store::arena_store::ArenaFactStore;
use crate::memory_pools::MemoryPoolManager;
use crate::parallel_rete::{ParallelReteConfig, ParallelReteProcessor};
use bingo_calculator::calculator::Calculator;
use std::sync::{Arc, RwLock};

/// Test that key components implement Send + Sync
fn _test_send_sync_bounds() {
    // Test memory pools
    fn test_memory_pools<T: Send + Sync>(_t: T) {}
    test_memory_pools(MemoryPoolManager::new());

    // Test alpha memory manager
    fn test_alpha_memory<T: Send + Sync>(_t: T) {}
    test_alpha_memory(AlphaMemoryManager::new());

    // Test beta network manager
    fn test_beta_network<T: Send + Sync>(_t: T) {}
    test_beta_network(BetaNetworkManager::new());

    // Test parallel RETE processor
    fn test_parallel_rete<T: Send + Sync>(_t: T) {}
    test_parallel_rete(ParallelReteProcessor::new(ParallelReteConfig::default()));

    // Test Calculator (read-only, should be Send + Sync)
    fn test_calculator<T: Send + Sync>(_t: T) {}
    test_calculator(Calculator::new());

    // Test thread-safe fact store
    fn test_fact_store<T: Send + Sync>(_t: T) {}
    test_fact_store(ArenaFactStore::new_shared());

    // Test Arc-wrapped components (what we actually use in parallel code)
    fn test_arc_wrapped<T: Send + Sync>(_t: Arc<RwLock<T>>) {}
    test_arc_wrapped(Arc::new(RwLock::new(AlphaMemoryManager::new())));
    test_arc_wrapped(Arc::new(RwLock::new(BetaNetworkManager::new())));

    // Test Arc-wrapped Calculator (for shared access)
    fn test_arc_calculator<T: Send + Sync>(_t: Arc<T>) {}
    test_arc_calculator(Arc::new(Calculator::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_components_are_send_sync() {
        // This test will only compile if all components are Send + Sync
        _test_send_sync_bounds();
    }
}
