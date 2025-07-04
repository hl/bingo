//! JSON-based Integration Test Framework for Bingo RETE Rules Engine
//!
//! This framework allows defining test cases as JSON files with inputs and expected outputs,
//! enabling comprehensive validation of the API functionality including streaming, caching,
//! calculator DSL, multi-rule processing, and operational features.

use axum_test::TestServer;
use bingo_api::{create_app, types::*};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Test input structure matching the API request format
#[derive(Debug, Deserialize)]
struct TestInput {
    facts: Vec<ApiFact>,
    rules: Option<Vec<ApiRule>>,
    ruleset_id: Option<String>,
    response_format: Option<ResponseFormat>,
    streaming_config: Option<StreamingConfig>,
    #[serde(default)]
    headers: HashMap<String, String>,
}

/// Expected output validation rules
#[derive(Debug, Deserialize)]
struct ExpectedOutput {
    validation_rules: ValidationRules,
    #[serde(default)]
    exact_matches: HashMap<String, Value>,
    #[serde(default)]
    range_validations: HashMap<String, RangeValidation>,
    #[serde(default)]
    array_validations: HashMap<String, ArrayValidation>,
    #[serde(default, rename = "pattern_validations")]
    _pattern_validations: HashMap<String, String>,
}

/// Validation rules for response structure and metadata
#[derive(Debug, Deserialize)]
struct ValidationRules {
    status_code: u16,
    content_type: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    response_structure: HashMap<String, String>,
    #[serde(default, rename = "allow_additional_fields")]
    _allow_additional_fields: bool,
}

/// Range validation for numeric values
#[derive(Debug, Deserialize)]
struct RangeValidation {
    min: f64,
    max: f64,
    #[serde(default)]
    inclusive: bool,
}

/// Array validation rules
#[derive(Debug, Deserialize)]
struct ArrayValidation {
    #[serde(default)]
    min_length: Option<usize>,
    #[serde(default)]
    max_length: Option<usize>,
    #[serde(default, rename = "item_structure")]
    _item_structure: Option<HashMap<String, String>>,
}

/// Test metadata and configuration
#[derive(Debug, Deserialize)]
struct TestMetadata {
    #[serde(rename = "test_name")]
    _test_name: String,
    #[serde(rename = "description")]
    _description: String,
    #[serde(rename = "category")]
    _category: String,
    #[serde(rename = "tags")]
    _tags: Vec<String>,
    timeout_ms: u64,
    expected_duration_ms: u64,
    skip_reason: Option<String>,
    #[serde(default, rename = "prerequisites")]
    _prerequisites: Vec<String>,
    #[serde(rename = "author")]
    _author: String,
    #[serde(rename = "created_at")]
    _created_at: String,
}

/// Test execution result
#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub category: String,
    pub passed: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub performance_notes: Vec<String>,
}

/// JSON Test Framework Runner
pub struct JsonTestFramework {
    server: TestServer,
    test_directory: PathBuf,
    verbose: bool,
    skip_performance: bool,
}

impl JsonTestFramework {
    /// Create a new test framework instance
    pub async fn new() -> Self {
        let app = create_app().await.expect("Failed to create app");
        let server = TestServer::new(app).expect("Failed to create test server");
        let test_directory = PathBuf::from("crates/bingo-api/tests/json-tests");

        Self { server, test_directory, verbose: false, skip_performance: false }
    }

    /// Set verbose output mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Skip performance validations
    pub fn skip_performance(mut self, skip: bool) -> Self {
        self.skip_performance = skip;
        self
    }

    /// Run all tests in all categories
    pub async fn run_all_tests(&self) -> Vec<TestResult> {
        let mut results = Vec::new();

        for category in self.get_test_categories() {
            let mut category_results = self.run_category_tests(&category).await;
            results.append(&mut category_results);
        }

        results
    }

    /// Run tests for a specific category
    pub async fn run_category_tests(&self, category: &str) -> Vec<TestResult> {
        let category_path = self.test_directory.join(category);
        if !category_path.exists() {
            println!("⚠️  Category directory not found: {}", category);
            return Vec::new();
        }

        let mut results = Vec::new();
        let test_cases = self.discover_test_cases(&category_path);

        println!(
            "🧪 Running {} tests in category: {}",
            test_cases.len(),
            category
        );

        for test_case in test_cases {
            let result = self.run_single_test(test_case, category).await;
            if self.verbose {
                self.print_test_result(&result);
            }
            results.push(result);
        }

        results
    }

    /// Run a single named test
    pub async fn run_named_test(&self, test_name: &str) -> Option<TestResult> {
        for category in self.get_test_categories() {
            let category_path = self.test_directory.join(&category);
            let test_cases = self.discover_test_cases(&category_path);

            for test_case in test_cases {
                if test_case.file_stem().unwrap().to_str().unwrap().ends_with(test_name) {
                    return Some(self.run_single_test(test_case, &category).await);
                }
            }
        }
        None
    }

    /// Discover all test categories
    fn get_test_categories(&self) -> Vec<String> {
        let mut categories = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.test_directory) {
            for entry in entries.flatten() {
                if entry.file_type().unwrap().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        categories.push(name.to_string());
                    }
                }
            }
        }

        categories.sort();
        categories
    }

    /// Discover test cases in a category directory
    fn discover_test_cases(&self, category_path: &Path) -> Vec<PathBuf> {
        let mut test_cases = Vec::new();

        if let Ok(entries) = fs::read_dir(category_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json")
                    && path.file_name().unwrap().to_str().unwrap().ends_with("_input.json")
                {
                    test_cases.push(path);
                }
            }
        }

        test_cases.sort();
        test_cases
    }

    /// Run a single test case
    async fn run_single_test(&self, input_path: PathBuf, category: &str) -> TestResult {
        let test_name = input_path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix("_input")
            .unwrap()
            .to_string();

        let start_time = Instant::now();

        // Load test files
        let (input, expected, metadata) = match self.load_test_files(&input_path) {
            Ok(files) => files,
            Err(e) => {
                return TestResult {
                    test_name,
                    category: category.to_string(),
                    passed: false,
                    duration: start_time.elapsed(),
                    error_message: Some(format!("Failed to load test files: {}", e)),
                    performance_notes: Vec::new(),
                };
            }
        };

        // Check if test should be skipped
        if let Some(skip_reason) = &metadata.skip_reason {
            return TestResult {
                test_name,
                category: category.to_string(),
                passed: true, // Skipped tests are considered passing
                duration: start_time.elapsed(),
                error_message: Some(format!("SKIPPED: {}", skip_reason)),
                performance_notes: Vec::new(),
            };
        }

        // Execute the test
        match self.execute_test(input, expected, metadata).await {
            Ok((passed, performance_notes)) => TestResult {
                test_name,
                category: category.to_string(),
                passed,
                duration: start_time.elapsed(),
                error_message: None,
                performance_notes,
            },
            Err(e) => TestResult {
                test_name,
                category: category.to_string(),
                passed: false,
                duration: start_time.elapsed(),
                error_message: Some(e),
                performance_notes: Vec::new(),
            },
        }
    }

    /// Load test input, expected output, and metadata files
    fn load_test_files(
        &self,
        input_path: &Path,
    ) -> Result<(TestInput, ExpectedOutput, TestMetadata), String> {
        let base_path = input_path
            .with_extension("")
            .to_string_lossy()
            .strip_suffix("_input")
            .unwrap()
            .to_string();

        let expected_path = format!("{}_expected.json", base_path);
        let metadata_path = format!("{}_metadata.json", base_path);

        let input_content = fs::read_to_string(input_path)
            .map_err(|e| format!("Failed to read input file: {}", e))?;
        let expected_content = fs::read_to_string(&expected_path)
            .map_err(|e| format!("Failed to read expected file: {}", e))?;
        let metadata_content = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read metadata file: {}", e))?;

        let input: TestInput = serde_json::from_str(&input_content)
            .map_err(|e| format!("Failed to parse input JSON: {}", e))?;
        let expected: ExpectedOutput = serde_json::from_str(&expected_content)
            .map_err(|e| format!("Failed to parse expected JSON: {}", e))?;
        let metadata: TestMetadata = serde_json::from_str(&metadata_content)
            .map_err(|e| format!("Failed to parse metadata JSON: {}", e))?;

        Ok((input, expected, metadata))
    }

    /// Execute a test case against the API
    async fn execute_test(
        &self,
        input: TestInput,
        expected: ExpectedOutput,
        metadata: TestMetadata,
    ) -> Result<(bool, Vec<String>), String> {
        let mut performance_notes = Vec::new();
        let start_time = Instant::now();

        // Create the API request
        let request = EvaluateRequest {
            facts: input.facts,
            rules: input.rules,
            ruleset_id: input.ruleset_id,
            response_format: input.response_format,
            streaming_config: input.streaming_config,
        };

        // Build the HTTP request with headers
        let mut http_request = self.server.post("/evaluate").json(&request);
        for (key, value) in &input.headers {
            http_request = http_request.add_header(key.as_str(), value.as_str());
        }

        // Execute the request with timeout
        let response =
            tokio::time::timeout(Duration::from_millis(metadata.timeout_ms), http_request)
                .await
                .map_err(|_| format!("Test timed out after {}ms", metadata.timeout_ms))?;

        let request_duration = start_time.elapsed();

        // Validate response status
        if response.status_code() != expected.validation_rules.status_code {
            return Err(format!(
                "Status code mismatch: expected {}, got {}",
                expected.validation_rules.status_code,
                response.status_code()
            ));
        }

        // Validate content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        if !content_type.contains(&expected.validation_rules.content_type) {
            return Err(format!(
                "Content type mismatch: expected {}, got {}",
                expected.validation_rules.content_type, content_type
            ));
        }

        // Validate headers
        for (header_name, expected_value) in &expected.validation_rules.headers {
            let actual_value =
                response.headers().get(header_name).and_then(|h| h.to_str().ok()).unwrap_or("");

            match expected_value.as_str() {
                "exists" => {
                    if actual_value.is_empty() {
                        return Err(format!("Header {} is missing", header_name));
                    }
                }
                "numeric" => {
                    if actual_value.parse::<f64>().is_err() {
                        return Err(format!(
                            "Header {} is not numeric: {}",
                            header_name, actual_value
                        ));
                    }
                }
                _ => {
                    if actual_value != expected_value {
                        return Err(format!(
                            "Header {} mismatch: expected {}, got {}",
                            header_name, expected_value, actual_value
                        ));
                    }
                }
            }
        }

        // Parse response body based on content type
        let response_json: Value = if content_type.contains("application/x-ndjson") {
            // Handle NDJSON streaming response
            let body = response.text();
            self.parse_ndjson_response(&body)?
        } else {
            // Handle regular JSON response
            response.json()
        };

        // Validate response structure
        self.validate_response_structure(
            &response_json,
            &expected.validation_rules.response_structure,
        )?;

        // Validate exact matches
        for (field_path, expected_value) in &expected.exact_matches {
            let actual_value = self.get_json_value_by_path(&response_json, field_path)?;
            if *actual_value != *expected_value {
                return Err(format!(
                    "Exact match failed for {}: expected {:?}, got {:?}",
                    field_path, expected_value, actual_value
                ));
            }
        }

        // Validate ranges
        for (field_path, range) in &expected.range_validations {
            let actual_value = self.get_json_value_by_path(&response_json, field_path)?;
            if let Some(num) = actual_value.as_f64() {
                let in_range = if range.inclusive {
                    num >= range.min && num <= range.max
                } else {
                    num > range.min && num < range.max
                };
                if !in_range {
                    return Err(format!(
                        "Range validation failed for {}: {} not in range [{}, {}]",
                        field_path, num, range.min, range.max
                    ));
                }
            } else {
                return Err(format!(
                    "Field {} is not numeric for range validation",
                    field_path
                ));
            }
        }

        // Validate arrays
        for (field_path, array_validation) in &expected.array_validations {
            let actual_value = self.get_json_value_by_path(&response_json, field_path)?;
            if let Some(array) = actual_value.as_array() {
                if let Some(min_len) = array_validation.min_length {
                    if array.len() < min_len {
                        return Err(format!(
                            "Array {} too short: {} < {}",
                            field_path,
                            array.len(),
                            min_len
                        ));
                    }
                }
                if let Some(max_len) = array_validation.max_length {
                    if array.len() > max_len {
                        return Err(format!(
                            "Array {} too long: {} > {}",
                            field_path,
                            array.len(),
                            max_len
                        ));
                    }
                }
            } else {
                return Err(format!("Field {} is not an array", field_path));
            }
        }

        // Performance validation
        if !self.skip_performance {
            if request_duration.as_millis() > metadata.expected_duration_ms as u128 {
                performance_notes.push(format!(
                    "⚠️  Performance: Request took {}ms, expected <{}ms",
                    request_duration.as_millis(),
                    metadata.expected_duration_ms
                ));
            } else {
                performance_notes.push(format!(
                    "✅ Performance: Request completed in {}ms",
                    request_duration.as_millis()
                ));
            }
        }

        Ok((true, performance_notes))
    }

    /// Parse NDJSON streaming response into a single JSON object
    fn parse_ndjson_response(&self, body: &str) -> Result<Value, String> {
        let lines: Vec<&str> = body.trim().split('\n').collect();
        let mut results = Vec::new();
        let mut progress_updates = Vec::new();
        let mut final_summary = None;

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            let json_line: Value = serde_json::from_str(line)
                .map_err(|e| format!("Failed to parse NDJSON line: {}", e))?;

            if let Some(result_type) = json_line.get("type").and_then(|t| t.as_str()) {
                match result_type {
                    "result" => results.push(json_line),
                    "progress" | "incremental_progress" => progress_updates.push(json_line),
                    "summary" => final_summary = Some(json_line),
                    _ => {} // Ignore unknown types
                }
            } else {
                // Assume it's a result if no type specified
                results.push(json_line);
            }
        }

        // Construct a unified response object
        Ok(json!({
            "results": results,
            "progress_updates": progress_updates,
            "summary": final_summary,
            "streaming": true
        }))
    }

    /// Validate the structure of the response JSON
    fn validate_response_structure(
        &self,
        response: &Value,
        expected_structure: &HashMap<String, String>,
    ) -> Result<(), String> {
        for (field_name, expected_type) in expected_structure {
            let field_value = response
                .get(field_name)
                .ok_or_else(|| format!("Required field '{}' missing from response", field_name))?;

            let actual_type = self.get_json_type(field_value);
            if actual_type != *expected_type {
                return Err(format!(
                    "Type mismatch for field '{}': expected {}, got {}",
                    field_name, expected_type, actual_type
                ));
            }
        }
        Ok(())
    }

    /// Get JSON value by dot-separated path
    fn get_json_value_by_path<'a>(&self, json: &'a Value, path: &str) -> Result<&'a Value, String> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;

        for part in parts {
            current =
                current.get(part).ok_or_else(|| format!("Path '{}' not found in JSON", path))?;
        }

        Ok(current)
    }

    /// Get the type name of a JSON value
    fn get_json_type(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }

    /// Print detailed test result
    fn print_test_result(&self, result: &TestResult) {
        let status = if result.passed { "✅" } else { "❌" };
        println!(
            "{} {} ({}) - {}ms",
            status,
            result.test_name,
            result.category,
            result.duration.as_millis()
        );

        if let Some(error) = &result.error_message {
            println!("   Error: {}", error);
        }

        for note in &result.performance_notes {
            println!("   {}", note);
        }
    }

    /// Print test summary
    pub fn print_summary(&self, results: &[TestResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        println!("\n📊 Test Summary:");
        println!("   Total: {}", total);
        println!("   Passed: {} ✅", passed);
        println!("   Failed: {} ❌", failed);

        if failed > 0 {
            println!("\n❌ Failed Tests:");
            for result in results.iter().filter(|r| !r.passed) {
                println!("   {} ({})", result.test_name, result.category);
                if let Some(error) = &result.error_message {
                    println!("     Error: {}", error);
                }
            }
        }

        let avg_duration: f64 =
            results.iter().map(|r| r.duration.as_millis() as f64).sum::<f64>() / total as f64;
        println!("\n⏱️  Average test duration: {:.1}ms", avg_duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_framework_initialization() {
        let framework = JsonTestFramework::new().await;
        assert!(!framework.verbose);
        assert!(!framework.skip_performance);
    }

    #[tokio::test]
    async fn test_framework_configuration() {
        let framework = JsonTestFramework::new().await.with_verbose(true).skip_performance(true);

        assert!(framework.verbose);
        assert!(framework.skip_performance);
    }
}
