# Bingo RETE Engine - Plugin Architecture Specification

## Overview

This document defines the plugin architecture for the Bingo RETE Engine, providing a standardized way to extend engine functionality without modifying core components. The plugin system enables safe, modular extensions for storage backends, calculators, monitoring tools, and custom business logic.

## Plugin System Architecture

### Core Plugin Interface

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::any::Any;

/// Base trait that all plugins must implement
pub trait Plugin: Send + Sync + std::fmt::Debug {
    /// Plugin identifier (must be unique)
    fn name(&self) -> &str;
    
    /// Plugin version (semantic versioning)
    fn version(&self) -> &str;
    
    /// Plugin description
    fn description(&self) -> &str;
    
    /// Plugin dependencies (other plugins this plugin requires)
    fn dependencies(&self) -> Vec<PluginDependency>;
    
    /// Initialize plugin with engine instance
    fn initialize(&mut self, context: &mut PluginContext) -> Result<()>;
    
    /// Shutdown plugin gracefully
    fn shutdown(&mut self) -> Result<()>;
    
    /// Plugin configuration schema
    fn config_schema(&self) -> serde_json::Value;
    
    /// Configure plugin with settings
    fn configure(&mut self, config: serde_json::Value) -> Result<()>;
    
    /// Plugin health check
    fn health_check(&self) -> PluginHealth;
    
    /// Get plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Get mutable plugin as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String, // semver requirement
    pub optional: bool,
}

/// Plugin health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Unhealthy,
    Unknown,
}

/// Plugin execution context
pub struct PluginContext {
    pub engine: *mut Engine, // Raw pointer for safety
    pub services: PluginServices,
    pub logger: PluginLogger,
    pub metrics: PluginMetrics,
}

/// Services available to plugins
pub struct PluginServices {
    pub fact_store: Box<dyn FactStore>,
    pub calculator_registry: CalculatorRegistry,
    pub event_bus: EventBus,
    pub config_manager: ConfigManager,
}
```

### Plugin Manager

```rust
use std::path::Path;
use std::collections::HashMap;
use libloading::Library;

/// Central plugin management system
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    plugin_registry: PluginRegistry,
    dependency_graph: DependencyGraph,
    configuration: PluginConfiguration,
}

/// Loaded plugin container
struct LoadedPlugin {
    plugin: Box<dyn Plugin>,
    library: Option<Library>, // For dynamically loaded plugins
    status: PluginStatus,
    metrics: PluginMetrics,
}

#[derive(Debug, Clone)]
pub enum PluginStatus {
    Loaded,
    Initialized,
    Running,
    Stopped,
    Error(String),
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_registry: PluginRegistry::new(),
            dependency_graph: DependencyGraph::new(),
            configuration: PluginConfiguration::default(),
        }
    }
    
    /// Register a compiled-in plugin
    pub fn register_static_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        self.validate_plugin_dependencies(&plugin)?;
        
        let loaded = LoadedPlugin {
            plugin,
            library: None,
            status: PluginStatus::Loaded,
            metrics: PluginMetrics::new(),
        };
        
        self.plugins.insert(name.clone(), loaded);
        self.dependency_graph.add_plugin(&name)?;
        
        Ok(())
    }
    
    /// Load plugin from shared library
    pub fn load_dynamic_plugin(&mut self, path: &Path) -> Result<()> {
        unsafe {
            let lib = Library::new(path)?;
            let create_plugin: Symbol<unsafe extern fn() -> *mut dyn Plugin> = 
                lib.get(b"create_plugin")?;
            
            let plugin = Box::from_raw(create_plugin());
            let name = plugin.name().to_string();
            
            self.validate_plugin_dependencies(&plugin)?;
            
            let loaded = LoadedPlugin {
                plugin,
                library: Some(lib),
                status: PluginStatus::Loaded,
                metrics: PluginMetrics::new(),
            };
            
            self.plugins.insert(name.clone(), loaded);
            self.dependency_graph.add_plugin(&name)?;
        }
        
        Ok(())
    }
    
    /// Initialize all plugins in dependency order
    pub fn initialize_all(&mut self, context: &mut PluginContext) -> Result<()> {
        let init_order = self.dependency_graph.resolve_initialization_order()?;
        
        for plugin_name in init_order {
            self.initialize_plugin(&plugin_name, context)?;
        }
        
        Ok(())
    }
    
    /// Initialize specific plugin
    pub fn initialize_plugin(&mut self, name: &str, context: &mut PluginContext) -> Result<()> {
        if let Some(loaded) = self.plugins.get_mut(name) {
            loaded.plugin.initialize(context)?;
            loaded.status = PluginStatus::Initialized;
        }
        Ok(())
    }
    
    /// Get plugin by name and type
    pub fn get_plugin<T: Plugin + 'static>(&self, name: &str) -> Option<&T> {
        self.plugins.get(name)
            .and_then(|loaded| loaded.plugin.as_any().downcast_ref::<T>())
    }
    
    /// Get mutable plugin by name and type
    pub fn get_plugin_mut<T: Plugin + 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.plugins.get_mut(name)
            .and_then(|loaded| loaded.plugin.as_any_mut().downcast_mut::<T>())
    }
    
    /// Shutdown all plugins
    pub fn shutdown_all(&mut self) -> Result<()> {
        let shutdown_order = self.dependency_graph.resolve_shutdown_order()?;
        
        for plugin_name in shutdown_order {
            if let Some(loaded) = self.plugins.get_mut(&plugin_name) {
                loaded.plugin.shutdown()?;
                loaded.status = PluginStatus::Stopped;
            }
        }
        
        Ok(())
    }
}
```

## Specific Plugin Types

### 1. Fact Store Plugins

```rust
/// Fact store plugin interface
pub trait FactStorePlugin: Plugin {
    /// Create a new fact store instance
    fn create_fact_store(&self, config: serde_json::Value) -> Result<Box<dyn FactStore>>;
    
    /// Get supported configuration options
    fn supported_options(&self) -> Vec<FactStoreOption>;
}

/// Configuration option for fact stores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactStoreOption {
    pub name: String,
    pub description: String,
    pub option_type: OptionType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<OptionType>),
    Object(HashMap<String, OptionType>),
}

/// Example: Database fact store plugin
pub struct DatabaseFactStorePlugin {
    name: String,
    version: String,
    supported_databases: Vec<String>,
}

impl Plugin for DatabaseFactStorePlugin {
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> &str { &self.version }
    fn description(&self) -> &str { "Database-backed fact storage" }
    
    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![] // No dependencies
    }
    
    fn initialize(&mut self, _context: &mut PluginContext) -> Result<()> {
        // Initialize database drivers
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        // Close database connections
        Ok(())
    }
    
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "connection_string": {"type": "string"},
                "table_name": {"type": "string"},
                "max_connections": {"type": "integer", "default": 10}
            },
            "required": ["connection_string", "table_name"]
        })
    }
    
    fn configure(&mut self, _config: serde_json::Value) -> Result<()> {
        // Configure database settings
        Ok(())
    }
    
    fn health_check(&self) -> PluginHealth {
        // Check database connectivity
        PluginHealth {
            status: HealthStatus::Healthy,
            message: Some("Database connection active".to_string()),
            details: HashMap::new(),
        }
    }
    
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl FactStorePlugin for DatabaseFactStorePlugin {
    fn create_fact_store(&self, config: serde_json::Value) -> Result<Box<dyn FactStore>> {
        // Create database fact store with configuration
        Ok(Box::new(DatabaseFactStore::new(config)?))
    }
    
    fn supported_options(&self) -> Vec<FactStoreOption> {
        vec![
            FactStoreOption {
                name: "connection_string".to_string(),
                description: "Database connection string".to_string(),
                option_type: OptionType::String,
                required: true,
                default_value: None,
            },
            // ... more options
        ]
    }
}
```

### 2. Calculator Plugins

```rust
/// Calculator plugin interface
pub trait CalculatorPlugin: Plugin {
    /// Get supported calculator types
    fn supported_calculator_types(&self) -> Vec<String>;
    
    /// Create calculator instance
    fn create_calculator(&self, calc_type: &str, config: serde_json::Value) -> Result<Box<dyn CalculatorEngine>>;
    
    /// Validate calculator configuration
    fn validate_config(&self, calc_type: &str, config: &serde_json::Value) -> Result<()>;
}

/// Example: Machine Learning calculator plugin
pub struct MLCalculatorPlugin {
    name: String,
    version: String,
    model_registry: ModelRegistry,
}

impl Plugin for MLCalculatorPlugin {
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> &str { &self.version }
    fn description(&self) -> &str { "Machine learning model integration" }
    
    // ... other Plugin methods
}

impl CalculatorPlugin for MLCalculatorPlugin {
    fn supported_calculator_types(&self) -> Vec<String> {
        vec![
            "LinearRegression".to_string(),
            "DecisionTree".to_string(),
            "NeuralNetwork".to_string(),
        ]
    }
    
    fn create_calculator(&self, calc_type: &str, config: serde_json::Value) -> Result<Box<dyn CalculatorEngine>> {
        match calc_type {
            "LinearRegression" => Ok(Box::new(LinearRegressionCalculator::new(config)?)),
            "DecisionTree" => Ok(Box::new(DecisionTreeCalculator::new(config)?)),
            "NeuralNetwork" => Ok(Box::new(NeuralNetworkCalculator::new(config)?)),
            _ => Err(anyhow::anyhow!("Unsupported calculator type: {}", calc_type)),
        }
    }
    
    fn validate_config(&self, calc_type: &str, config: &serde_json::Value) -> Result<()> {
        // Validate model file exists, parameters are correct, etc.
        Ok(())
    }
}
```

### 3. Monitoring Plugins

```rust
/// Monitoring plugin interface
pub trait MonitoringPlugin: Plugin {
    /// Subscribe to engine events
    fn subscribe_to_events(&mut self, event_bus: &mut EventBus) -> Result<()>;
    
    /// Handle performance metrics
    fn handle_metrics(&mut self, metrics: &PerformanceMetrics) -> Result<()>;
    
    /// Handle alerts
    fn handle_alert(&mut self, alert: &Alert) -> Result<()>;
}

/// Example: Prometheus monitoring plugin
pub struct PrometheusMonitoringPlugin {
    name: String,
    version: String,
    metrics_endpoint: String,
    metric_registry: prometheus::Registry,
}

impl Plugin for PrometheusMonitoringPlugin {
    // ... Plugin implementation
}

impl MonitoringPlugin for PrometheusMonitoringPlugin {
    fn subscribe_to_events(&mut self, event_bus: &mut EventBus) -> Result<()> {
        event_bus.subscribe("rule_fired", Box::new(|event| {
            // Update Prometheus metrics
        }))?;
        
        event_bus.subscribe("fact_processed", Box::new(|event| {
            // Update fact processing metrics
        }))?;
        
        Ok(())
    }
    
    fn handle_metrics(&mut self, metrics: &PerformanceMetrics) -> Result<()> {
        // Export metrics to Prometheus
        Ok(())
    }
    
    fn handle_alert(&mut self, alert: &Alert) -> Result<()> {
        // Forward alert to Prometheus Alertmanager
        Ok(())
    }
}
```

## Plugin Configuration System

### Configuration Schema

```rust
/// Plugin configuration management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfiguration {
    pub plugins: HashMap<String, PluginConfig>,
    pub global_settings: GlobalPluginSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: serde_json::Value,
    pub load_priority: i32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPluginSettings {
    pub plugin_directory: String,
    pub max_load_time_ms: u64,
    pub security_policy: SecurityPolicy,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub allow_dynamic_loading: bool,
    pub require_code_signing: bool,
    pub sandbox_plugins: bool,
    pub allowed_system_calls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_file_handles: u32,
    pub max_network_connections: u32,
}
```

### Example Configuration File

```toml
[plugins.database_store]
enabled = true
load_priority = 10
settings = { connection_string = "postgresql://localhost/bingo", table_name = "facts" }

[plugins.ml_calculator]
enabled = true
load_priority = 20
dependencies = ["database_store"]
settings = { model_path = "/models/", cache_size = 1000 }

[plugins.prometheus_monitoring]
enabled = true
load_priority = 30
settings = { endpoint = "http://localhost:9090", scrape_interval = "30s" }

[global_settings]
plugin_directory = "/usr/local/lib/bingo/plugins"
max_load_time_ms = 5000

[global_settings.security_policy]
allow_dynamic_loading = true
require_code_signing = false
sandbox_plugins = true

[global_settings.resource_limits]
max_memory_mb = 1024
max_cpu_percent = 50.0
max_file_handles = 100
max_network_connections = 50
```

## Plugin Development Guidelines

### Plugin Development Template

```rust
use bingo_core::{Plugin, PluginContext, PluginHealth, PluginDependency};
use anyhow::Result;

pub struct MyPlugin {
    name: String,
    version: String,
    // Plugin-specific fields
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            name: "my_plugin".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> &str { &self.version }
    fn description(&self) -> &str { "My custom plugin" }
    
    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![]
    }
    
    fn initialize(&mut self, context: &mut PluginContext) -> Result<()> {
        // Initialize plugin
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<()> {
        // Cleanup plugin resources
        Ok(())
    }
    
    fn config_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "setting1": {"type": "string"},
                "setting2": {"type": "integer"}
            }
        })
    }
    
    fn configure(&mut self, config: serde_json::Value) -> Result<()> {
        // Configure plugin with provided settings
        Ok(())
    }
    
    fn health_check(&self) -> PluginHealth {
        PluginHealth {
            status: HealthStatus::Healthy,
            message: None,
            details: HashMap::new(),
        }
    }
    
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// For dynamic loading
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn Plugin {
    Box::into_raw(Box::new(MyPlugin::new()))
}
```

### Build Configuration (Cargo.toml)

```toml
[package]
name = "bingo-plugin-my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
bingo-core = "1.0"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
cbindgen = "0.20"
```

### Plugin Testing Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bingo_core::testing::{MockPluginContext, PluginTestHarness};
    
    #[test]
    fn test_plugin_initialization() {
        let mut plugin = MyPlugin::new();
        let mut context = MockPluginContext::new();
        
        assert!(plugin.initialize(&mut context).is_ok());
        assert_eq!(plugin.health_check().status, HealthStatus::Healthy);
    }
    
    #[test]
    fn test_plugin_configuration() {
        let mut plugin = MyPlugin::new();
        let config = serde_json::json!({
            "setting1": "value1",
            "setting2": 42
        });
        
        assert!(plugin.configure(config).is_ok());
    }
    
    #[test]
    fn test_plugin_lifecycle() {
        let harness = PluginTestHarness::new();
        let plugin = Box::new(MyPlugin::new());
        
        harness.test_full_lifecycle(plugin);
    }
}
```

## Security and Sandboxing

### Plugin Sandboxing

```rust
/// Plugin sandbox configuration
pub struct PluginSandbox {
    memory_limit: usize,
    cpu_limit: f64,
    network_access: bool,
    file_access: FileAccessPolicy,
    system_calls: SystemCallPolicy,
}

#[derive(Debug, Clone)]
pub enum FileAccessPolicy {
    None,
    ReadOnly(Vec<String>), // Allowed paths
    ReadWrite(Vec<String>), // Allowed paths
    Full,
}

#[derive(Debug, Clone)]
pub enum SystemCallPolicy {
    Whitelist(Vec<String>),
    Blacklist(Vec<String>),
    None,
    All,
}

impl PluginSandbox {
    pub fn create_sandbox(&self, plugin_name: &str) -> Result<SandboxEnvironment> {
        // Create isolated environment for plugin
        Ok(SandboxEnvironment::new(plugin_name, self.clone())?)
    }
}
```

### Code Signing and Verification

```rust
/// Plugin verification system
pub struct PluginVerifier {
    trusted_keys: Vec<PublicKey>,
    verification_policy: VerificationPolicy,
}

#[derive(Debug, Clone)]
pub enum VerificationPolicy {
    Required,
    Optional,
    Disabled,
}

impl PluginVerifier {
    pub fn verify_plugin(&self, plugin_path: &Path) -> Result<VerificationResult> {
        // Verify plugin signature and integrity
        Ok(VerificationResult::Trusted)
    }
}

#[derive(Debug, Clone)]
pub enum VerificationResult {
    Trusted,
    Untrusted,
    Invalid,
}
```

## Plugin Registry and Discovery

### Plugin Registry

```rust
/// Central plugin registry
pub struct PluginRegistry {
    local_plugins: HashMap<String, PluginMetadata>,
    remote_registries: Vec<RemoteRegistry>,
    cache: PluginCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub tags: Vec<String>,
    pub compatibility: EngineCompatibility,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCompatibility {
    pub min_version: String,
    pub max_version: Option<String>,
    pub features: Vec<String>,
}

impl PluginRegistry {
    pub fn discover_plugins(&mut self, directory: &Path) -> Result<Vec<PluginMetadata>> {
        // Scan directory for plugin files and metadata
        Ok(vec![])
    }
    
    pub fn search_plugins(&self, query: &str) -> Vec<&PluginMetadata> {
        // Search plugins by name, description, tags
        vec![]
    }
    
    pub fn install_plugin(&mut self, name: &str, version: Option<&str>) -> Result<()> {
        // Download and install plugin
        Ok(())
    }
}
```

This plugin architecture provides a comprehensive foundation for extending the Bingo RETE Engine while maintaining security, performance, and compatibility. The system supports both static (compiled-in) and dynamic (runtime-loaded) plugins with proper dependency management, configuration, and lifecycle management.