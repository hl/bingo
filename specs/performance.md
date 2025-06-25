# Performance Specification

## Hardware Baseline

### Reference Configuration
- **CPU**: Apple M1 Pro (8-core performance + 2-core efficiency)
- **RAM**: 32GB unified memory
- **OS**: macOS 14.5+ 
- **Rust**: 1.75+ stable
- **Disk**: NVMe SSD for benchmark data storage

### Target Environment Scaling
- **Production**: AWS c6i.4xlarge (16 vCPU, 32GB RAM) or equivalent
- **Testing**: 8+ core x86_64 with 16GB+ RAM minimum
- **Development**: 4+ core with 8GB+ RAM minimum

## Performance Targets (Phased)

### Phase 1: MVP Targets (Current) - CRITICAL BASELINE
- **Throughput**: 100K facts processed in <1 second (measured baseline)
- **Memory Usage**: <50MB RSS for 100K facts (RSS tracking required)
- **Rules**: 100 built-in rules without degradation
- **Network Size**: <1,000 RETE nodes
- **Memory Issues**: Fix fact cloning that doubles memory usage
- **Goal**: Establish empirical baseline with automated benchmarking and regression detection

### Phase 2: Intermediate Targets
- **Throughput**: 1M facts processed in <2 seconds  
- **Memory Usage**: <150MB RSS for 1M facts
- **Rules**: 500 rules with basic optimization
- **Network Size**: <5,000 RETE nodes
- **Goal**: Validate scaling patterns and memory management

### Phase 3: Production Targets (JSON Rules + Calculator DSL) - UPDATED
- **Throughput**: 1M facts processed in <30 seconds (enterprise production target)
- **JSON Rule Compilation**: <50ms per rule compilation
- **Memory Usage**: <4GB RSS for 1M facts (enterprise production target)
- **Latency**: P95 < 500ms for rule evaluation (private network optimized)
- **Rule Capacity**: 2,000 rules (built-in + JSON) without performance degradation
- **Calculator DSL**: Business-friendly rules compile to optimal RETE performance
- **Realistic Scaling**: 100K facts in <3s, 500K facts in <10s with <1.3GB memory

### Measurement Requirements
- **Automated Benchmarks**: Criterion-based harness in CI/CD
- **Memory Profiling**: Valgrind, heaptrack, or Rust-specific tooling
- **Regression Detection**: Performance alerts on >10% degradation
- **Load Testing**: Representative datasets with realistic rule complexity

## Memory Management Strategy

### ✅ Implemented Optimizations

#### Advanced Memory Sharing (COMPLETED)
- **Token Sharing**: Arc-based FactIdSet reduces memory duplication in RETE network
- **LRU Caching**: Intelligent caching of frequently accessed facts and tokens
- **Fact Partitioning**: Distributed storage for very large datasets (1M+ facts)
- **Memory Pooling**: Token pools reduce allocation overhead in high-throughput scenarios
- **Smart Factory**: Automatic selection of optimal storage strategy based on dataset size

#### Benchmarked Performance Improvements
- **Memory Usage**: 30-50% reduction in token memory through Arc sharing
- **Cache Hit Rates**: 80%+ cache hit rates for frequently accessed facts
- **Allocation Overhead**: Significant reduction in token allocation/deallocation cycles
- **Partitioning Efficiency**: Near-linear scaling for datasets exceeding 1M facts

### Phased Approach to Allocation

#### Phase 1: Proven Libraries Foundation (COMPLETED ✅)
```rust
// Implemented fact store abstraction with multiple strategies
trait FactStore {
    fn insert(&mut self, fact: Fact) -> FactId;
    fn get(&self, id: FactId) -> Option<&Fact>;
    fn extend_from_vec(&mut self, facts: Vec<Fact>);
    fn len(&self) -> usize;
    fn clear(&mut self);
    fn find_by_field(&self, field: &str, value: &FactValue) -> Vec<&Fact>;
    fn find_by_criteria(&self, criteria: &[(String, FactValue)]) -> Vec<&Fact>;
}

struct VecFactStore {
    facts: Vec<Fact>,                    // Simple Vec for baseline
    field_indexes: HashMap<String, HashMap<String, Vec<FactId>>>,
}

struct CachedFactStore {
    inner: VecFactStore,
    cache: LruCache<FactId, Fact>,       // LRU caching layer
}

struct PartitionedFactStore {
    partitions: Vec<VecFactStore>,       // Distributed storage
    partition_count: usize,
}
```

#### Phase 2: Custom Optimizations (COMPLETED ✅)
- **Token Sharing**: Arc-based memory sharing implemented and benchmarked
- **Memory Pooling**: TokenPool implementation reduces allocation overhead
- **LRU Caching**: Intelligent cache management for hot facts
- **Fact Partitioning**: Distributed storage for memory-efficient large datasets

### Memory Layout (Target)
- **Fact Objects**: Start with 200 bytes per fact, optimize down to 100 bytes
- **RETE Network**: HashMap-based node storage initially
- **Aggregation State**: Standard collections, optimize to ~1KB per group later
- **Safety First**: Use Rust's standard allocator with careful measurement

### Risk Mitigation
- **Invariant Testing**: Use `loom` and `miri` for memory safety validation
- **Abstraction Layer**: Allow swapping allocators without changing algorithm code
- **Measurement First**: Profile before optimizing, benchmark after changes
- **Gradual Migration**: Phase custom allocators only when standard approaches hit limits

## Optimisation Techniques

### Data Structures
- **ahash**: SIMD-accelerated hashing for fact lookup
- **Roaring Bitmaps**: Compressed fact sets for large selections
- **Slot Maps**: Stable references with O(1) access
- **Custom Collections**: Specialised containers for RETE data

### Algorithmic Optimisations
- **Node Sharing**: Eliminate duplicate alpha conditions
- **Condition Ordering**: Most selective conditions first
- **Hash Indices**: O(1) fact retrieval by attributes
- **Lazy Evaluation**: Defer expensive computations

### Concurrency Optimisations
- **Partitioning**: Split 3M facts across 8-16 worker tasks (~200K facts per partition)
- **Lock-Free**: Single-threaded within partitions
- **Batch Processing**: Group facts for efficient processing (1K-10K fact batches)
- **Pipeline**: Overlap computation and I/O
- **Work Stealing**: Dynamic load balancing between partitions

## Benchmarking Strategy

### Micro-Benchmarks
```rust
#[bench]
fn bench_fact_insertion(b: &mut Bencher) {
    let mut arena = FactArena::new();
    b.iter(|| {
        black_box(arena.insert_fact(test_fact()));
    });
}
```

### System Benchmarks
- **End-to-End**: Full request/response cycle
- **Rule Compilation**: Time to build RETE network
- **Memory Usage**: RSS and allocation tracking
- **Scalability**: Performance vs. dataset size

### Performance Regression Testing
- **Criterion**: Statistical benchmarking with CI integration
- **Flamegraphs**: CPU profiling for hotspot identification
- **Memory Profiling**: Allocation tracking and leak detection
- **Load Testing**: Sustained throughput measurement

## Monitoring and Telemetry

### Key Metrics
```rust
struct PerformanceMetrics {
    evaluation_time_ns: u64,
    memory_used_bytes: usize,
    facts_processed: usize,
    rules_fired: usize,
}
```

### Tracing Integration
- **Span Timing**: Automatic performance measurement
- **Memory Tracking**: Arena usage in spans
- **Throughput Metrics**: Facts/second calculation
- **Error Rates**: Performance degradation detection

### Performance Alerts
- **Memory Threshold**: Alert when >250MB RSS
- **Latency Threshold**: Alert when P95 >1000ms
- **Throughput Drop**: Alert when <1M facts/second
- **Error Rate**: Alert when >1% evaluation failures

## Scaling Characteristics

### Horizontal Scaling
- **Partition Strategy**: Split by employee ID ranges
- **Load Balancing**: Distribute requests across partitions
- **Aggregation**: Merge results from multiple partitions
- **Consistency**: Ensure deterministic results

### Vertical Scaling
- **Multi-Core**: Utilise all available CPU cores
- **Memory Hierarchy**: Optimise for cache locality
- **SIMD**: Vectorised operations where possible
- **Async I/O**: Non-blocking network operations

### Performance vs. Dataset Size
- **Linear Scaling**: O(n) performance with fact count up to 5M facts
- **Rule Complexity**: Impact of rule interdependencies (2K rules baseline)
- **Memory Growth**: Sublinear memory usage growth with partitioning
- **Network Size**: RETE network scaling characteristics (target: <15K nodes for 2K rules)