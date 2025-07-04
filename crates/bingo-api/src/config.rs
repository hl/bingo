use serde::{Deserialize, Serialize};
use std::fs;
use toml;
use tracing::{info, warn};
use utoipa::ToSchema;

#[derive(Deserialize, Debug, Clone)]
pub struct Environment {
    pub env_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LimitsConfig {
    pub max_body_size_mb: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CachingConfig {
    pub cache_type: String,
    pub redis_url: Option<String>,
    pub ruleset_cache_ttl_minutes: u64,
    pub engine_cache_ttl_minutes: u64,
    pub adhoc_ruleset_ttl_seconds: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecurityConfig {
    #[serde(default = "default_max_rules_per_request")]
    pub max_rules_per_request: usize,
    #[serde(default = "default_max_facts_per_request")]
    pub max_facts_per_request: usize,
    #[serde(default = "default_max_conditions_per_rule")]
    pub max_conditions_per_rule: usize,
    #[serde(default = "default_max_expression_depth")]
    pub max_expression_depth: usize,
    #[serde(default = "default_max_expression_complexity")]
    pub max_expression_complexity: usize,
    #[serde(default = "default_max_expression_length")]
    pub max_expression_length: usize,
    #[serde(default = "default_max_calculator_inputs")]
    pub max_calculator_inputs: usize,
    #[serde(default = "default_max_streaming_chunk_size")]
    pub max_streaming_chunk_size: usize,
    #[serde(default = "default_max_rule_name_length")]
    pub max_rule_name_length: usize,
    #[serde(default = "default_max_rule_description_length")]
    pub max_rule_description_length: usize,
    #[serde(default = "default_max_calculator_name_length")]
    pub max_calculator_name_length: usize,
    #[serde(default = "default_max_calculator_input_key_length")]
    pub max_calculator_input_key_length: usize,
    #[serde(default = "default_max_calculator_input_value_length")]
    pub max_calculator_input_value_length: usize,
    #[serde(default = "default_max_log_message_length")]
    pub max_log_message_length: usize,
    #[serde(default = "default_max_created_fact_fields")]
    pub max_created_fact_fields: usize,
    #[serde(default = "default_max_created_fact_field_value_length")]
    pub max_created_fact_field_value_length: usize,
    #[serde(default = "default_memory_safety_threshold")]
    pub memory_safety_threshold: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IncrementalProcessingConfig {
    pub default_memory_limit_mb: usize,
    pub default_fact_batch_size: usize,
}

impl Default for IncrementalProcessingConfig {
    fn default() -> Self {
        Self { default_memory_limit_mb: 2048, default_fact_batch_size: 1000 }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct HardeningConfig {
    #[serde(default = "default_true")]
    pub enable_concurrency_limiter: bool,
    #[serde(default = "default_true")]
    pub enable_rate_limiter: bool,
    #[serde(default = "default_true")]
    pub enable_request_monitor: bool,
    #[serde(default = "default_true")]
    pub enable_timeout: bool,
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    #[serde(default = "default_request_timeout_seconds")]
    pub request_timeout_seconds: u64,
}

impl Default for HardeningConfig {
    fn default() -> Self {
        Self {
            enable_concurrency_limiter: true,
            enable_rate_limiter: true,
            enable_request_monitor: true,
            enable_timeout: true,
            max_concurrent_requests: default_max_concurrent_requests(),
            requests_per_minute: default_requests_per_minute(),
            request_timeout_seconds: default_request_timeout_seconds(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BingoConfig {
    pub environment: Environment,
    pub limits: LimitsConfig,
    pub caching: CachingConfig,
    pub security: SecurityConfig,
    #[serde(default)]
    pub hardening: HardeningConfig,
    #[serde(default)]
    pub incremental_processing: IncrementalProcessingConfig,
}

impl BingoConfig {
    pub fn load() -> Self {
        // Load configuration with environment variable support
        let config_path =
            std::env::var("BINGO_CONFIG_PATH").unwrap_or_else(|_| "bingo.toml".to_string());

        let config_str = fs::read_to_string(&config_path).unwrap_or_else(|_| {
            warn!(
                "Configuration file '{}' not found. Using default configuration.",
                config_path
            );
            // Provide a default config string to avoid panics if the file is missing.
            r#"[environment]
env_type = "default"
[limits]
max_body_size_mb = 16
[caching]
cache_type = "in_memory"
ruleset_cache_ttl_minutes = 60
engine_cache_ttl_minutes = 30
adhoc_ruleset_ttl_seconds = 300
[security]
max_rules_per_request = 1000
max_facts_per_request = 100000
max_conditions_per_rule = 50
max_expression_depth = 50
max_expression_complexity = 1000
max_expression_length = 10000
max_calculator_inputs = 100
memory_safety_threshold = 10000
"#
            .to_string()
        });

        toml::from_str(&config_str).expect("Failed to parse configuration file.")
    }

    pub fn apply_profile(mut self) -> Self {
        // Apply environment variable overrides
        info!(
            "Applying configuration profile for '{}' environment.",
            self.environment.env_type
        );

        // Environment overrides for cache configuration
        if let Ok(cache_type) = std::env::var("BINGO_CACHE_TYPE") {
            self.caching.cache_type = cache_type;
        }
        if let Ok(redis_url) = std::env::var("BINGO_REDIS_URL") {
            self.caching.redis_url = Some(redis_url);
        }
        if let Ok(ttl) = std::env::var("BINGO_CACHE_TTL_MINUTES") {
            if let Ok(ttl_num) = ttl.parse::<u64>() {
                self.caching.ruleset_cache_ttl_minutes = ttl_num;
                self.caching.engine_cache_ttl_minutes = ttl_num;
            }
        }

        // Environment overrides for security limits
        if let Ok(max_rules) = std::env::var("BINGO_MAX_RULES_PER_REQUEST") {
            if let Ok(max_rules_num) = max_rules.parse::<usize>() {
                self.security.max_rules_per_request = max_rules_num;
            }
        }
        if let Ok(max_facts) = std::env::var("BINGO_MAX_FACTS_PER_REQUEST") {
            if let Ok(max_facts_num) = max_facts.parse::<usize>() {
                self.security.max_facts_per_request = max_facts_num;
            }
        }

        // Environment overrides for hardening
        if let Ok(max_concurrent) = std::env::var("BINGO_MAX_CONCURRENT_REQUESTS") {
            if let Ok(max_concurrent_num) = max_concurrent.parse::<usize>() {
                self.hardening.max_concurrent_requests = max_concurrent_num;
            }
        }
        if let Ok(rate_limit) = std::env::var("BINGO_REQUESTS_PER_MINUTE") {
            if let Ok(rate_limit_num) = rate_limit.parse::<u32>() {
                self.hardening.requests_per_minute = rate_limit_num;
            }
        }

        self
    }
}

fn default_max_rules_per_request() -> usize {
    1000
}
fn default_max_facts_per_request() -> usize {
    100_000
}
fn default_max_conditions_per_rule() -> usize {
    50
}
fn default_max_expression_depth() -> usize {
    50
}
fn default_max_expression_complexity() -> usize {
    1000
}
fn default_max_expression_length() -> usize {
    10000
}
fn default_max_calculator_inputs() -> usize {
    100
}

fn default_true() -> bool {
    true
}
fn default_max_concurrent_requests() -> usize {
    100
}
fn default_requests_per_minute() -> u32 {
    300
}
fn default_request_timeout_seconds() -> u64 {
    120
}

fn default_max_streaming_chunk_size() -> usize {
    10000
}
fn default_max_rule_name_length() -> usize {
    1000
}
fn default_max_rule_description_length() -> usize {
    5000
}
fn default_max_calculator_name_length() -> usize {
    100
}
fn default_max_calculator_input_key_length() -> usize {
    100
}
fn default_max_calculator_input_value_length() -> usize {
    10000
}
fn default_max_log_message_length() -> usize {
    10000
}
fn default_max_created_fact_fields() -> usize {
    1000
}
fn default_max_created_fact_field_value_length() -> usize {
    10000
}
fn default_memory_safety_threshold() -> usize {
    10000
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_rules_per_request: default_max_rules_per_request(),
            max_facts_per_request: default_max_facts_per_request(),
            max_conditions_per_rule: default_max_conditions_per_rule(),
            max_expression_depth: default_max_expression_depth(),
            max_expression_complexity: default_max_expression_complexity(),
            max_expression_length: default_max_expression_length(),
            max_calculator_inputs: default_max_calculator_inputs(),
            max_streaming_chunk_size: default_max_streaming_chunk_size(),
            max_rule_name_length: default_max_rule_name_length(),
            max_rule_description_length: default_max_rule_description_length(),
            max_calculator_name_length: default_max_calculator_name_length(),
            max_calculator_input_key_length: default_max_calculator_input_key_length(),
            max_calculator_input_value_length: default_max_calculator_input_value_length(),
            max_log_message_length: default_max_log_message_length(),
            max_created_fact_fields: default_max_created_fact_fields(),
            max_created_fact_field_value_length: default_max_created_fact_field_value_length(),
            memory_safety_threshold: default_memory_safety_threshold(),
        }
    }
}
