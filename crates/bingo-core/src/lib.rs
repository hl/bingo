use tracing::{debug, instrument};

pub mod adaptive_backends;
pub mod advanced_indexing;
pub mod bloom_filter;
pub mod cache;
pub mod calculator;
pub mod calculator_cache;
pub mod calculators;
pub mod debug_hooks;
pub mod distributed_rete;
pub mod engine;
pub mod fact_store;
pub mod fast_lookup;
pub mod field_indexing;
pub mod incremental_construction;
pub mod incremental_processing;
pub mod memory;
pub mod memory_pools;
pub mod memory_profiler;
pub mod node_sharing;
pub mod optimization_coordinator;
pub mod optimization_demo;
pub mod pattern_cache;
pub mod performance_regression_testing;
pub mod performance_tracking;
pub mod rete_network;
pub mod rete_nodes;
pub mod rule_visualization;
pub mod stream_processing;
/// Core types and functionality for the Bingo RETE rules engine
pub mod types;
pub mod unified_fact_store;
pub mod unified_memory_coordinator;
pub mod unified_statistics;
// Temporarily disabled during development
// pub mod debugging;

pub use adaptive_backends::{
    AccessPattern, AdaptationConfig, AdaptiveBackendSelector, AdaptiveFactStore, BackendMetrics,
    BackendPerformanceSummary, BackendStrategy, DataDistribution, DatasetCharacteristics,
};
pub use advanced_indexing::{
    AdvancedFieldIndexer, AdvancedIndexingStats, BitSet, FieldAnalysis, IndexStrategy,
    IndexStrategyType, OrderedValue,
};
pub use bloom_filter::{
    BloomFilter, BloomFilterStats, FactBloomConfig, FactBloomFilter, FactBloomStats,
};
pub use cache::{CacheStats, LruCache};
pub use calculator::{
    Calculator as CalculatorEngine, CalculatorExpression, CalculatorResult, EvaluationContext,
};
pub use calculator_cache::{
    CacheUtilization, CachedCalculator, CalculatorCacheStats, ExpressionCacheKey,
};
pub use calculators::{Calculator, CalculatorInputs, get_calculator};
pub use debug_hooks::{
    Breakpoint, BreakpointAction, BreakpointCondition, ConsoleEventHook, DebugConfig, DebugContext,
    DebugEvent, DebugEventType, DebugHookManager, DebugOverheadStats, DebugSession, DebugSessionId,
    EventHook, EventSeverity, ExecutionResult, ExecutionStep, ExecutionTrace, FactPattern,
    FileEventHook, PatternOperator, PerformanceRuleHook, RuleFireHook, SessionDebugStats, StepData,
    StepType, TokenPropagationHook,
};
pub use distributed_rete::{
    ClusterConfig, ClusterHealth, ClusterNode, ClusterStatus, DistributedReteNetwork,
    DistributedReteStats, LoadBalancingStrategy, NodeCapabilities, NodeMetrics, NodeStatus,
};
pub use engine::*;
pub use fact_store::{
    CachedFactStore, CachedStoreStats, FactStore, FactStoreFactory, PartitionedFactStore,
    VecFactStore,
};
pub use fast_lookup::{FastFactLookup, FastLookupStats};
pub use field_indexing::{FieldIndexStats, FieldIndexer};
pub use incremental_construction::{
    IncrementalConstructionManager, IncrementalConstructionStats, JoinPathStats, NetworkTopology,
    NodeActivationState,
};
pub use incremental_processing::{
    ChangeTrackingStats, FactChangeTracker, FactState, IncrementalProcessingPlan, ProcessingMode,
};
pub use memory::*;
pub use memory_pools::{ObjectPool, PoolStats, ReteMemoryPools, RetePoolStats, ThreadSafePool};
pub use memory_profiler::{
    ComponentMemoryStats, MemoryOptimizationEvent, MemoryPressureLevel, MemoryPressureThresholds,
    MemoryProfilerConfig, MemoryUsageReport, OptimizationEventType, ReteMemoryProfiler,
};
pub use node_sharing::{
    AlphaNodeSignature, BetaNodeSignature, MemorySavings, NodeSharingRegistry, NodeSharingStats,
};
pub use optimization_coordinator::{
    OptimizationAction, OptimizationConfig, OptimizationCoordinator, OptimizationImprovements,
    OptimizationMetrics, OptimizationReport, OptimizationResult, OptimizationStats,
    PerformanceTrend,
};
pub use optimization_demo::{ImprovementMetrics, OptimizationDemo, PerformanceMetrics};
pub use pattern_cache::{
    AlphaNodePlan, BetaNodePlan, CompilationPlan, PatternCache, PatternCacheStats, PatternSignature,
};
pub use performance_regression_testing::{
    BaselineComparison, BenchmarkConfig, BenchmarkResult, BenchmarkScenario, BenchmarkSummary,
    ComplexityClass, MemoryBaseline, MemoryResult, PerformanceBaseline, PerformanceBenchmarkSuite,
    ScalabilityBaseline, TimingBaseline, TimingResult,
};
pub use performance_tracking::{
    BottleneckType, ExecutionContext, ExecutionRecord, GlobalPerformanceMetrics, MemoryStatistics,
    PerformanceBottleneck, PerformanceConfig, PerformanceSummary, PerformanceTrendPoint,
    RuleExecutionProfile, RuleExecutionTimer, RulePerformanceTracker, SessionSummary,
    TimingStatistics,
};
pub use rete_network::*;
pub use rete_nodes::{
    AlphaNode, BetaNode, FactIdSet, JoinCondition, NodeId, TerminalNode, Token, TokenPool,
    TokenPoolStats,
};
pub use rule_visualization::{
    AlphaNodeInfo, BetaNodeInfo, BottleneckType as RuleBottleneckType, ComplexityRating,
    ConnectionType, DependencyAnalysisSummary, FieldUsageAnalysis,
    NetworkTopology as RuleNetworkTopology, NodeBottleneck, NodeConnection, RuleComplexityMetrics,
    RuleDependencyAnalyzer, RuleDependencyGraph, TerminalNodeInfo, VisualizationFormat,
    VisualizationOptions,
};
pub use stream_processing::{
    AggregationFunction, StreamProcessingStats, StreamProcessor, Timestamp, WindowInstance,
    WindowSpec,
};
pub use types::*;
pub use unified_fact_store::{OptimizedFactStore, OptimizedStoreStats};
pub use unified_memory_coordinator::{
    CacheMemoryConsumer, CoordinationResult, CoordinationStats, MemoryConsumer,
    MemoryCoordinatorConfig, MemoryInfo, UnifiedMemoryCoordinator,
};
pub use unified_statistics::{
    CachingStats, CalculatorStats, ComponentStats, FactStorageStats, IndexingStats, MemoryStats,
    UnifiedStats, UnifiedStatsBuilder,
};
// Temporarily disabled during development
// pub use debugging::{
//     DebugManager, DebugSession, DebugConfig, ExecutionTrace, RulePerformanceProfile,
//     PerformanceBottleneck, OptimizationOpportunity, DebugSessionId, TraceId
// };

/// Initialize the core engine components
#[instrument]
pub fn init() -> anyhow::Result<()> {
    debug!("Initializing Bingo core engine");
    Ok(())
}
