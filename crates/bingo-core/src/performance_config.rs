//! Performance testing configuration and environment detection
//!
//! This module provides environment-adaptive performance testing capabilities,
//! allowing tests to adjust their thresholds based on the execution environment
//! (CI, local development, benchmarking).

use std::time::Duration;
use sys_info;

/// Performance testing environment types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceEnvironment {
    /// Local development environment - strictest thresholds
    Local,
    /// Continuous Integration environment - relaxed thresholds
    CI,
    /// Benchmarking environment - most relaxed thresholds
    Benchmark,
    /// Low-resource environment - most lenient thresholds
    LowResource,
    /// Custom environment with manual configuration
    Custom,
}

/// Defines a performance profile with specific multipliers
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    /// Name of the profile (e.g., "ci", "local")
    pub name: String,
    /// Time threshold multiplier
    pub time_multiplier: f64,
    /// Memory threshold multiplier
    pub memory_multiplier: f64,
}

/// Configuration for performance test thresholds
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Current environment type
    pub environment: PerformanceEnvironment,
    /// Time threshold multiplier (1.0 = base, 2.0 = double time allowance)
    pub time_multiplier: f64,
    /// Memory threshold multiplier (1.0 = base, 2.0 = double memory allowance)
    pub memory_multiplier: f64,
    /// Whether to enable detailed performance logging
    pub verbose_logging: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self::detect_environment()
    }
}

impl PerformanceConfig {
    /// Detect the current execution environment and create appropriate config
    pub fn detect_environment() -> Self {
        let environment = detect_environment();

        match environment {
            PerformanceEnvironment::Local => Self::from_profile("local"),
            PerformanceEnvironment::CI => Self::from_profile("ci"),
            PerformanceEnvironment::Benchmark => Self::from_profile("benchmark"),
            PerformanceEnvironment::LowResource => Self::from_profile("low_resource"),
            PerformanceEnvironment::Custom => Self::from_profile("custom"),
        }
    }

    /// Create a configuration from a specific performance profile, with dynamic scaling
    pub fn from_profile(profile_name: &str) -> Self {
        let (environment, mut time_multiplier, mut memory_multiplier) = match profile_name {
            "local" => (PerformanceEnvironment::Local, 1.0, 1.0),
            "ci" => (PerformanceEnvironment::CI, 2.0, 1.5),
            "benchmark" => (PerformanceEnvironment::Benchmark, 3.0, 2.0),
            "low_resource" => (PerformanceEnvironment::LowResource, 4.0, 2.5),
            _ => (PerformanceEnvironment::Custom, 1.0, 1.0),
        };

        // Apply dynamic scaling for local and CI environments
        if let Ok(scaling_factor) = calculate_scaling_factor() {
            if environment == PerformanceEnvironment::Local
                || environment == PerformanceEnvironment::CI
            {
                time_multiplier *= scaling_factor;
                memory_multiplier *= scaling_factor;
            }
        }

        Self { environment, time_multiplier, memory_multiplier, verbose_logging: true }
    }

    /// Create a custom configuration with specific multipliers
    pub fn custom(time_multiplier: f64, memory_multiplier: f64) -> Self {
        Self {
            environment: PerformanceEnvironment::Custom,
            time_multiplier,
            memory_multiplier,
            verbose_logging: true,
        }
    }

    /// Calculate adjusted time threshold based on environment
    pub fn adjust_time_threshold(&self, base_duration: Duration) -> Duration {
        let adjusted_secs = base_duration.as_secs_f64() * self.time_multiplier;
        Duration::from_secs_f64(adjusted_secs)
    }

    /// Calculate adjusted memory threshold based on environment
    pub fn adjust_memory_threshold(&self, base_bytes: u64) -> u64 {
        (base_bytes as f64 * self.memory_multiplier) as u64
    }

    /// Get a descriptive string for the current configuration
    pub fn description(&self) -> String {
        format!(
            "{:?} environment (time: {:.2}x, memory: {:.2}x)",
            self.environment, self.time_multiplier, self.memory_multiplier
        )
    }
}

/// Get system resource information (CPU speed and memory)
pub fn get_system_info() -> Result<(u64, u64), sys_info::Error> {
    let cpu_speed = sys_info::cpu_speed()?;
    let mem_info = sys_info::mem_info()?;
    Ok((cpu_speed, mem_info.total))
}

/// Calculate a performance scaling factor based on system resources
///
/// This function compares the current system's resources against a baseline
/// to produce a scaling factor for performance thresholds.
///
/// Baseline: 3 GHz CPU, 16 GB RAM
const BASELINE_CPU_SPEED: f64 = 3000.0; // 3 GHz
const BASELINE_MEMORY: f64 = 16_000_000.0; // 16 GB in KB

pub fn calculate_scaling_factor() -> Result<f64, sys_info::Error> {
    let (cpu_speed, total_memory) = get_system_info()?;

    let cpu_factor = BASELINE_CPU_SPEED / cpu_speed as f64;
    let mem_factor = BASELINE_MEMORY / total_memory as f64;

    // Combine factors, giving more weight to CPU
    let scaling_factor = (cpu_factor * 0.7) + (mem_factor * 0.3);

    // Clamp the factor to a reasonable range (e.g., 0.5x to 4x)
    Ok(scaling_factor.clamp(0.5, 4.0))
}

/// Detect the current execution environment
fn detect_environment() -> PerformanceEnvironment {
    // Check for explicit environment variable override
    if let Ok(env) = std::env::var("BINGO_PERF_ENV") {
        match env.to_lowercase().as_str() {
            "local" => return PerformanceEnvironment::Local,
            "ci" => return PerformanceEnvironment::CI,
            "benchmark" => return PerformanceEnvironment::Benchmark,
            "low_resource" => return PerformanceEnvironment::LowResource,
            "custom" => return PerformanceEnvironment::Custom,
            _ => {}
        }
    }

    // Check for common CI environment variables
    if is_ci_environment() {
        return PerformanceEnvironment::CI;
    }

    // Check for benchmark indicators
    if std::env::var("BINGO_BENCHMARK").is_ok() {
        return PerformanceEnvironment::Benchmark;
    }

    // Check for low resource environments
    if is_low_resource_environment() {
        return PerformanceEnvironment::LowResource;
    }

    // Default to local environment
    PerformanceEnvironment::Local
}

/// Check if we're running in a low-resource environment
const LOW_CPU_SPEED: u64 = 2000; // 2 GHz
const LOW_MEMORY: u64 = 4_000_000; // 4 GB in KB

fn is_low_resource_environment() -> bool {
    if let Ok((cpu_speed, total_memory)) = get_system_info() {
        return cpu_speed < LOW_CPU_SPEED || total_memory < LOW_MEMORY;
    }
    false
}

/// Check if we're running in a CI environment
fn is_ci_environment() -> bool {
    // Check for common CI environment variables
    let ci_vars = [
        "CI",
        "CONTINUOUS_INTEGRATION",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "JENKINS_URL",
        "TRAVIS",
        "CIRCLECI",
        "APPVEYOR",
        "DRONE",
        "BUILDKITE",
    ];

    ci_vars.iter().any(|var| std::env::var(var).is_ok())
}

/// Convenience macro for performance assertions with environment adaptation
#[macro_export]
macro_rules! assert_performance {
    ($condition:expr, $message:expr) => {
        let config = $crate::performance_config::PerformanceConfig::detect_environment();
        if config.verbose_logging {
            println!("🔧 Performance test running in: {}", config.description());
        }
        assert!(
            $condition,
            "{} (Environment: {:?})",
            $message, config.environment
        );
    };

    ($condition:expr, $message:expr, $config:expr) => {
        if $config.verbose_logging {
            println!("🔧 Performance test running in: {}", $config.description());
        }
        assert!(
            $condition,
            "{} (Environment: {:?})",
            $message, $config.environment
        );
    };
}

/// Convenience macro for time-based performance assertions
#[macro_export]
macro_rules! assert_time_performance {
    ($elapsed:expr, $base_duration:expr, $message:expr) => {
        let config = $crate::performance_config::PerformanceConfig::detect_environment();
        let adjusted_threshold = config.adjust_time_threshold($base_duration);

        if config.verbose_logging {
            println!(
                "⏱️  Time threshold: {:?} -> {:?} ({})",
                $base_duration,
                adjusted_threshold,
                config.description()
            );
        }

        assert!(
            $elapsed <= adjusted_threshold,
            "{} - took {:?}, expected <= {:?} (Environment: {:?})",
            $message,
            $elapsed,
            adjusted_threshold,
            config.environment
        );
    };
}

/// Convenience macro for memory-based performance assertions
#[macro_export]
macro_rules! assert_memory_performance {
    ($memory_used:expr, $base_limit:expr, $message:expr) => {
        let config = $crate::performance_config::PerformanceConfig::detect_environment();
        let adjusted_threshold = config.adjust_memory_threshold($base_limit);

        if config.verbose_logging {
            println!(
                "🧠 Memory threshold: {} -> {} bytes ({})",
                $base_limit,
                adjusted_threshold,
                config.description()
            );
        }

        assert!(
            $memory_used <= adjusted_threshold,
            "{} - used {} bytes, expected <= {} bytes (Environment: {:?})",
            $message,
            $memory_used,
            adjusted_threshold,
            config.environment
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_detection() {
        let config = PerformanceConfig::detect_environment();
        assert!(config.time_multiplier > 0.0);
        assert!(config.memory_multiplier > 0.0);
    }

    #[test]
    fn test_custom_config() {
        let config = PerformanceConfig::custom(1.5, 2.0);
        assert_eq!(config.environment, PerformanceEnvironment::Custom);
        assert_eq!(config.time_multiplier, 1.5);
        assert_eq!(config.memory_multiplier, 2.0);
    }

    #[test]
    fn test_threshold_adjustment() {
        let config = PerformanceConfig::custom(2.0, 1.5);

        let base_duration = Duration::from_secs(5);
        let adjusted_duration = config.adjust_time_threshold(base_duration);
        assert_eq!(adjusted_duration, Duration::from_secs(10));

        let base_memory = 1_000_000;
        let adjusted_memory = config.adjust_memory_threshold(base_memory);
        assert_eq!(adjusted_memory, 1_500_000);
    }

    #[test]
    fn test_ci_detection() {
        // This would need to be tested with actual CI environment variables
        // For now, just ensure the function doesn't panic
        let _is_ci = is_ci_environment();
    }

    #[test]
    fn test_scaling_factor_calculation() {
        // This test will only run if sys-info can successfully get system info
        if let Ok(scaling_factor) = calculate_scaling_factor() {
            println!("Detected performance scaling factor: {scaling_factor}");
            assert!((0.5..=4.0).contains(&scaling_factor));
        }
    }
}
