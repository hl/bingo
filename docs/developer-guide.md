# Bingo RETE Rules Engine - Developer Guide

This comprehensive guide provides everything developers need to understand, develop, and extend the Bingo RETE Rules Engine, including the advanced RETE algorithm implementation with rule optimization, parallel processing, conflict resolution, and dependency analysis using Kahn's topological sorting algorithm.

## 📋 Table of Contents

- [Development Environment Setup](#development-environment-setup)
- [Code Architecture & Organization](#code-architecture--organization)
- [Development Workflow](#development-workflow)
- [Testing Strategy](#testing-strategy)
- [Code Quality Standards](#code-quality-standards)
- [Performance Optimization](#performance-optimization)
- [Debugging & Troubleshooting](#debugging--troubleshooting)
- [Contributing Guidelines](#contributing-guidelines)
- [Deployment & Operations](#deployment--operations)

---

## Development Environment Setup

### Prerequisites

#### Required Software
- **Rust 1.88.0+** with 2024 edition support
- **Cargo** (included with Rust)
- **Git** for version control
- **Protocol Buffers Compiler** (`protoc`) for gRPC development

#### Installation Commands
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required Rust components
rustup component add rustfmt clippy

# Install Protocol Buffers (macOS)
brew install protobuf

# Install Protocol Buffers (Ubuntu/Debian)
sudo apt install protobuf-compiler

# Verify installation
cargo --version
rustc --version
protoc --version
```

#### Development Tools (Recommended)
```bash
# Enhanced development tools
cargo install cargo-watch     # Auto-rebuild on file changes
cargo install cargo-audit     # Security vulnerability scanning
cargo install cargo-outdated  # Dependency update checking
cargo install cargo-expand    # Macro expansion debugging
```

### Project Setup

#### Repository Clone and Build
```bash
# Clone the repository
git clone <repository-url>
cd bingo

# Build in development mode
cargo build

# Build optimized release version
cargo build --release

# Run full test suite
cargo test --workspace

# Start development server
cargo run --bin bingo
```

#### IDE Configuration

##### VS Code Setup
Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

Create `.vscode/tasks.json`:
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo check",
            "type": "shell",
            "command": "cargo",
            "args": ["check", "--workspace", "--all-targets"],
            "group": "build"
        },
        {
            "label": "cargo test",
            "type": "shell",
            "command": "cargo",
            "args": ["test", "--workspace"],
            "group": "test"
        },
        {
            "label": "quality check",
            "type": "shell",
            "command": "cargo",
            "args": ["fmt", "--check", "&&", "cargo", "clippy", "--", "-D", "warnings"],
            "group": "build"
        }
    ]
}
```

##### IntelliJ IDEA / CLion Setup
- Install the **Rust** plugin
- Configure Rust toolchain: Settings → Languages & Frameworks → Rust
- Enable Clippy integration: Settings → Tools → Rust → External Linters
- Set up run configurations for tests and benchmarks

---

## Code Architecture & Organization

### Workspace Structure
```
bingo/
├── Cargo.toml                    # Workspace configuration
├── README.md                     # Project overview
├── docs/                         # Documentation
├── specs/                        # Technical specifications
├── crates/
│   ├── bingo-types/             # Shared type definitions
│   │   ├── src/
│   │   │   └── lib.rs           # Core data types
│   │   └── Cargo.toml
│   ├── bingo-calculator/        # Calculator plugin system
│   │   ├── src/
│   │   │   ├── lib.rs           # Calculator interface
│   │   │   ├── plugin.rs        # Plugin trait definition
│   │   │   ├── plugin_manager.rs # Plugin management
│   │   │   └── built_in/        # Built-in calculators
│   │   │       ├── add.rs
│   │   │       ├── multiply.rs
│   │   │       └── ...
│   │   └── Cargo.toml
│   ├── bingo-core/              # Core RETE engine
│   │   ├── src/
│   │   │   ├── lib.rs           # Public API
│   │   │   ├── engine.rs        # Main engine implementation
│   │   │   ├── types.rs         # Engine-specific types
│   │   │   ├── error.rs         # Error handling
│   │   │   ├── profiler.rs      # Performance monitoring
│   │   │   ├── rete_network.rs  # RETE algorithm
│   │   │   ├── rete_nodes.rs    # Network node types
│   │   │   └── fact_store/      # Fact storage
│   │   │       ├── mod.rs
│   │   │       └── arena_store.rs
│   │   ├── tests/               # Integration tests
│   │   └── Cargo.toml
│   ├── bingo-api/               # gRPC API server
│   │   ├── src/
│   │   │   ├── lib.rs           # API library
│   │   │   ├── main.rs          # Server binary
│   │   │   ├── grpc/            # gRPC service implementation
│   │   │   └── generated/       # Protocol buffer generated code
│   │   ├── proto/               # Protocol buffer definitions
│   │   └── Cargo.toml
│   └── bingo-web/               # Web interface (optional)
└── target/                      # Build outputs
```

### Module Design Principles

#### 1. Separation of Concerns
Each crate has a single, well-defined responsibility:
- **bingo-types**: Shared data structures and serialization
- **bingo-calculator**: Business logic and extensibility
- **bingo-core**: RETE algorithm and rule processing
- **bingo-api**: External interface and protocol handling

#### 2. Dependency Management
```rust
// Dependency hierarchy (top to bottom)
bingo-api     ──┐
                ├─── bingo-core ──── bingo-calculator ──── bingo-types
bingo-web     ──┘
```

**Rules:**
- No circular dependencies between crates
- Shared types live in `bingo-types`
- Core business logic stays in `bingo-core`
- External interfaces in dedicated crates

#### 3. Error Handling Strategy
```rust
// Centralized error types with context
pub enum BingoError {
    Rule { message: String, rule_id: Option<u64>, /* ... */ },
    Calculator { message: String, calculator_name: String, /* ... */ },
    // ... other variants
}

// Result type alias for consistency
pub type BingoResult<T> = Result<T, BingoError>;

// Context enhancement trait
pub trait ResultExt<T> {
    fn with_context<F>(self, f: F) -> BingoResult<T> where F: FnOnce() -> String;
}
```

### Key Architectural Patterns

#### 1. RETE Network Implementation
```rust
// Alpha Memory: Fact-to-Rule indexing
pub struct AlphaMemory {
    field_indexes: HashMap<String, HashMap<FactValue, Vec<RuleId>>>,
    universal_rules: Vec<RuleId>,
}

// Beta Memory: Multi-fact pattern matching
pub struct BetaMemory {
    partial_matches: HashMap<u64, PartialMatch>,
    join_results: Vec<JoinResult>,
}

// Network compilation and optimization
impl ReteNetwork {
    pub fn compile_rule(&mut self, rule: &Rule) -> BingoResult<()> {
        // Convert rule conditions to network nodes
        // Optimize for common patterns
        // Index for efficient matching
    }
}
```

#### 2. Plugin Architecture
```rust
// Calculator plugin trait
pub trait CalculatorPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult;
}

// Plugin manager with registration
pub struct Calculator {
    plugins: HashMap<String, Box<dyn CalculatorPlugin>>,
}

impl Calculator {
    pub fn register_plugin(&mut self, plugin: Box<dyn CalculatorPlugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }
}
```

#### 3. Arena-based Memory Management
```rust
// Efficient fact storage with arena allocation
pub struct ArenaFactStore {
    facts: Arena<Fact>,
    indices: HashMap<String, HashMap<FactValue, Vec<FactId>>>,
    id_counter: AtomicU64,
}
```

---

## Development Workflow

### Daily Development Process

#### 1. Pre-development Setup
```bash
# Start development session
git pull origin main
cargo check --workspace
cargo test --workspace --lib  # Quick test run
```

#### 2. Feature Development Cycle
```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Development with auto-rebuild
cargo watch -x "check --workspace" -x "test --workspace --lib"

# Before committing - full quality check
cargo fmt --check && \
cargo clippy --workspace --all-targets -- -D warnings && \
cargo check --workspace --all-targets && \
cargo test --workspace
```

#### 3. Commit and Integration
```bash
# Commit with descriptive message
git add .
git commit -m "feat: add new calculator for compound interest

- Implement CompoundInterestCalculator plugin
- Add comprehensive unit tests
- Update calculator documentation"

# Push and create pull request
git push origin feature/your-feature-name
```

### Code Organization Guidelines

#### File Naming Conventions
- **Modules**: `snake_case.rs` (e.g., `rete_network.rs`)
- **Types**: `PascalCase` structs and enums
- **Functions**: `snake_case` functions and methods
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Test files**: `*_test.rs` or `tests/` directory

#### Module Structure Template
```rust
//! Module documentation
//!
//! Brief description of module purpose and key concepts.

// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// External crate imports
use anyhow::Result;
use serde::{Deserialize, Serialize};

// Internal crate imports
use crate::types::{FactId, FactValue};
use crate::error::{BingoError, BingoResult};

// Public types and constants
pub const DEFAULT_CAPACITY: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStruct {
    // Public fields
    pub id: u64,
    
    // Private fields
    inner: Arc<InnerData>,
}

// Implementation blocks
impl ModuleStruct {
    /// Public constructor with documentation
    pub fn new(id: u64) -> Self {
        // Implementation
    }
    
    /// Public method with examples
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let instance = ModuleStruct::new(1);
    /// let result = instance.process_data(data)?;
    /// ```
    pub fn process_data(&self, data: &[u8]) -> BingoResult<Vec<u8>> {
        // Implementation
    }
    
    // Private helper methods
    fn internal_helper(&self) -> bool {
        // Implementation
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constructor() {
        let instance = ModuleStruct::new(42);
        assert_eq!(instance.id, 42);
    }
    
    #[test]
    fn test_process_data() {
        // Test implementation
    }
}
```

### Version Control Best Practices

#### Branch Strategy
- **main**: Production-ready code
- **feature/**: New features (`feature/calculator-extensions`)
- **fix/**: Bug fixes (`fix/memory-leak-in-network`)
- **docs/**: Documentation updates (`docs/api-reference-update`)
- **perf/**: Performance improvements (`perf/optimize-fact-indexing`)

#### Commit Message Format
```
<type>(<scope>): <description>

<optional body>

<optional footer>
```

**Types:**
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `perf`: Performance improvements
- `refactor`: Code refactoring
- `test`: Test additions/modifications
- `style`: Code style changes

**Examples:**
```
feat(calculator): add weighted average calculator

Implement weighted average calculation for complex aggregations:
- Support for arrays of value/weight pairs
- Proper error handling for invalid inputs
- Comprehensive test coverage

Closes #123
```

---

## Testing Strategy

### Test Categories

#### 1. Unit Tests
Location: `src/` files with `#[cfg(test)]` modules

**Coverage Requirements:**
- All public functions must have tests
- All error conditions must be tested
- Edge cases and boundary conditions
- Performance regression tests

**Example Unit Test:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rule_compilation_success() {
        let mut engine = BingoEngine::new().unwrap();
        let rule = create_test_rule();
        
        let result = engine.add_rule(rule);
        
        assert!(result.is_ok());
        assert_eq!(engine.rule_count(), 1);
    }
    
    #[test]
    fn test_rule_compilation_error() {
        let mut engine = BingoEngine::new().unwrap();
        let invalid_rule = create_invalid_rule();
        
        let result = engine.add_rule(invalid_rule);
        
        assert!(result.is_err());
        if let Err(BingoError::Rule { message, .. }) = result {
            assert!(message.contains("Invalid condition"));
        }
    }
    
    #[test]
    fn test_performance_regression() {
        let mut engine = BingoEngine::new().unwrap();
        let rules = create_complex_rule_set(100);
        let facts = create_test_facts(1000);
        
        let start = std::time::Instant::now();
        let results = engine.process_facts(facts).unwrap();
        let duration = start.elapsed();
        
        assert!(duration < std::time::Duration::from_millis(100));
        assert_eq!(results.len(), 1000); // Expected result count
    }
}
```

#### 2. Integration Tests
Location: `tests/` directory

**Purpose:**
- End-to-end workflow testing
- Multi-component interaction
- API contract validation
- Performance benchmarking

**Example Integration Test:**
```rust
// tests/end_to_end_payroll_test.rs
use bingo_core::{BingoEngine, Rule, Fact};

#[test]
fn test_complete_payroll_workflow() {
    let mut engine = BingoEngine::new().unwrap();
    
    // Add payroll rules
    add_overtime_rules(&mut engine);
    add_tax_calculation_rules(&mut engine);
    add_deduction_rules(&mut engine);
    
    // Process employee timesheets
    let timesheets = create_employee_timesheets();
    let results = engine.process_facts(timesheets).unwrap();
    
    // Verify payroll calculations
    verify_overtime_calculations(&results);
    verify_tax_calculations(&results);
    verify_net_pay_calculations(&results);
}
```

#### 3. Performance Tests
Location: `tests/performance/` or marked with `#[ignore]`

**Execution:**
```bash
# Run performance tests
cargo test --release -- --ignored

# Run specific performance test
cargo test --release test_million_facts_processing -- --ignored --nocapture
```

**Example Performance Test:**
```rust
#[test]
#[ignore] // Run only with --ignored flag
fn test_large_scale_fact_processing() {
    let mut engine = BingoEngine::with_capacity(100_000).unwrap();
    
    // Add representative rule set
    let rules = create_realistic_rule_set(200);
    for rule in rules {
        engine.add_rule(rule).unwrap();
    }
    
    // Generate large fact set
    let facts = generate_test_facts(100_000);
    
    let start = std::time::Instant::now();
    let results = engine.process_facts(facts).unwrap();
    let duration = start.elapsed();
    
    let facts_per_second = 100_000.0 / duration.as_secs_f64();
    
    println!("Processed 100k facts in {:?}", duration);
    println!("Throughput: {:.0} facts/second", facts_per_second);
    
    // Performance assertions
    assert!(facts_per_second > 10_000.0, "Throughput below minimum threshold");
    assert!(duration < std::time::Duration::from_secs(30), "Processing took too long");
}
```

### Testing Tools and Utilities

#### Test Helpers
```rust
// tests/common/mod.rs - Shared test utilities
use bingo_core::types::{Rule, Condition, Action, Fact, FactData, FactValue};

pub fn create_test_engine() -> BingoEngine {
    BingoEngine::new().expect("Failed to create test engine")
}

pub fn create_simple_rule(id: u64, field: &str, value: f64) -> Rule {
    Rule {
        id,
        name: format!("Test Rule {}", id),
        conditions: vec![
            Condition::Simple {
                field: field.to_string(),
                operator: Operator::GreaterThan,
                value: FactValue::Float(value),
            }
        ],
        actions: vec![
            Action {
                action_type: ActionType::SetField {
                    field: "processed".to_string(),
                    value: FactValue::Boolean(true),
                },
            }
        ],
    }
}

pub fn create_test_fact(id: u64, fields: Vec<(&str, FactValue)>) -> Fact {
    let mut field_map = HashMap::new();
    for (key, value) in fields {
        field_map.insert(key.to_string(), value);
    }
    
    Fact::new(id, FactData { fields: field_map })
}
```

#### Mock Objects
```rust
// For testing external dependencies
pub struct MockCalculator {
    pub name: String,
    pub responses: HashMap<String, FactValue>,
}

impl CalculatorPlugin for MockCalculator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn calculate(&self, args: &HashMap<String, &FactValue>) -> CalculationResult {
        let key = format!("{:?}", args);
        if let Some(response) = self.responses.get(&key) {
            Ok(response.clone())
        } else {
            Err("Mock response not configured".to_string())
        }
    }
}
```

### Test Execution and CI/CD

#### Local Testing
```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_rule_compilation

# Run tests in parallel
cargo test --workspace --jobs $(nproc)

# Run with environment variables
RUST_LOG=debug cargo test test_debug_scenario -- --nocapture
```

#### Continuous Integration
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta, nightly]
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
        override: true
    
    - name: Cache dependencies
      uses: actions/cache@v2
      with:
        path: ~/.cargo
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Format check
      run: cargo fmt --all -- --check
    
    - name: Clippy check
      run: cargo clippy --workspace --all-targets -- -D warnings
    
    - name: Build
      run: cargo build --workspace --all-targets
    
    - name: Test
      run: cargo test --workspace
    
    - name: Performance tests
      run: cargo test --workspace --release -- --ignored
```

---

## Code Quality Standards

### Formatting and Style

#### Rust Format Configuration
Create `.rustfmt.toml`:
```toml
# Formatting configuration
edition = "2024"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

#### Clippy Configuration
Create `.clippy.toml`:
```toml
# Clippy linting configuration
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 8
type-complexity-threshold = 250
single-char-lifetime-names = false
```

#### Code Style Guidelines

**Documentation Standards:**
```rust
//! Module-level documentation
//!
//! Provides comprehensive description of module purpose,
//! key concepts, and usage examples.

/// Function-level documentation with examples
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value and possible error conditions
///
/// # Examples
///
/// ```rust
/// use your_crate::YourStruct;
///
/// let instance = YourStruct::new();
/// let result = instance.method(42)?;
/// assert_eq!(result, expected_value);
/// ```
///
/// # Errors
///
/// This function returns an error when:
/// - Invalid input parameters
/// - Resource allocation failures
pub fn your_function(param1: u32, param2: &str) -> BingoResult<String> {
    // Implementation
}
```

**Error Handling Patterns:**
```rust
// Prefer ? operator for error propagation
pub fn process_data(input: &[u8]) -> BingoResult<ProcessedData> {
    let validated = validate_input(input)?;
    let processed = transform_data(validated)?;
    let result = finalize_processing(processed)?;
    Ok(result)
}

// Use context for error enhancement
pub fn complex_operation() -> BingoResult<()> {
    perform_step_one()
        .with_context(|| "Failed during step one execution")?;
    
    perform_step_two()
        .with_context(|| "Failed during step two execution")?;
    
    Ok(())
}

// Prefer early returns for error conditions
pub fn validate_rule(rule: &Rule) -> BingoResult<()> {
    if rule.conditions.is_empty() {
        return Err(BingoError::rule("Rule must have at least one condition"));
    }
    
    if rule.actions.is_empty() {
        return Err(BingoError::rule("Rule must have at least one action"));
    }
    
    // Continue validation...
    Ok(())
}
```

### Performance Guidelines

#### Memory Management
```rust
// Prefer references over cloning
pub fn process_facts(facts: &[Fact]) -> BingoResult<Vec<RuleExecutionResult>> {
    let mut results = Vec::with_capacity(facts.len());
    
    for fact in facts {
        let fact_results = self.process_single_fact(fact)?;
        results.extend(fact_results);
    }
    
    Ok(results)
}

// Use arena allocation for temporary objects
pub struct TemporaryArena<T> {
    items: Vec<T>,
}

impl<T> TemporaryArena<T> {
    pub fn allocate(&mut self, item: T) -> &T {
        self.items.push(item);
        self.items.last().unwrap()
    }
    
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

// Implement Drop for cleanup
impl Drop for ExpensiveResource {
    fn drop(&mut self) {
        self.cleanup_resources();
    }
}
```

#### Algorithm Optimization
```rust
// Use appropriate data structures
use std::collections::{HashMap, BTreeMap, HashSet};

// HashMap for O(1) lookups
pub struct FastLookup {
    index: HashMap<String, Vec<RuleId>>,
}

// BTreeMap for sorted iteration
pub struct OrderedData {
    sorted_facts: BTreeMap<u64, Fact>,
}

// HashSet for existence checks
pub struct UniqueTracker {
    seen_ids: HashSet<FactId>,
}

// Optimize hot paths with inline hints
#[inline]
pub fn frequently_called_function(&self, param: u32) -> u32 {
    // Hot path implementation
    param * 2
}

#[inline(never)]
pub fn rarely_called_function(&self) {
    // Cold path implementation
}
```

### Security Guidelines

#### Input Validation
```rust
pub fn validate_fact_data(data: &FactData) -> BingoResult<()> {
    // Size limits
    if data.fields.len() > MAX_FIELD_COUNT {
        return Err(BingoError::validation(
            "Too many fields in fact data",
            Some("field_count"),
            Some(format!("maximum {}", MAX_FIELD_COUNT)),
        ));
    }
    
    // Content validation
    for (key, value) in &data.fields {
        validate_field_name(key)?;
        validate_field_value(value)?;
    }
    
    Ok(())
}

fn validate_field_name(name: &str) -> BingoResult<()> {
    if name.is_empty() {
        return Err(BingoError::validation("Field name cannot be empty"));
    }
    
    if name.len() > MAX_FIELD_NAME_LENGTH {
        return Err(BingoError::validation("Field name too long"));
    }
    
    // Prevent injection attacks
    if name.contains(['<', '>', '"', '\'', '&']) {
        return Err(BingoError::validation("Invalid characters in field name"));
    }
    
    Ok(())
}
```

#### Resource Management
```rust
pub struct ResourceLimiter {
    max_memory_bytes: usize,
    max_processing_time: Duration,
    current_usage: AtomicUsize,
}

impl ResourceLimiter {
    pub fn check_memory_limit(&self, additional: usize) -> BingoResult<()> {
        let current = self.current_usage.load(Ordering::Relaxed);
        if current + additional > self.max_memory_bytes {
            return Err(BingoError::memory(
                "Memory limit exceeded",
                "allocation_check",
                Some(current + additional),
                Some(self.max_memory_bytes),
            ));
        }
        Ok(())
    }
    
    pub fn with_timeout<T, F>(&self, operation: F) -> BingoResult<T>
    where
        F: FnOnce() -> BingoResult<T>,
    {
        let start = Instant::now();
        let result = operation()?;
        
        if start.elapsed() > self.max_processing_time {
            return Err(BingoError::performance(
                "Operation timeout exceeded",
                "timeout_check",
                Some(start.elapsed().as_millis() as u64),
            ));
        }
        
        Ok(result)
    }
}
```

---

## Performance Optimization

### Profiling and Measurement

#### Built-in Profiling
```rust
use bingo_core::profiler::EngineProfiler;

let mut profiler = EngineProfiler::new();

// Time operations
let result = profiler.time_operation("rule_compilation", || {
    engine.add_rule(complex_rule)
});

// Manual timing
profiler.start_operation("custom_processing");
perform_custom_processing();
let duration = profiler.end_operation("custom_processing");

// Get comprehensive report
let report = profiler.get_performance_report();
for (operation, metrics) in report.operation_metrics {
    println!("{}: avg={:?}, p95={:?}", operation, metrics.average_time, metrics.p95_time);
}
```

#### External Profiling Tools
```bash
# CPU profiling with perf
sudo perf record -g cargo test performance_benchmark
sudo perf report

# Memory profiling with valgrind
valgrind --tool=massif cargo test memory_usage_test

# Flamegraph generation
cargo install flamegraph
cargo flamegraph --test performance_test
```

### Optimization Strategies

#### 1. Memory Optimization
```rust
// Object pooling for frequently allocated objects
pub struct ObjectPool<T> {
    pool: Vec<T>,
    create_fn: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_else(|| (self.create_fn)())
    }
    
    pub fn return_object(&mut self, obj: T) {
        if self.pool.len() < MAX_POOL_SIZE {
            self.pool.push(obj);
        }
    }
}

// Arena allocation for temporary objects
pub struct Arena<T> {
    chunks: Vec<Vec<T>>,
    current_chunk: usize,
    current_index: usize,
}

impl<T> Arena<T> {
    pub fn allocate(&mut self, value: T) -> &T {
        if self.current_index >= CHUNK_SIZE {
            self.chunks.push(Vec::with_capacity(CHUNK_SIZE));
            self.current_chunk += 1;
            self.current_index = 0;
        }
        
        let chunk = &mut self.chunks[self.current_chunk];
        chunk.push(value);
        self.current_index += 1;
        
        chunk.last().unwrap()
    }
}
```

#### 2. Algorithm Optimization
```rust
// Efficient indexing structures
pub struct OptimizedIndex {
    // Primary index for exact matches
    exact_matches: HashMap<FactValue, Vec<RuleId>>,
    
    // Range index for numeric comparisons
    numeric_ranges: BTreeMap<OrderedFloat<f64>, Vec<RuleId>>,
    
    // Prefix index for string patterns
    string_prefixes: HashMap<String, Vec<RuleId>>,
}

impl OptimizedIndex {
    pub fn find_matching_rules(&self, value: &FactValue) -> Vec<RuleId> {
        match value {
            FactValue::Float(f) => {
                let mut matches = Vec::new();
                
                // Exact matches
                if let Some(exact) = self.exact_matches.get(value) {
                    matches.extend(exact);
                }
                
                // Range matches
                let ordered_f = OrderedFloat(*f);
                for (_, rules) in self.numeric_ranges.range(..=ordered_f) {
                    matches.extend(rules);
                }
                
                matches
            }
            FactValue::String(s) => {
                let mut matches = Vec::new();
                
                // Exact matches
                if let Some(exact) = self.exact_matches.get(value) {
                    matches.extend(exact);
                }
                
                // Prefix matches
                for prefix_len in 1..=s.len() {
                    let prefix = &s[..prefix_len];
                    if let Some(prefix_rules) = self.string_prefixes.get(prefix) {
                        matches.extend(prefix_rules);
                    }
                }
                
                matches
            }
            _ => {
                self.exact_matches.get(value).cloned().unwrap_or_default()
            }
        }
    }
}
```

#### 3. Concurrent Processing
```rust
use rayon::prelude::*;

// Parallel fact processing
pub fn process_facts_parallel(&mut self, facts: Vec<Fact>) -> BingoResult<Vec<RuleExecutionResult>> {
    let chunk_size = (facts.len() / num_cpus::get()).max(1);
    
    let results: Result<Vec<_>, _> = facts
        .par_chunks(chunk_size)
        .map(|chunk| self.process_fact_chunk(chunk))
        .collect();
    
    let mut all_results = Vec::new();
    for chunk_results in results? {
        all_results.extend(chunk_results);
    }
    
    Ok(all_results)
}

// Lock-free data structures where possible
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ConcurrentCounter {
    value: AtomicU64,
}

impl ConcurrentCounter {
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
    
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}
```

### Performance Testing

#### Benchmark Framework
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_rule_compilation(c: &mut Criterion) {
    let mut engine = BingoEngine::new().unwrap();
    let rules = create_benchmark_rules(100);
    
    c.bench_function("rule_compilation_100_rules", |b| {
        b.iter(|| {
            let mut test_engine = BingoEngine::new().unwrap();
            for rule in black_box(&rules) {
                test_engine.add_rule(rule.clone()).unwrap();
            }
        })
    });
}

fn benchmark_fact_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("fact_processing");
    
    for fact_count in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_facts", fact_count),
            fact_count,
            |b, &fact_count| {
                let mut engine = setup_benchmark_engine();
                let facts = create_benchmark_facts(fact_count);
                
                b.iter(|| {
                    engine.process_facts(black_box(facts.clone())).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_rule_compilation, benchmark_fact_processing);
criterion_main!(benches);
```

---

## Debugging & Troubleshooting

### Logging and Observability

#### Logging Configuration
```rust
use tracing::{debug, info, warn, error, instrument};

// Structured logging with context
#[instrument(skip(self, facts))]
pub fn process_facts(&mut self, facts: Vec<Fact>) -> BingoResult<Vec<RuleExecutionResult>> {
    info!(fact_count = facts.len(), "Processing facts through engine");
    
    let mut results = Vec::new();
    
    for (index, fact) in facts.iter().enumerate() {
        debug!(fact_id = fact.id, index = index, "Processing individual fact");
        
        match self.process_single_fact(fact) {
            Ok(fact_results) => {
                debug!(result_count = fact_results.len(), "Fact processed successfully");
                results.extend(fact_results);
            }
            Err(e) => {
                warn!(
                    fact_id = fact.id,
                    error = %e,
                    "Failed to process fact"
                );
                return Err(e);
            }
        }
    }
    
    info!(total_results = results.len(), "Fact processing completed");
    Ok(results)
}

// Environment-based log level configuration
fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bingo=info".into())
        ))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}
```

#### Runtime Debugging
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Module-specific logging
RUST_LOG=bingo_core::rete_network=trace cargo run

# Structured logging with JSON output
RUST_LOG=info cargo run 2>&1 | jq '.'

# Performance debugging
RUST_LOG=bingo_core::profiler=debug cargo run
```

### Common Issues and Solutions

#### Memory Issues
```rust
// Memory leak detection
#[cfg(debug_assertions)]
pub fn check_memory_leaks(&self) {
    let current_usage = self.get_memory_usage();
    if current_usage > self.baseline_memory * 2 {
        warn!(
            current_mb = current_usage / 1024 / 1024,
            baseline_mb = self.baseline_memory / 1024 / 1024,
            "Potential memory leak detected"
        );
    }
}

// Memory usage reporting
pub fn report_memory_usage(&self) {
    let stats = self.get_detailed_memory_stats();
    info!(
        "Memory usage: facts={} MB, rules={} MB, network={} MB, total={} MB",
        stats.fact_store_mb,
        stats.rule_storage_mb,
        stats.rete_network_mb,
        stats.total_mb
    );
}
```

#### Performance Issues
```rust
// Performance bottleneck detection
pub fn analyze_performance_bottlenecks(&self) -> Vec<PerformanceIssue> {
    let report = self.profiler.get_performance_report();
    let mut issues = Vec::new();
    
    for (operation, metrics) in report.operation_metrics {
        if metrics.average_time > Duration::from_millis(100) {
            issues.push(PerformanceIssue {
                operation,
                issue_type: IssueType::SlowOperation,
                severity: if metrics.average_time > Duration::from_millis(500) {
                    Severity::High
                } else {
                    Severity::Medium
                },
                details: format!("Average time: {:?}", metrics.average_time),
            });
        }
        
        if metrics.total_calls > 10000 {
            issues.push(PerformanceIssue {
                operation,
                issue_type: IssueType::HighFrequency,
                severity: Severity::Medium,
                details: format!("Called {} times", metrics.total_calls),
            });
        }
    }
    
    issues
}
```

#### Rule Debugging
```rust
// Rule execution tracing
#[instrument(skip(self, rule, fact))]
fn execute_rule(&mut self, rule: &Rule, fact: &Fact) -> BingoResult<Vec<ActionResult>> {
    debug!(rule_id = rule.id, rule_name = %rule.name, "Executing rule");
    
    let mut action_results = Vec::new();
    
    for (action_index, action) in rule.actions.iter().enumerate() {
        debug!(
            action_index = action_index,
            action_type = ?action.action_type,
            "Executing action"
        );
        
        match self.execute_action(action, fact) {
            Ok(result) => {
                debug!(action_result = ?result, "Action executed successfully");
                action_results.push(result);
            }
            Err(e) => {
                error!(
                    rule_id = rule.id,
                    action_index = action_index,
                    error = %e,
                    "Action execution failed"
                );
                return Err(e);
            }
        }
    }
    
    Ok(action_results)
}

// Rule validation debugging
pub fn validate_rule_with_debug(&self, rule: &Rule) -> BingoResult<Vec<ValidationWarning>> {
    let mut warnings = Vec::new();
    
    // Check for common issues
    if rule.conditions.is_empty() {
        warnings.push(ValidationWarning::new(
            "Rule has no conditions and will match all facts",
            WarningLevel::Medium,
        ));
    }
    
    if rule.actions.is_empty() {
        warnings.push(ValidationWarning::new(
            "Rule has no actions and will have no effect",
            WarningLevel::High,
        ));
    }
    
    // Check for performance issues
    if rule.conditions.len() > 10 {
        warnings.push(ValidationWarning::new(
            "Rule has many conditions which may impact performance",
            WarningLevel::Low,
        ));
    }
    
    Ok(warnings)
}
```

### Debugging Tools

#### Custom Debug Commands
```rust
#[derive(Debug, clap::Parser)]
pub enum DebugCommand {
    /// Analyze rule performance
    AnalyzeRules {
        #[clap(long)]
        rule_id: Option<u64>,
    },
    
    /// Dump engine state
    DumpState {
        #[clap(long)]
        include_facts: bool,
    },
    
    /// Trace fact processing
    TraceFacts {
        #[clap(long)]
        fact_ids: Vec<u64>,
    },
    
    /// Memory analysis
    AnalyzeMemory,
}

impl DebugCommand {
    pub fn execute(&self, engine: &BingoEngine) -> BingoResult<()> {
        match self {
            DebugCommand::AnalyzeRules { rule_id } => {
                if let Some(id) = rule_id {
                    analyze_single_rule(engine, *id)?;
                } else {
                    analyze_all_rules(engine)?;
                }
            }
            DebugCommand::DumpState { include_facts } => {
                dump_engine_state(engine, *include_facts)?;
            }
            DebugCommand::TraceFacts { fact_ids } => {
                trace_fact_processing(engine, fact_ids)?;
            }
            DebugCommand::AnalyzeMemory => {
                analyze_memory_usage(engine)?;
            }
        }
        Ok(())
    }
}
```

---

This comprehensive developer guide provides everything needed to effectively develop, test, optimize, and maintain the Bingo RETE Rules Engine. The guide emphasizes practical examples, best practices, and proven patterns for building robust, high-performance rule processing systems.