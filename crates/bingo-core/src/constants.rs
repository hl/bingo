/// Core system constants used throughout the Bingo RETE engine
///
/// This module centralizes all magic numbers and configuration constants
/// to improve maintainability and consistency across the codebase.
/// Fact ID management constants
pub mod fact_ids {
    /// Starting ID for auto-generated facts to avoid conflicts with input facts
    /// Facts created by rules start at this ID to distinguish from user facts
    pub const CREATED_FACT_ID_OFFSET: u64 = 1_000_000;

    /// Maximum ID for user-provided facts
    /// IDs above this threshold are considered auto-generated
    pub const MAX_USER_FACT_ID: u64 = 1_000_000;
}

/// Performance and memory management constants
pub mod performance {
    /// Base memory allocation for performance calculations (1MB)
    pub const BASE_MEMORY_BYTES: usize = 1_000_000;

    /// Default cache size for various caching mechanisms
    pub const DEFAULT_CACHE_SIZE: usize = 1024;

    /// Memory pool initial capacity
    pub const MEMORY_POOL_INITIAL_CAPACITY: usize = 256;
}

/// Production configuration limits
pub mod limits {
    /// Maximum number of facts the system can handle
    pub const MAX_FACTS: u64 = 1_000_000;

    /// Maximum number of rules per engine instance
    pub const MAX_RULES: u64 = 100_000;

    /// Maximum fact size in bytes
    pub const MAX_FACT_SIZE_BYTES: usize = 1024;
}

/// Profiler time bucket constants (in microseconds)
pub mod profiler {
    /// Time buckets for performance profiling
    pub const TIME_BUCKET_500MS_US: u64 = 500_000;
    pub const TIME_BUCKET_1S_US: u64 = 1_000_000;
    pub const TIME_BUCKET_5S_US: u64 = 5_000_000;
}
