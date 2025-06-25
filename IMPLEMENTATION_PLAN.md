# Bingo RETE Implementation Plan - Week 1 Actions

Based on comprehensive analysis findings, this document outlines immediate actions to address critical issues and establish performance baselines.

## Critical Issues Identified

### 1. **Memory Cloning Problem** (CRITICAL)
**Issue**: `engine.rs:36` and `lib.rs:119` clone facts, doubling memory usage
**Impact**: Makes 3M fact processing impossible due to memory constraints
**Priority**: IMMEDIATE FIX

### 2. **No Performance Baseline** (CRITICAL) 
**Issue**: No automated benchmarks for realistic workloads
**Impact**: Cannot validate performance targets or detect regressions
**Priority**: IMMEDIATE IMPLEMENTATION

### 3. **Hard-coded Memory Management** (HIGH)
**Issue**: Direct `Vec<Fact>` usage prevents future optimization
**Impact**: Difficult to swap allocation strategies later
**Priority**: WEEK 1

## Week 1 Immediate Actions

### Day 1-2: Fix Critical Memory Issues

#### Action 1.1: Fix Fact Cloning in engine.rs
```rust
// Current problematic code in engine.rs:36
self.facts.extend(facts.clone()); // ❌ DOUBLES MEMORY

// Fix: Use move semantics
self.facts.extend(facts.into_iter()); // ✅ NO CLONING
```

#### Action 1.2: Fix Fact Cloning in lib.rs  
```rust
// Current problematic code in lib.rs:119
self.fact_memory.extend(facts.clone()); // ❌ DOUBLES MEMORY

// Fix: Store references or move facts
self.fact_memory.extend(facts.iter().cloned()); // Temporary fix
// Better: Implement FactStore trait (see Action 1.4)
```

#### Action 1.3: Add RSS Memory Tracking
Create `crates/bingo-core/src/memory.rs`:
```rust
use std::fs;

pub fn get_memory_usage() -> anyhow::Result<usize> {
    #[cfg(target_os = "linux")]
    {
        let status = fs::read_to_string("/proc/self/status")?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: usize = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                return Ok(kb * 1024); // Convert KB to bytes
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Use task_info for macOS
        Ok(0) // Placeholder - implement with mach API
    }
    
    Ok(0)
}
```

#### Action 1.4: Implement FactStore Abstraction
Create `crates/bingo-core/src/fact_store.rs`:
```rust
use crate::types::{Fact, FactId};

pub trait FactStore {
    fn insert(&mut self, fact: Fact) -> FactId;
    fn get(&self, id: FactId) -> Option<&Fact>;
    fn extend<I: IntoIterator<Item = Fact>>(&mut self, facts: I);
    fn len(&self) -> usize;
    fn clear(&mut self);
}

pub struct VecFactStore {
    facts: Vec<Fact>,
}

impl VecFactStore {
    pub fn new() -> Self {
        Self { facts: Vec::new() }
    }
}

impl FactStore for VecFactStore {
    fn insert(&mut self, fact: Fact) -> FactId {
        let id = self.facts.len() as FactId;
        self.facts.push(fact);
        id
    }
    
    fn get(&self, id: FactId) -> Option<&Fact> {
        self.facts.get(id as usize)
    }
    
    fn extend<I: IntoIterator<Item = Fact>>(&mut self, facts: I) {
        self.facts.extend(facts);
    }
    
    fn len(&self) -> usize {
        self.facts.len()
    }
    
    fn clear(&mut self) {
        self.facts.clear();
    }
}
```

### Day 3-4: Performance Baseline Implementation

#### Action 2.1: Add Criterion Benchmarks
Create `crates/bingo-core/benches/engine_bench.rs`:
```rust
use bingo_core::{Engine, Fact, FactData, FactValue};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;

fn generate_test_facts(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| Fact {
            id: i as u64,
            data: FactData {
                fields: {
                    let mut map = HashMap::new();
                    map.insert("entity_id".to_string(), FactValue::Integer(i as i64));
                    map.insert("value".to_string(), FactValue::Float(i as f64 * 1.5));
                    map.insert("status".to_string(), FactValue::String("active".to_string()));
                    map
                }
            }
        })
        .collect()
}

fn bench_fact_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("fact_processing");
    
    for size in [1000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_facts", size),
            size,
            |b, &size| {
                let facts = generate_test_facts(size);
                let mut engine = Engine::new();
                
                b.iter(|| {
                    engine.process_facts(black_box(facts.clone()))
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_fact_processing);
criterion_main!(benches);
```

#### Action 2.2: Add Memory Tracking to Benchmarks
Enhance benchmark with RSS tracking:
```rust
use bingo_core::memory::get_memory_usage;

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    for size in [10_000, 100_000].iter() {
        group.bench_function(
            BenchmarkId::new("memory_rss", size),
            |b| {
                let facts = generate_test_facts(*size);
                let mut engine = Engine::new();
                
                b.iter_custom(|iters| {
                    let start_memory = get_memory_usage().unwrap_or(0);
                    let start = std::time::Instant::now();
                    
                    for _ in 0..iters {
                        engine.process_facts(black_box(facts.clone())).unwrap();
                    }
                    
                    let elapsed = start.elapsed();
                    let end_memory = get_memory_usage().unwrap_or(0);
                    
                    println!("Memory delta: {} bytes", end_memory.saturating_sub(start_memory));
                    elapsed
                });
            },
        );
    }
    group.finish();
}
```

#### Action 2.3: Add Cargo.toml Benchmark Configuration
Update `crates/bingo-core/Cargo.toml`:
```toml
[[bench]]
name = "engine_bench"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### Day 5: CI Integration and Deterministic Node IDs

#### Action 3.1: Add Performance CI Job
Create `.github/workflows/performance.yml`:
```yaml
name: Performance Benchmarks

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Install flamegraph
        run: cargo install flamegraph
        
      - name: Run benchmarks
        run: |
          cd crates/bingo-core
          cargo bench --bench engine_bench -- --output-format html
          
      - name: Generate flamegraph
        run: |
          cd crates/bingo-core
          sudo -E cargo flamegraph --bench engine_bench --output flamegraph.svg
          
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: |
            crates/bingo-core/target/criterion/
            crates/bingo-core/flamegraph.svg
```

#### Action 3.2: Add Deterministic Node IDs
Update `crates/bingo-rete/src/lib.rs`:
```rust
pub struct NodeIdGenerator {
    counter: u64,
    prefix: String,
}

impl NodeIdGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            counter: 0,
            prefix: prefix.to_string(),
        }
    }
    
    pub fn next_alpha_id(&mut self) -> NodeId {
        self.counter += 1;
        format!("{}:alpha:{:04}", self.prefix, self.counter)
    }
    
    pub fn next_beta_id(&mut self) -> NodeId {
        self.counter += 1;
        format!("{}:beta:{:04}", self.prefix, self.counter)
    }
    
    pub fn next_terminal_id(&mut self) -> NodeId {
        self.counter += 1;
        format!("{}:terminal:{:04}", self.prefix, self.counter)
    }
}
```

## Success Criteria for Week 1

### Must Have ✅
- [ ] Fix fact cloning in engine.rs and lib.rs (no memory doubling)
- [ ] RSS memory tracking implemented and tested
- [ ] 100K fact benchmark running in <1 second
- [ ] Memory usage <50MB for 100K facts (measured)
- [ ] FactStore trait with Vec implementation
- [ ] Deterministic node IDs for all RETE nodes

### Should Have 🎯
- [ ] Flamegraph profiling integrated in CI
- [ ] Performance regression detection (>10% degradation alerts)
- [ ] Basic hash indexing for fact lookup
- [ ] Criterion benchmarks with memory tracking

### Could Have 💡
- [ ] Arena allocator behind feature flag
- [ ] Multiple benchmark scenarios
- [ ] Performance comparison dashboard

## Week 2+ Planning

### Week 2: Hash Indexing and Optimization
- Implement hash-based fact lookup
- Add node sharing for identical conditions
- Optimize token propagation

### Week 3: FactStore Arena Implementation
- Add bumpalo-based arena allocator
- Benchmark Vec vs Arena performance
- Implement memory compaction

### Week 4: Performance Validation
- Validate 1M fact processing
- Memory profiling and optimization
- Prepare for Phase 2 features

## Risk Mitigation

### If Week 1 Targets Not Met
1. **Performance < 100K facts/second**: Focus on profiling bottlenecks
2. **Memory > 50MB for 100K facts**: Investigate memory leaks and allocations
3. **CI integration issues**: Simplify to local benchmarks first

### Escalation Path
- **Day 3**: If memory fixes don't show improvement, escalate to team lead
- **Day 5**: If benchmarks show <50% of target performance, reassess approach
- **End of Week 1**: If core issues unresolved, extend Week 1 scope into Week 2

## Measurement and Reporting

### Daily Tracking
- Memory usage for 10K, 50K, 100K facts
- Processing time for benchmark suite
- CI benchmark run status

### Weekly Review Metrics
- Baseline performance established: Yes/No
- Memory issues resolved: Yes/No  
- Benchmark automation working: Yes/No
- Performance regression detection: Yes/No

This plan addresses the critical specification-implementation gap identified in the analysis and establishes the foundation for all future development phases.