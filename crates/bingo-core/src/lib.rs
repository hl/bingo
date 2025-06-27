use tracing::{debug, instrument};

pub mod cache;
pub mod calculator;
pub mod calculator_cache;
pub mod calculators;
pub mod debugging;
pub mod engine;
pub mod fact_store;
pub mod fast_lookup;
pub mod field_indexing;
pub mod memory;
pub mod rete_network;
pub mod rete_nodes;
pub mod rule_visualization;
pub mod stream_processing;
/// Core types and functionality for the Bingo RETE rules engine
pub mod types;
pub mod unified_statistics;

pub use cache::{CacheStats, LruCache};
pub use calculator::{
    Calculator as CalculatorEngine, CalculatorExpression, CalculatorResult, EvaluationContext,
};
pub use calculator_cache::{
    CacheUtilization, CachedCalculator, CalculatorCacheStats, ExpressionCacheKey,
};
pub use calculators::{Calculator, CalculatorInputs, get_calculator};
pub use debugging::{DebugManager, OptimizationOpportunity, RulePerformanceProfile, TraceId};
pub use engine::*;
pub use fact_store::{ArenaFactStore, FactStore, VecFactStore};
pub use fast_lookup::{FastFactLookup, FastLookupStats};
pub use field_indexing::{FieldIndexStats, FieldIndexer};
pub use memory::*;
pub use rete_network::{ActionResult, NetworkStats, ReteNetwork, RuleExecutionResult};
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
pub use unified_statistics::{
    CachingStats, CalculatorStats, ComponentStats, FactStorageStats, IndexingStats, MemoryStats,
    UnifiedStats, UnifiedStatsBuilder,
};

/// Initialize the core engine components
#[instrument]
pub fn init() -> anyhow::Result<()> {
    debug!("Initializing Bingo core engine");
    Ok(())
}
