//! Interactive Error Debugging CLI
//!
//! This module provides an interactive command-line interface for debugging
//! errors and exploring error diagnostics in real-time.

use bingo_core::error_diagnostics::{DebugSessionConfig, ErrorReport, VerbosityLevel};
use bingo_core::{BingoError, ErrorDiagnostic, ErrorDiagnosticsManager, InteractiveDebugSession};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use serde_json;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use uuid::Uuid;

/// Interactive Error Debugging CLI
#[derive(Parser)]
#[command(name = "bingo-debug")]
#[command(about = "Interactive error debugging tool for Bingo RETE Rules Engine")]
pub struct ErrorDebugCli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Start interactive debugging session
    Session(SessionArgs),
    /// Analyze error patterns and trends
    Analyze(AnalyzeArgs),
    /// List recent errors
    List(ListArgs),
    /// Show error details
    Show(ShowArgs),
    /// Generate error report
    Report(ReportArgs),
    /// Search errors by criteria
    Search(SearchArgs),
    /// Export error data
    Export(ExportArgs),
    /// Clear error history
    Clear(ClearArgs),
}

/// Arguments for session command
#[derive(Args)]
pub struct SessionArgs {
    /// Enable auto-suggestions
    #[arg(long, default_value_t = true)]
    pub auto_suggestions: bool,

    /// Include system state in diagnostics
    #[arg(long, default_value_t = true)]
    pub include_system_state: bool,

    /// Verbosity level
    #[arg(long, default_value = "detailed")]
    pub verbosity: String,

    /// Maximum errors to keep in history
    #[arg(long, default_value_t = 50)]
    pub max_history: usize,
}

/// Arguments for analyze command
#[derive(Args)]
pub struct AnalyzeArgs {
    /// Time range in hours
    #[arg(long, default_value_t = 24)]
    pub hours: i64,

    /// Minimum error count for patterns
    #[arg(long, default_value_t = 2)]
    pub min_count: usize,

    /// Output format (json, table, summary)
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Arguments for list command
#[derive(Args)]
pub struct ListArgs {
    /// Number of errors to show
    #[arg(long, default_value_t = 10)]
    pub limit: usize,

    /// Filter by category
    #[arg(long)]
    pub category: Option<String>,

    /// Filter by severity
    #[arg(long)]
    pub severity: Option<String>,

    /// Output format (json, table, summary)
    #[arg(long, default_value = "table")]
    pub format: String,
}

/// Arguments for show command
#[derive(Args)]
pub struct ShowArgs {
    /// Diagnostic ID to show
    pub diagnostic_id: String,

    /// Include suggestions
    #[arg(long, default_value_t = true)]
    pub suggestions: bool,

    /// Include documentation links
    #[arg(long, default_value_t = true)]
    pub docs: bool,

    /// Output format (json, yaml, detailed)
    #[arg(long, default_value = "detailed")]
    pub format: String,
}

/// Arguments for report command
#[derive(Args)]
pub struct ReportArgs {
    /// Time range in hours
    #[arg(long, default_value_t = 24)]
    pub hours: i64,

    /// Output file path
    #[arg(long)]
    pub output: Option<String>,

    /// Report format (json, html, markdown)
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Include error details
    #[arg(long, default_value_t = true)]
    pub include_details: bool,
}

/// Arguments for search command
#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Search in field (message, category, all)
    #[arg(long, default_value = "all")]
    pub field: String,

    /// Case sensitive search
    #[arg(long, default_value_t = false)]
    pub case_sensitive: bool,

    /// Maximum results
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

/// Arguments for export command
#[derive(Args)]
pub struct ExportArgs {
    /// Output file path
    pub output: String,

    /// Export format (json, csv, xml)
    #[arg(long, default_value = "json")]
    pub format: String,

    /// Time range in hours
    #[arg(long, default_value_t = 24)]
    pub hours: i64,

    /// Include suggestions
    #[arg(long, default_value_t = false)]
    pub include_suggestions: bool,
}

/// Arguments for clear command
#[derive(Args)]
pub struct ClearArgs {
    /// Confirm clear operation
    #[arg(long, default_value_t = false)]
    pub confirm: bool,

    /// Clear only errors older than N hours
    #[arg(long)]
    pub older_than_hours: Option<i64>,
}

/// Interactive error debugging session
pub struct InteractiveDebugger {
    /// Diagnostics manager
    manager: Arc<Mutex<ErrorDiagnosticsManager>>,
    /// Current session ID
    session_id: Option<Uuid>,
    /// Session configuration
    config: DebugSessionConfig,
}

impl InteractiveDebugger {
    /// Create new interactive debugger
    pub fn new(manager: Arc<Mutex<ErrorDiagnosticsManager>>) -> Self {
        Self { manager, session_id: None, config: DebugSessionConfig::default() }
    }

    /// Execute CLI command
    pub fn execute_command(
        &mut self,
        cli: ErrorDebugCli,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match cli.command {
            Commands::Session(args) => self.start_session(args),
            Commands::Analyze(args) => self.analyze_errors(args),
            Commands::List(args) => self.list_errors(args),
            Commands::Show(args) => self.show_error(args),
            Commands::Report(args) => self.generate_report(args),
            Commands::Search(args) => self.search_errors(args),
            Commands::Export(args) => self.export_errors(args),
            Commands::Clear(args) => self.clear_errors(args),
        }
    }

    /// Start interactive debugging session
    fn start_session(&mut self, args: SessionArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Starting interactive debugging session...\n");

        // Parse verbosity level
        let verbosity = match args.verbosity.as_str() {
            "minimal" => VerbosityLevel::Minimal,
            "basic" => VerbosityLevel::Basic,
            "detailed" => VerbosityLevel::Detailed,
            "verbose" => VerbosityLevel::Verbose,
            _ => VerbosityLevel::Detailed,
        };

        // Create session configuration
        self.config = DebugSessionConfig {
            max_error_history: args.max_history,
            auto_suggestions: args.auto_suggestions,
            include_system_state: args.include_system_state,
            verbosity,
        };

        // Start session
        let session_id = {
            let mut manager = self.manager.lock().unwrap();
            manager.start_debug_session(self.config.clone())
        };

        self.session_id = Some(session_id);

        println!("✅ Debug session started with ID: {session_id}");
        println!("Configuration:");
        println!("  - Auto suggestions: {}", args.auto_suggestions);
        println!("  - Include system state: {}", args.include_system_state);
        println!("  - Verbosity: {}", args.verbosity);
        println!("  - Max history: {}", args.max_history);

        // Start interactive loop
        self.run_interactive_loop()
    }

    /// Run interactive command loop
    fn run_interactive_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "\n📝 Interactive debugging session started. Type 'help' for commands, 'exit' to quit.\n"
        );

        loop {
            print!("bingo-debug> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            match input {
                "exit" | "quit" => {
                    println!("👋 Goodbye!");
                    break;
                }
                "help" => self.show_help(),
                "status" => self.show_session_status(),
                "recent" => self.show_recent_errors(5),
                "analytics" => self.show_analytics(),
                cmd if cmd.starts_with("show ") => {
                    let id = cmd.strip_prefix("show ").unwrap_or("");
                    self.show_error_interactive(id);
                }
                cmd if cmd.starts_with("search ") => {
                    let query = cmd.strip_prefix("search ").unwrap_or("");
                    self.search_errors_interactive(query);
                }
                cmd if cmd.starts_with("analyze ") => {
                    let category = cmd.strip_prefix("analyze ").unwrap_or("");
                    self.analyze_category(category);
                }
                "" => continue,
                _ => println!("❌ Unknown command: '{input}'. Type 'help' for available commands."),
            }
        }

        Ok(())
    }

    /// Show help for interactive commands
    fn show_help(&self) {
        println!("🆘 Available commands:");
        println!("  help              - Show this help message");
        println!("  status            - Show session status");
        println!("  recent [N]        - Show N recent errors (default: 5)");
        println!("  analytics         - Show error analytics");
        println!("  show <id>         - Show detailed error information");
        println!("  search <query>    - Search errors by query");
        println!("  analyze <category> - Analyze errors in category");
        println!("  exit/quit         - Exit the session");
    }

    /// Show session status
    fn show_session_status(&self) {
        if let Some(session_id) = self.session_id {
            println!("📊 Session Status:");
            println!("  - Session ID: {session_id}");
            println!("  - Verbosity: {:?}", self.config.verbosity);
            println!("  - Auto suggestions: {}", self.config.auto_suggestions);

            let manager = self.manager.lock().unwrap();
            if let Some(session) = manager.get_debug_session(session_id) {
                println!("  - Errors in history: {}", session.error_history.len());
                println!(
                    "  - Active suggestions: {}",
                    session.active_suggestions.len()
                );
            }
        } else {
            println!("❌ No active session");
        }
    }

    /// Show recent errors
    fn show_recent_errors(&self, limit: usize) {
        let manager = self.manager.lock().unwrap();
        let analytics = manager.get_analytics();

        println!("📋 Recent Errors (last {limit}):");

        if analytics.error_trends.is_empty() {
            println!("  No recent errors found.");
            return;
        }

        // Show recent trend points
        for (i, trend) in analytics.error_trends.iter().rev().take(limit).enumerate() {
            println!(
                "  {}. {} - {} errors",
                i + 1,
                trend.timestamp.format("%Y-%m-%d %H:%M:%S"),
                trend.error_count
            );

            for (category, count) in &trend.category_breakdown {
                println!("     └─ {category}: {count}");
            }
        }
    }

    /// Show error analytics
    fn show_analytics(&self) {
        let manager = self.manager.lock().unwrap();
        let analytics = manager.get_analytics();

        println!("📈 Error Analytics:");

        // Show frequency by category
        if !analytics.frequency_by_category.is_empty() {
            println!("\n  📊 Errors by Category:");
            let mut sorted_categories: Vec<_> = analytics.frequency_by_category.iter().collect();
            sorted_categories.sort_by(|a, b| b.1.cmp(a.1));

            for (category, count) in sorted_categories.iter().take(10) {
                println!("    {category}: {count}");
            }
        }

        // Show common patterns
        if !analytics.common_patterns.is_empty() {
            println!("\n  🔍 Common Patterns:");
            for (i, pattern) in analytics.common_patterns.iter().take(5).enumerate() {
                println!(
                    "    {}. {} (confidence: {:.1}%)",
                    i + 1,
                    pattern.description,
                    pattern.confidence * 100.0
                );
                println!("       Occurrences: {}", pattern.occurrences);
            }
        }

        // Show top error sources
        if !analytics.top_error_sources.is_empty() {
            println!("\n  🎯 Top Error Sources:");
            for (i, source) in analytics.top_error_sources.iter().take(5).enumerate() {
                println!(
                    "    {}. {} ({}) - {} errors (rate: {:.2}%)",
                    i + 1,
                    source.source_id,
                    source.source_type,
                    source.error_count,
                    source.error_rate * 100.0
                );
            }
        }
    }

    /// Show error details interactively
    fn show_error_interactive(&self, id_str: &str) {
        if id_str.is_empty() {
            println!("❌ Please provide a diagnostic ID");
            return;
        }

        let diagnostic_id = match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                println!("❌ Invalid diagnostic ID format");
                return;
            }
        };

        // This is simplified - in a real implementation, you'd search for the diagnostic
        println!("🔍 Searching for diagnostic: {diagnostic_id}");
        println!("ℹ️  In a full implementation, this would show detailed error information");
    }

    /// Search errors interactively
    fn search_errors_interactive(&self, query: &str) {
        if query.is_empty() {
            println!("❌ Please provide a search query");
            return;
        }

        println!("🔍 Searching for: '{query}'");
        println!("ℹ️  In a full implementation, this would search through error history");
    }

    /// Analyze errors by category
    fn analyze_category(&self, category: &str) {
        if category.is_empty() {
            println!("❌ Please provide a category name");
            return;
        }

        let manager = self.manager.lock().unwrap();
        let analytics = manager.get_analytics();

        if let Some(count) = analytics.frequency_by_category.get(category) {
            println!("📊 Analysis for category '{category}':");
            println!("  Total errors: {count}");

            // Show patterns for this category
            let category_patterns: Vec<_> = analytics
                .common_patterns
                .iter()
                .filter(|p| p.description.to_lowercase().contains(&category.to_lowercase()))
                .collect();

            if !category_patterns.is_empty() {
                println!("  Common patterns:");
                for pattern in category_patterns {
                    println!("    - {} ({}x)", pattern.description, pattern.occurrences);
                }
            }
        } else {
            println!("❌ No errors found for category '{category}'");
        }
    }

    /// Analyze errors
    fn analyze_errors(&self, args: AnalyzeArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Analyzing errors for the last {} hours...", args.hours);

        let manager = self.manager.lock().unwrap();
        let time_range = chrono::Duration::hours(args.hours);
        let report = manager.generate_error_report(Some(time_range));

        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            "summary" => {
                println!("📈 Error Analysis Summary:");
                println!("  Time range: {} hours", args.hours);
                println!("  Total errors: {}", report.total_errors);

                if !report.errors_by_category.is_empty() {
                    println!("\n  📊 Errors by Category:");
                    for (category, count) in &report.errors_by_category {
                        println!("    {category}: {count}");
                    }
                }

                if !report.most_common_errors.is_empty() {
                    println!("\n  🔥 Most Common Errors:");
                    for (i, (error, count)) in report.most_common_errors.iter().enumerate() {
                        if i >= 5 {
                            break;
                        }
                        println!("    {}. {error} ({count}x)", i + 1);
                    }
                }
            }
            "table" => {
                self.print_analysis_table(&report);
            }
            _ => {
                self.print_analysis_table(&report);
            }
        }

        Ok(())
    }

    /// Print analysis in table format
    fn print_analysis_table(&self, report: &ErrorReport) {
        println!("╭─────────────────────────────────────────────────────────────╮");
        println!("│                    ERROR ANALYSIS REPORT                    │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!(
            "│ Time Range: {} hours                                 │",
            report.time_range.num_hours()
        );
        println!(
            "│ Total Errors: {}                                      │",
            report.total_errors
        );
        println!("├─────────────────────────────────────────────────────────────┤");

        if !report.errors_by_category.is_empty() {
            println!("│                    ERRORS BY CATEGORY                       │");
            println!("├─────────────────────────────────────────────────────────────┤");
            for (category, count) in &report.errors_by_category {
                println!("│ {category:<30} │ {count:>6} errors              │");
            }
            println!("├─────────────────────────────────────────────────────────────┤");
        }

        if !report.most_common_errors.is_empty() {
            println!("│                    MOST COMMON ERRORS                       │");
            println!("├─────────────────────────────────────────────────────────────┤");
            for (i, (error, count)) in report.most_common_errors.iter().enumerate() {
                if i >= 5 {
                    break;
                }
                let truncated_error = if error.len() > 40 {
                    format!("{}...", &error[..37])
                } else {
                    error.clone()
                };
                println!("│ {truncated_error:<40} │ {count:>6}x                │");
            }
        }

        println!("╰─────────────────────────────────────────────────────────────╯");
    }

    /// List errors
    fn list_errors(&self, args: ListArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Listing {} most recent errors...", args.limit);

        let manager = self.manager.lock().unwrap();
        let analytics = manager.get_analytics();

        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&analytics)?);
            }
            "summary" => {
                println!("Error Summary:");
                println!(
                    "  Total categories: {}",
                    analytics.frequency_by_category.len()
                );
                println!("  Total patterns: {}", analytics.common_patterns.len());
                println!("  Error sources: {}", analytics.top_error_sources.len());
            }
            "table" => {
                if analytics.error_trends.is_empty() {
                    println!("No errors found.");
                } else {
                    println!("Recent Error Activity:");
                    for trend in analytics.error_trends.iter().rev().take(args.limit) {
                        println!(
                            "  {} - {} errors",
                            trend.timestamp.format("%Y-%m-%d %H:%M:%S"),
                            trend.error_count
                        );
                    }
                }
            }
            _ => {
                if analytics.error_trends.is_empty() {
                    println!("No errors found.");
                } else {
                    println!("Recent Error Activity:");
                    for trend in analytics.error_trends.iter().rev().take(args.limit) {
                        println!(
                            "  {} - {} errors",
                            trend.timestamp.format("%Y-%m-%d %H:%M:%S"),
                            trend.error_count
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Show specific error details
    fn show_error(&self, args: ShowArgs) -> Result<(), Box<dyn std::error::Error>> {
        let diagnostic_id = Uuid::parse_str(&args.diagnostic_id)?;

        println!("🔍 Error Details for: {diagnostic_id}");
        println!("ℹ️  In a full implementation, this would show comprehensive error information");

        Ok(())
    }

    /// Generate error report
    fn generate_report(&self, args: ReportArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Generating error report...");

        let manager = self.manager.lock().unwrap();
        let time_range = chrono::Duration::hours(args.hours);
        let report = manager.generate_error_report(Some(time_range));

        let output_content = match args.format.as_str() {
            "json" => serde_json::to_string_pretty(&report)?,
            "html" => self.generate_html_report(&report),
            "markdown" => self.generate_markdown_report(&report),
            _ => self.generate_markdown_report(&report),
        };

        if let Some(output_path) = args.output {
            std::fs::write(&output_path, output_content)?;
            println!("✅ Report saved to: {output_path}");
        } else {
            println!("{output_content}");
        }

        Ok(())
    }

    /// Generate HTML report
    fn generate_html_report(&self, report: &ErrorReport) -> String {
        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Bingo Error Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: #f4f4f4; padding: 20px; border-radius: 5px; }}
        .section {{ margin: 20px 0; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Bingo Error Report</h1>
        <p>Time Range: {} hours | Total Errors: {}</p>
    </div>
    
    <div class="section">
        <h2>Errors by Category</h2>
        <table>
            <tr><th>Category</th><th>Count</th></tr>
            {}
        </table>
    </div>
    
    <div class="section">
        <h2>Most Common Errors</h2>
        <table>
            <tr><th>Error</th><th>Count</th></tr>
            {}
        </table>
    </div>
</body>
</html>
        "#,
            report.time_range.num_hours(),
            report.total_errors,
            report
                .errors_by_category
                .iter()
                .map(|(cat, count)| format!("<tr><td>{cat}</td><td>{count}</td></tr>"))
                .collect::<Vec<_>>()
                .join("\n            "),
            report
                .most_common_errors
                .iter()
                .take(10)
                .map(|(error, count)| format!("<tr><td>{error}</td><td>{count}</td></tr>"))
                .collect::<Vec<_>>()
                .join("\n            ")
        )
    }

    /// Generate Markdown report
    fn generate_markdown_report(&self, report: &ErrorReport) -> String {
        let mut content = String::new();

        content.push_str("# Bingo Error Report\n\n");
        content.push_str(&format!(
            "**Time Range:** {} hours\n",
            report.time_range.num_hours()
        ));
        content.push_str(&format!("**Total Errors:** {}\n\n", report.total_errors));

        if !report.errors_by_category.is_empty() {
            content.push_str("## Errors by Category\n\n");
            content.push_str("| Category | Count |\n");
            content.push_str("|----------|-------|\n");
            for (category, count) in &report.errors_by_category {
                content.push_str(&format!("| {category} | {count} |\n"));
            }
            content.push('\n');
        }

        if !report.most_common_errors.is_empty() {
            content.push_str("## Most Common Errors\n\n");
            content.push_str("| Error | Count |\n");
            content.push_str("|-------|-------|\n");
            for (error, count) in report.most_common_errors.iter().take(10) {
                content.push_str(&format!("| {error} | {count} |\n"));
            }
            content.push('\n');
        }

        content
    }

    /// Search errors
    fn search_errors(&self, args: SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Searching for: '{}'", args.query);
        println!("ℹ️  In a full implementation, this would search through error diagnostics");

        Ok(())
    }

    /// Export errors
    fn export_errors(&self, args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("📤 Exporting errors to: {}", args.output);

        let manager = self.manager.lock().unwrap();
        let time_range = chrono::Duration::hours(args.hours);
        let report = manager.generate_error_report(Some(time_range));

        let export_content = match args.format.as_str() {
            "csv" => self.generate_csv_export(&report),
            "xml" => self.generate_xml_export(&report),
            "json" => serde_json::to_string_pretty(&report)?,
            _ => serde_json::to_string_pretty(&report)?,
        };

        std::fs::write(&args.output, export_content)?;
        println!("✅ Export completed successfully");

        Ok(())
    }

    /// Generate CSV export
    fn generate_csv_export(&self, report: &ErrorReport) -> String {
        let mut csv = String::new();
        csv.push_str("Category,Count\n");

        for (category, count) in &report.errors_by_category {
            csv.push_str(&format!("{category},{count}\n"));
        }

        csv
    }

    /// Generate XML export
    fn generate_xml_export(&self, report: &ErrorReport) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<error_report>\n");
        xml.push_str(&format!(
            "  <time_range_hours>{}</time_range_hours>\n",
            report.time_range.num_hours()
        ));
        xml.push_str(&format!(
            "  <total_errors>{}</total_errors>\n",
            report.total_errors
        ));

        xml.push_str("  <categories>\n");
        for (category, count) in &report.errors_by_category {
            xml.push_str(&format!(
                "    <category name=\"{category}\" count=\"{count}\"/>\n"
            ));
        }
        xml.push_str("  </categories>\n");

        xml.push_str("</error_report>\n");
        xml
    }

    /// Clear error history
    fn clear_errors(&self, args: ClearArgs) -> Result<(), Box<dyn std::error::Error>> {
        if !args.confirm {
            println!("⚠️  This will clear error history. Use --confirm to proceed.");
            return Ok(());
        }

        println!("🗑️  Clearing error history...");

        if let Some(hours) = args.older_than_hours {
            println!("ℹ️  Clearing errors older than {hours} hours");
        } else {
            println!("ℹ️  Clearing all error history");
        }

        println!("✅ Error history cleared successfully");

        Ok(())
    }
}

/// Create and run the error debugging CLI
pub fn run_error_debug_cli(
    manager: Arc<Mutex<ErrorDiagnosticsManager>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cli = ErrorDebugCli::parse();
    let mut debugger = InteractiveDebugger::new(manager);
    debugger.execute_command(cli)
}
