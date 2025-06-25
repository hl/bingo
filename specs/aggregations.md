# Aggregation Framework Specification

## Overview

Bingo provides first-class support for aggregations that integrate seamlessly with the RETE network, offering incremental processing capabilities that scale efficiently with large datasets. This enables complex multi-phase rule processing for various analytical use cases.

## Example Use Case: Multi-Phase Processing

### Phase 1: Individual Processing
Process each fact to enrich with calculated fields:
```json
{
  "rule_id": "enrich_facts",
  "conditions": [
    {"field": "category", "operator": "Equal", "value": "TypeA"},
    {"field": "score", "operator": "GreaterThan", "value": 75}
  ],
  "actions": [
    {"SetField": {"field": "rate", "value": 1.5}}
  ]
}
```

### Phase 2: Multi-Fact Aggregation
Aggregate related facts and create derived facts:
```json
{
  "rule_id": "aggregate_by_group",
  "conditions": [
    {
      "aggregation": {
        "type": "Sum",
        "field": "value",
        "group_by": ["entity_id", "period"],
        "having": {"operator": "GreaterThan", "value": {"Float": 100.0}},
        "alias": "total_value"
      }
    }
  ],
  "actions": [
    {
      "CreateFact": {
        "fact_type": "summary",
        "fields": {
          "entity_id": {"Reference": "entity_id"},
          "period": {"Reference": "period"},
          "calculated_total": {"Reference": "total_value"},
          "derived_metric": {"Formula": "total_value * rate"}
        }
      }
    }
  ]
}
```

## Aggregation Node Architecture

### Node Types
```rust
pub enum ReteNode {
    Alpha(AlphaNode),
    Beta(BetaNode),
    Aggregation(AggregationNode),  // ← New aggregation support
    Terminal(TerminalNode),
}

pub struct AggregationNode {
    pub node_id: NodeId,
    pub aggregation_spec: AggregationSpec,
    pub state: AggregationState,
    pub successors: Vec<NodeId>,
}
```

### Aggregation Specification
```rust
pub struct AggregationSpec {
    pub aggregation_type: AggregationType,
    pub source_field: String,
    pub group_by: Vec<String>,
    pub having_condition: Option<Condition>,
    pub window: Option<AggregationWindow>,
    pub alias: String,
}

pub enum AggregationType {
    Sum,
    Count,
    Average,
    Min,
    Max,
    StandardDeviation,
    Percentile(f64),
    Custom(Box<dyn CustomAggregation>),
}

pub enum AggregationWindow {
    Sliding { size: usize },
    Tumbling { size: usize },
    Session { timeout_ms: u64 },
    Time { duration_ms: u64 },
}
```

## Incremental Processing

### State Management
```rust
pub struct AggregationState {
    groups: HashMap<GroupKey, GroupState>,
    total_facts_processed: usize,
    last_updated: Instant,
}

pub struct GroupState {
    current_value: AggregationValue,
    fact_count: usize,
    contributing_facts: HashSet<FactId>,
    last_modified: Instant,
}

pub enum AggregationValue {
    Integer(i64),
    Float(f64),
    Distribution(StatisticalDistribution),
}
```

### Incremental Updates
```rust
impl AggregationNode {
    pub fn process_fact(&mut self, fact: &Fact) -> Vec<Token> {
        let group_key = self.extract_group_key(fact);
        let old_value = self.state.get_group_value(&group_key);
        
        // Update aggregation incrementally
        let new_value = self.update_aggregation(&group_key, fact);
        
        // Check if having condition is now satisfied
        if self.having_condition_met(&new_value) {
            self.propagate_aggregation_result(group_key, new_value)
        } else {
            Vec::new()
        }
    }
    
    fn update_aggregation(&mut self, group_key: &GroupKey, fact: &Fact) -> AggregationValue {
        match self.aggregation_spec.aggregation_type {
            AggregationType::Sum => self.update_sum(group_key, fact),
            AggregationType::Count => self.update_count(group_key, fact),
            AggregationType::Average => self.update_average(group_key, fact),
            // ... other aggregation types
        }
    }
}
```

## Multi-Phase Processing

### Execution Phases
```rust
pub struct ExecutionPhase {
    pub phase_id: usize,
    pub rules: Vec<RuleId>,
    pub depends_on: Vec<usize>,
    pub aggregation_required: bool,
}

pub struct MultiPhaseEngine {
    phases: Vec<ExecutionPhase>,
    current_phase: usize,
    intermediate_results: HashMap<usize, Vec<Fact>>,
}

impl MultiPhaseEngine {
    pub async fn execute_phases(&mut self, initial_facts: Vec<Fact>) -> Result<Vec<Fact>> {
        let mut current_facts = initial_facts;
        
        for phase in &self.phases {
            tracing::info!(phase_id = phase.phase_id, "Starting execution phase");
            
            // Execute rules in current phase
            let phase_results = self.execute_phase(phase, &current_facts).await?;
            
            // If this phase produces aggregations, wait for completion
            if phase.aggregation_required {
                self.finalize_aggregations(phase.phase_id).await?;
            }
            
            // Combine results for next phase
            current_facts.extend(phase_results);
            
            tracing::info!(
                phase_id = phase.phase_id,
                fact_count = current_facts.len(),
                "Phase completed"
            );
        }
        
        Ok(current_facts)
    }
}
```

## Performance Optimizations

### Memory Management
```rust
pub struct AggregationArena {
    group_states: SlotMap<GroupId, GroupState>,
    value_pool: Pool<AggregationValue>,
    fact_references: CompactMap<FactId, GroupId>,
    memory_limit: usize,
}

impl AggregationArena {
    pub fn compact(&mut self) {
        // Remove expired groups
        self.group_states.retain(|_, state| !state.is_expired());
        
        // Compact value storage
        self.value_pool.compact();
        
        // Update memory tracking
        self.update_memory_usage();
    }
}
```

### Parallel Aggregation
```rust
pub struct PartitionedAggregation {
    partitions: Vec<AggregationNode>,
    merger: AggregationMerger,
    partition_strategy: PartitionStrategy,
}

impl PartitionedAggregation {
    pub async fn process_facts_parallel(&mut self, facts: Vec<Fact>) -> Result<Vec<Token>> {
        // Partition facts by group key for parallel processing
        let partitioned_facts = self.partition_facts(facts);
        
        // Process each partition in parallel
        let partition_results = futures::future::join_all(
            partitioned_facts.into_iter().enumerate().map(|(i, facts)| {
                self.partitions[i].process_facts_batch(facts)
            })
        ).await;
        
        // Merge results from all partitions
        self.merger.merge_results(partition_results)
    }
}
```

## Example: Generic Multi-Phase Processing

### Input Facts
```json
[
  {
    "id": 1,
    "data": {
      "fields": {
        "entity_id": {"Integer": 12345},
        "category": {"String": "TypeA"},
        "score": {"Integer": 85},
        "period": {"String": "2024-Q1"},
        "value": {"Float": 42.5}
      }
    }
  }
]
```

### Phase 1 Output (After Enrichment)
```json
[
  {
    "id": 1,
    "data": {
      "fields": {
        "entity_id": {"Integer": 12345},
        "category": {"String": "TypeA"},
        "score": {"Integer": 85},
        "period": {"String": "2024-Q1"},
        "value": {"Float": 42.5},
        "rate": {"Float": 1.5}
      }
    }
  }
]
```

### Phase 2 Output (After Aggregation)
```json
[
  {
    "id": 1001,
    "data": {
      "fields": {
        "fact_type": {"String": "summary"},
        "entity_id": {"Integer": 12345},
        "period": {"String": "2024-Q1"},
        "calculated_total": {"Float": 127.5},
        "derived_metric": {"Float": 191.25}
      }
    }
  }
]
```

## Performance Characteristics

### Scalability
- **Group Cardinality**: Efficiently handles high-cardinality groupings (millions of employees)
- **Incremental Updates**: O(log n) per fact for most aggregations
- **Memory Usage**: Bounded memory growth with configurable eviction policies
- **Parallel Processing**: Horizontal scaling across partitions

### Benchmarks
- **Target**: 3M facts → multi-phase processing in <5 seconds
- **Memory**: <100MB additional for aggregation state (3M facts)
- **Throughput**: >500K facts/second through aggregation nodes
- **Latency**: <1ms per aggregation update