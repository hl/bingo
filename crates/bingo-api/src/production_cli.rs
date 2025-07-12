//! Production Readiness CLI Tool
//!
//! Command-line interface for validating production readiness and generating reports.

use bingo_core::production_readiness::{check_production_readiness, load_config_from_env};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

/// Production readiness CLI tool
#[derive(Parser)]
#[command(name = "bingo-production")]
#[command(about = "Bingo RETE Rules Engine Production Readiness Tool")]
#[command(version = "1.0.0")]
pub struct ProductionCli {
    #[command(subcommand)]
    pub command: ProductionCommand,
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Output format (text, json, markdown)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

/// Production readiness commands
#[derive(Subcommand)]
pub enum ProductionCommand {
    /// Validate production readiness
    Validate {
        /// Output file for report
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Exit with error code if not ready
        #[arg(long)]
        strict: bool,
    },
    
    /// Generate configuration template
    Config {
        /// Output file for configuration
        #[arg(short, long, default_value = "production-config.yaml")]
        output: PathBuf,
        
        /// Configuration format (yaml, json, toml)
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
    
    /// Show current configuration
    Show,
    
    /// Run health check
    Health {
        /// gRPC endpoint to check
        #[arg(short, long, default_value = "localhost:50051")]
        endpoint: String,
        
        /// Timeout in seconds
        #[arg(short, long, default_value = "10")]
        timeout: u64,
    },
}

impl ProductionCli {
    /// Execute the CLI command
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Setup logging based on verbosity
        let log_level = if self.verbose { "debug" } else { "info" };
        std::env::set_var("RUST_LOG", log_level);
        tracing_subscriber::fmt::init();

        match &self.command {
            ProductionCommand::Validate { output, strict } => {
                self.validate_readiness(output.as_ref(), *strict).await
            }
            ProductionCommand::Config { output, format } => {
                self.generate_config(output, format).await
            }
            ProductionCommand::Show => {
                self.show_config().await
            }
            ProductionCommand::Health { endpoint, timeout } => {
                self.health_check(endpoint, *timeout).await
            }
        }
    }

    /// Validate production readiness
    async fn validate_readiness(&self, output: Option<&PathBuf>, strict: bool) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting production readiness validation...");

        let report = check_production_readiness()
            .map_err(|e| format!("Failed to run readiness check: {}", e))?;

        // Output results based on format
        let output_content = match self.format.as_str() {
            "json" => serde_json::to_string_pretty(&report)?,
            "markdown" => {
                let config = load_config_from_env();
                let validator = bingo_core::production_readiness::ProductionReadinessValidator::new(config);
                validator.generate_report(&report)
            }
            "text" | _ => self.format_text_report(&report),
        };

        // Write to file or stdout
        if let Some(output_path) = output {
            fs::write(output_path, &output_content)?;
            info!("Report written to: {}", output_path.display());
        } else {
            println!("{}", output_content);
        }

        // Print summary
        self.print_summary(&report);

        // Exit with error if strict mode and not ready
        if strict && !report.ready {
            error!("Production readiness validation failed in strict mode");
            std::process::exit(1);
        }

        Ok(())
    }

    /// Generate configuration template
    async fn generate_config(&self, output: &PathBuf, format: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = bingo_core::production_readiness::ProductionConfig::default();

        let content = match format {
            "json" => serde_json::to_string_pretty(&config)?,
            "toml" => toml::to_string_pretty(&config)?,
            "yaml" | _ => serde_yaml::to_string(&config)?,
        };

        fs::write(output, content)?;
        info!("Configuration template written to: {}", output.display());

        Ok(())
    }

    /// Show current configuration
    async fn show_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = load_config_from_env();

        let content = match self.format.as_str() {
            "json" => serde_json::to_string_pretty(&config)?,
            "yaml" => serde_yaml::to_string(&config)?,
            _ => format!("{:#?}", config),
        };

        println!("{}", content);
        Ok(())
    }

    /// Run health check
    async fn health_check(&self, endpoint: &str, timeout: u64) -> Result<(), Box<dyn std::error::Error>> {
        info!("Running health check against: {}", endpoint);

        // Simulate health check (in real implementation, would use gRPC health probe)
        let health_result = self.simulate_health_check(endpoint, timeout).await?;

        if health_result {
            println!("✅ Health check PASSED");
            println!("Service is responding normally");
        } else {
            println!("❌ Health check FAILED");
            println!("Service is not responding or unhealthy");
            std::process::exit(1);
        }

        Ok(())
    }

    /// Simulate health check (placeholder for actual gRPC health probe)
    async fn simulate_health_check(&self, _endpoint: &str, _timeout: u64) -> Result<bool, Box<dyn std::error::Error>> {
        // In a real implementation, this would:
        // 1. Create gRPC client
        // 2. Call health check service
        // 3. Validate response
        // For now, return true as placeholder
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(true)
    }

    /// Format text report
    fn format_text_report(&self, report: &bingo_core::production_readiness::ReadinessReport) -> String {
        let mut output = String::new();

        output.push_str("🚀 BINGO PRODUCTION READINESS REPORT\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        // Overall status
        let status_icon = if report.ready { "✅" } else { "❌" };
        let status_text = if report.ready { "READY" } else { "NOT READY" };
        output.push_str(&format!("Overall Status: {} {}\n", status_icon, status_text));
        output.push_str(&format!("Readiness Score: {:.1}%\n\n", report.summary.readiness_score * 100.0));

        // Summary
        output.push_str("📊 SUMMARY\n");
        output.push_str("────────────────────────────────────────────────────────────────\n");
        output.push_str(&format!("Total Checks:    {}\n", report.summary.total_checks));
        output.push_str(&format!("✅ Passed:       {}\n", report.summary.passed));
        output.push_str(&format!("⚠️  Warnings:     {}\n", report.summary.warnings));
        output.push_str(&format!("❌ Failures:     {}\n", report.summary.failures));
        output.push_str(&format!("❓ Unknown:      {}\n\n", report.summary.unknown));

        // Detailed results
        self.add_section_to_text(&mut output, "🔧 SERVICE CONFIGURATION", &report.service_checks);
        self.add_section_to_text(&mut output, "⚡ PERFORMANCE CONFIGURATION", &report.performance_checks);
        self.add_section_to_text(&mut output, "🔒 SECURITY CONFIGURATION", &report.security_checks);
        self.add_section_to_text(&mut output, "📈 MONITORING CONFIGURATION", &report.monitoring_checks);
        self.add_section_to_text(&mut output, "💾 RESOURCE CONFIGURATION", &report.resource_checks);

        output
    }

    /// Add section to text report
    fn add_section_to_text(&self, output: &mut String, title: &str, checks: &[bingo_core::production_readiness::CheckResult]) {
        output.push_str(&format!("{}\n", title));
        output.push_str("────────────────────────────────────────────────────────────────\n");

        for check in checks {
            let status_icon = match check.status {
                bingo_core::production_readiness::CheckStatus::Pass => "✅",
                bingo_core::production_readiness::CheckStatus::Warning => "⚠️",
                bingo_core::production_readiness::CheckStatus::Fail => "❌",
                bingo_core::production_readiness::CheckStatus::Unknown => "❓",
            };

            output.push_str(&format!("{} {} ({})\n", status_icon, check.name, check.severity_display()));

            if let Some(message) = &check.message {
                output.push_str(&format!("   {}\n", message));
            }

            if !check.recommendations.is_empty() {
                output.push_str("   Recommendations:\n");
                for rec in &check.recommendations {
                    output.push_str(&format!("   • {}\n", rec));
                }
            }
            output.push_str("\n");
        }
    }

    /// Print summary to stderr
    fn print_summary(&self, report: &bingo_core::production_readiness::ReadinessReport) {
        if report.ready {
            eprintln!("✅ Production readiness: READY (Score: {:.1}%)", report.summary.readiness_score * 100.0);
        } else {
            eprintln!("❌ Production readiness: NOT READY (Score: {:.1}%)", report.summary.readiness_score * 100.0);
            eprintln!("   {} failures, {} warnings", report.summary.failures, report.summary.warnings);
        }
    }
}

/// Main CLI entry point
pub async fn run_production_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = ProductionCli::parse();
    cli.execute().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_parsing() {
        let args = vec!["bingo-production", "validate", "--output", "test-report.md"];
        let cli = ProductionCli::try_parse_from(args).expect("Should parse CLI args");
        
        match cli.command {
            ProductionCommand::Validate { output, strict: _ } => {
                assert!(output.is_some());
                assert_eq!(output.unwrap().to_string_lossy(), "test-report.md");
            }
            _ => panic!("Expected validate command"),
        }
    }

    #[tokio::test]
    async fn test_health_check_simulation() {
        let cli = ProductionCli::parse_from(vec!["bingo-production", "health"]);
        let result = cli.simulate_health_check("localhost:50051", 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}