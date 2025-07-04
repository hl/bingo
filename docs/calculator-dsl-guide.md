# Calculator DSL and Business Logic Extension Guide

## Overview

The Bingo Rules Engine provides a powerful Calculator Domain-Specific Language (DSL) for implementing complex business logic in a declarative, maintainable way. This guide covers calculator development, integration patterns, and best practices for extending the engine with custom business logic.

## Calculator Architecture

### Core Concepts

Calculators in Bingo are modular, reusable components that:
- Accept typed inputs through field mappings
- Execute business logic computations
- Return results that can be assigned to fact fields
- Provide error handling and validation
- Support caching for performance optimization

### Calculator Registry System

```rust
// From crates/bingo-core/src/calculator_integration.rs
use crate::types::CalculatorInputs;
use anyhow::Result;
use std::collections::HashMap;

pub trait Calculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String>;
    fn validate_inputs(&self, inputs: &CalculatorInputs) -> Result<()>;
    fn get_required_fields(&self) -> Vec<String>;
    fn get_optional_fields(&self) -> Vec<String>;
}

#[derive(Debug)]
pub struct CalculatorRegistry {
    calculators: HashMap<String, Box<dyn Calculator>>,
}

impl CalculatorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            calculators: HashMap::new(),
        };
        
        // Register built-in calculators
        registry.register_builtin_calculators();
        registry
    }
    
    pub fn register_calculator(&mut self, name: String, calculator: Box<dyn Calculator>) {
        self.calculators.insert(name, calculator);
    }
    
    pub fn get_calculator(&self, name: &str) -> Option<&dyn Calculator> {
        self.calculators.get(name).map(|c| c.as_ref())
    }
}
```

## Built-in Calculators

### 1. Threshold Check Calculator

Validates that numeric values fall within specified thresholds.

```rust
// Example usage in rules
{
  "action_type": {
    "CallCalculator": {
      "calculator_name": "threshold_check",
      "input_mapping": {
        "value": "salary",
        "min_threshold": "40000",
        "max_threshold": "200000"
      },
      "output_field": "salary_valid"
    }
  }
}
```

**Input Fields:**
- `value` (required): Numeric value to check
- `min_threshold` (optional): Minimum allowed value
- `max_threshold` (optional): Maximum allowed value

**Output:**
- Returns "true" if within thresholds, "false" otherwise
- Includes validation error details if thresholds are violated

### 2. Limit Validation Calculator

Ensures values don't exceed defined limits with configurable tolerance.

```rust
// Example usage
{
  "action_type": {
    "CallCalculator": {
      "calculator_name": "limit_validate", 
      "input_mapping": {
        "current_value": "overtime_hours",
        "limit": "max_overtime_per_week",
        "tolerance_percent": "10"
      },
      "output_field": "overtime_compliance"
    }
  }
}
```

**Input Fields:**
- `current_value` (required): Value to validate
- `limit` (required): Maximum allowed limit
- `tolerance_percent` (optional): Percentage tolerance (default: 0)

**Output:**
- Returns compliance status and any overage amount

### 3. Percentage Calculator

Applies percentage calculations with support for different calculation modes.

```rust
// Example: Calculate tax amount
{
  "action_type": {
    "CallCalculator": {
      "calculator_name": "percentage_calculator",
      "input_mapping": {
        "base_amount": "gross_salary",
        "percentage": "tax_rate",
        "mode": "multiply"
      },
      "output_field": "tax_amount"
    }
  }
}
```

**Calculation Modes:**
- `multiply`: base_amount * (percentage / 100)
- `add`: base_amount + (base_amount * percentage / 100)
- `subtract`: base_amount - (base_amount * percentage / 100)

### 4. Tiered Rate Calculator

Applies progressive rate calculations commonly used in tax and commission systems.

```rust
// Example: Progressive tax calculation
{
  "action_type": {
    "CallCalculator": {
      "calculator_name": "tiered_rate",
      "input_mapping": {
        "base_amount": "annual_income",
        "tier_config": "tax_brackets_2024"
      },
      "output_field": "total_tax"
    }
  }
}
```

**Tier Configuration Format:**
```json
{
  "tiers": [
    {"threshold": 0, "rate": 0.10},
    {"threshold": 50000, "rate": 0.22},
    {"threshold": 100000, "rate": 0.32},
    {"threshold": 200000, "rate": 0.37}
  ]
}
```

## Custom Calculator Development

### Basic Calculator Implementation

```rust
use crate::calculator_integration::Calculator;
use crate::types::CalculatorInputs;
use anyhow::{Result, anyhow};

pub struct CustomBusinessCalculator {
    name: String,
    description: String,
}

impl CustomBusinessCalculator {
    pub fn new() -> Self {
        Self {
            name: "custom_business_calculator".to_string(),
            description: "Custom business logic implementation".to_string(),
        }
    }
}

impl Calculator for CustomBusinessCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Validate inputs first
        self.validate_inputs(inputs)?;
        
        // Extract required fields
        let employee_level = inputs.fields.get("employee_level")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow!("Missing employee_level"))?;
            
        let base_salary = inputs.fields.get("base_salary")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing or invalid base_salary"))?;
        
        let years_experience = inputs.fields.get("years_experience")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        
        // Business logic implementation
        let bonus_multiplier = match employee_level.as_str() {
            "junior" => 0.05 + (years_experience * 0.01),
            "senior" => 0.10 + (years_experience * 0.015),
            "lead" => 0.15 + (years_experience * 0.02),
            "principal" => 0.20 + (years_experience * 0.025),
            _ => return Err(anyhow!("Invalid employee level: {}", employee_level)),
        };
        
        let bonus_amount = base_salary * bonus_multiplier;
        
        // Return formatted result
        Ok(format!("{:.2}", bonus_amount))
    }
    
    fn validate_inputs(&self, inputs: &CalculatorInputs) -> Result<()> {
        let required_fields = self.get_required_fields();
        
        for field in &required_fields {
            if !inputs.fields.contains_key(field) {
                return Err(anyhow!("Missing required field: {}", field));
            }
        }
        
        // Type validation
        if let Some(salary) = inputs.fields.get("base_salary") {
            if salary.as_f64().is_none() {
                return Err(anyhow!("base_salary must be numeric"));
            }
        }
        
        // Business rule validation
        if let Some(experience) = inputs.fields.get("years_experience") {
            if let Some(exp_val) = experience.as_f64() {
                if exp_val < 0.0 || exp_val > 50.0 {
                    return Err(anyhow!("years_experience must be between 0 and 50"));
                }
            }
        }
        
        Ok(())
    }
    
    fn get_required_fields(&self) -> Vec<String> {
        vec![
            "employee_level".to_string(),
            "base_salary".to_string(),
        ]
    }
    
    fn get_optional_fields(&self) -> Vec<String> {
        vec![
            "years_experience".to_string(),
            "performance_rating".to_string(),
        ]
    }
}
```

### Advanced Calculator with Configuration

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollCalculatorConfig {
    pub overtime_multiplier: f64,
    pub double_time_threshold: f64,
    pub holiday_multiplier: f64,
    pub tax_rates: HashMap<String, f64>,
    pub deduction_limits: HashMap<String, f64>,
}

pub struct PayrollCalculator {
    config: PayrollCalculatorConfig,
}

impl PayrollCalculator {
    pub fn new(config: PayrollCalculatorConfig) -> Self {
        Self { config }
    }
    
    pub fn from_file(config_path: &str) -> Result<Self> {
        let config_content = std::fs::read_to_string(config_path)?;
        let config: PayrollCalculatorConfig = toml::from_str(&config_content)?;
        Ok(Self::new(config))
    }
}

impl Calculator for PayrollCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        let regular_hours = inputs.fields.get("regular_hours")
            .and_then(|v| v.as_f64())
            .unwrap_or(40.0);
            
        let overtime_hours = inputs.fields.get("overtime_hours")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
            
        let holiday_hours = inputs.fields.get("holiday_hours")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
            
        let hourly_rate = inputs.fields.get("hourly_rate")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing hourly_rate"))?;
        
        // Calculate gross pay
        let regular_pay = regular_hours * hourly_rate;
        let overtime_pay = overtime_hours * hourly_rate * self.config.overtime_multiplier;
        let holiday_pay = holiday_hours * hourly_rate * self.config.holiday_multiplier;
        
        // Handle double-time for excessive overtime
        let double_time_hours = if overtime_hours > self.config.double_time_threshold {
            overtime_hours - self.config.double_time_threshold
        } else {
            0.0
        };
        let double_time_pay = double_time_hours * hourly_rate * 2.0;
        
        let gross_pay = regular_pay + overtime_pay + holiday_pay + double_time_pay;
        
        // Calculate deductions
        let state = inputs.fields.get("state")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "default".to_string());
            
        let tax_rate = self.config.tax_rates.get(&state).unwrap_or(&0.0);
        let tax_amount = gross_pay * tax_rate;
        
        // Calculate net pay
        let net_pay = gross_pay - tax_amount;
        
        // Return detailed result
        let result = serde_json::json!({
            "gross_pay": gross_pay,
            "tax_amount": tax_amount,
            "net_pay": net_pay,
            "regular_pay": regular_pay,
            "overtime_pay": overtime_pay,
            "holiday_pay": holiday_pay,
            "double_time_pay": double_time_pay
        });
        
        Ok(result.to_string())
    }
    
    fn validate_inputs(&self, inputs: &CalculatorInputs) -> Result<()> {
        // Comprehensive validation logic
        if let Some(rate) = inputs.fields.get("hourly_rate") {
            if let Some(rate_val) = rate.as_f64() {
                if rate_val <= 0.0 {
                    return Err(anyhow!("hourly_rate must be positive"));
                }
                if rate_val > 1000.0 {
                    return Err(anyhow!("hourly_rate exceeds maximum allowed"));
                }
            }
        }
        
        Ok(())
    }
    
    fn get_required_fields(&self) -> Vec<String> {
        vec!["hourly_rate".to_string()]
    }
    
    fn get_optional_fields(&self) -> Vec<String> {
        vec![
            "regular_hours".to_string(),
            "overtime_hours".to_string(),
            "holiday_hours".to_string(),
            "state".to_string(),
        ]
    }
}
```

## Business Logic Patterns

### 1. Conditional Logic Pattern

```rust
impl Calculator for ConditionalBonusCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        let performance_rating = inputs.fields.get("performance_rating")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow!("Missing performance_rating"))?;
            
        let base_salary = inputs.fields.get("base_salary")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing base_salary"))?;
            
        let department = inputs.fields.get("department")
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "general".to_string());
        
        // Multi-factor conditional logic
        let bonus_percentage = match (performance_rating.as_str(), department.as_str()) {
            ("excellent", "sales") => 0.20,
            ("excellent", "engineering") => 0.15,
            ("excellent", _) => 0.12,
            ("good", "sales") => 0.15,
            ("good", "engineering") => 0.10,
            ("good", _) => 0.08,
            ("satisfactory", _) => 0.05,
            ("needs_improvement", _) => 0.0,
            _ => return Err(anyhow!("Invalid performance rating: {}", performance_rating)),
        };
        
        // Apply department-specific multipliers
        let dept_multiplier = match department.as_str() {
            "sales" => 1.1,      // 10% boost for sales
            "engineering" => 1.05, // 5% boost for engineering
            _ => 1.0,
        };
        
        let bonus_amount = base_salary * bonus_percentage * dept_multiplier;
        Ok(format!("{:.2}", bonus_amount))
    }
}
```

### 2. Time-based Calculation Pattern

```rust
use chrono::{DateTime, Utc, Datelike};

impl Calculator for TimeBasedCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        let hire_date_str = inputs.fields.get("hire_date")
            .and_then(|v| v.as_string())
            .ok_or_else(|| anyhow!("Missing hire_date"))?;
            
        let hire_date = DateTime::parse_from_rfc3339(&hire_date_str)?;
        let current_date = Utc::now();
        
        // Calculate tenure
        let years_employed = current_date.year() - hire_date.year();
        let months_employed = (years_employed * 12) + 
            (current_date.month() as i32 - hire_date.month() as i32);
        
        // Time-based benefit calculation
        let vacation_days = match years_employed {
            0..=1 => 10,
            2..=4 => 15,
            5..=9 => 20,
            10..=14 => 25,
            _ => 30,
        };
        
        // Anniversary bonus
        let anniversary_bonus = if current_date.month() == hire_date.month() &&
                                  current_date.day() == hire_date.day() {
            years_employed as f64 * 100.0 // $100 per year of service
        } else {
            0.0
        };
        
        let result = serde_json::json!({
            "years_employed": years_employed,
            "months_employed": months_employed,
            "vacation_days": vacation_days,
            "anniversary_bonus": anniversary_bonus
        });
        
        Ok(result.to_string())
    }
}
```

### 3. Aggregation and Rollup Pattern

```rust
impl Calculator for AggregationCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Extract array of values for aggregation
        let values_array = inputs.fields.get("monthly_sales")
            .and_then(|v| match v {
                FactValue::Array(arr) => Some(arr),
                _ => None,
            })
            .ok_or_else(|| anyhow!("Missing or invalid monthly_sales array"))?;
        
        // Convert to numeric values
        let numeric_values: Result<Vec<f64>, _> = values_array
            .iter()
            .map(|v| v.as_f64().ok_or_else(|| anyhow!("Non-numeric value in array")))
            .collect();
        
        let values = numeric_values?;
        
        if values.is_empty() {
            return Ok("0".to_string());
        }
        
        // Calculate various aggregations
        let total: f64 = values.iter().sum();
        let average = total / values.len() as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        // Calculate variance and standard deviation
        let variance = values.iter()
            .map(|&x| (x - average).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        // Performance rating based on consistency
        let performance_score = if std_dev / average < 0.2 {
            "consistent"
        } else if std_dev / average < 0.4 {
            "variable"
        } else {
            "inconsistent"
        };
        
        let result = serde_json::json!({
            "total": total,
            "average": average,
            "min": min,
            "max": max,
            "std_dev": std_dev,
            "performance_score": performance_score
        });
        
        Ok(result.to_string())
    }
}
```

## Integration Patterns

### 1. Calculator Chaining

Chain multiple calculators together for complex workflows:

```json
{
  "name": "Employee Compensation Review",
  "conditions": [
    {
      "field": "review_date",
      "operator": "Equal",
      "value": "2024-01-01"
    }
  ],
  "actions": [
    {
      "action_type": {
        "CallCalculator": {
          "calculator_name": "time_based_calculator",
          "input_mapping": {
            "hire_date": "hire_date"
          },
          "output_field": "tenure_info"
        }
      }
    },
    {
      "action_type": {
        "CallCalculator": {
          "calculator_name": "performance_bonus",
          "input_mapping": {
            "base_salary": "base_salary",
            "performance_rating": "performance_rating",
            "years_employed": "tenure_info.years_employed"
          },
          "output_field": "bonus_amount"
        }
      }
    },
    {
      "action_type": {
        "CallCalculator": {
          "calculator_name": "tax_calculator",
          "input_mapping": {
            "gross_amount": "bonus_amount",
            "employee_state": "state",
            "tax_year": "2024"
          },
          "output_field": "net_bonus"
        }
      }
    }
  ]
}
```

### 2. Conditional Calculator Selection

Use rule conditions to select appropriate calculators:

```json
{
  "name": "Region-Specific Payroll",
  "conditions": [
    {
      "field": "employee_region",
      "operator": "Equal",
      "value": "US"
    }
  ],
  "actions": [
    {
      "action_type": {
        "CallCalculator": {
          "calculator_name": "us_payroll_calculator",
          "input_mapping": {
            "gross_pay": "gross_pay",
            "state": "state",
            "filing_status": "filing_status"
          },
          "output_field": "us_tax_calculation"
        }
      }
    }
  ]
}
```

```json
{
  "name": "Region-Specific Payroll - EU",
  "conditions": [
    {
      "field": "employee_region",
      "operator": "Equal",
      "value": "EU"
    }
  ],
  "actions": [
    {
      "action_type": {
        "CallCalculator": {
          "calculator_name": "eu_payroll_calculator",
          "input_mapping": {
            "gross_pay": "gross_pay",
            "country": "country",
            "tax_class": "tax_class"
          },
          "output_field": "eu_tax_calculation"
        }
      }
    }
  ]
}
```

### 3. Error Handling and Fallback

```rust
impl Calculator for RobustCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Primary calculation attempt
        match self.primary_calculation(inputs) {
            Ok(result) => Ok(result),
            Err(primary_error) => {
                log::warn!("Primary calculation failed: {}", primary_error);
                
                // Attempt fallback calculation
                match self.fallback_calculation(inputs) {
                    Ok(fallback_result) => {
                        log::info!("Using fallback calculation result");
                        Ok(format!("fallback:{}", fallback_result))
                    },
                    Err(fallback_error) => {
                        log::error!("Both primary and fallback calculations failed");
                        Err(anyhow!("Calculation failed: {} | Fallback: {}", 
                                   primary_error, fallback_error))
                    }
                }
            }
        }
    }
    
    fn primary_calculation(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Complex calculation that might fail
        // ...
        Ok("primary_result".to_string())
    }
    
    fn fallback_calculation(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Simpler, more reliable calculation
        // ...
        Ok("fallback_result".to_string())
    }
}
```

## Testing Strategies

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FactValue, CalculatorInputs};
    use std::collections::HashMap;

    fn create_test_inputs(fields: Vec<(&str, FactValue)>) -> CalculatorInputs {
        let mut input_fields = HashMap::new();
        for (key, value) in fields {
            input_fields.insert(key.to_string(), value);
        }
        CalculatorInputs { fields: input_fields }
    }

    #[test]
    fn test_bonus_calculator_excellent_performance() {
        let calculator = CustomBusinessCalculator::new();
        let inputs = create_test_inputs(vec![
            ("employee_level", FactValue::String("senior".to_string())),
            ("base_salary", FactValue::Float(100000.0)),
            ("years_experience", FactValue::Float(5.0)),
        ]);

        let result = calculator.execute(&inputs).unwrap();
        let bonus: f64 = result.parse().unwrap();
        
        // Expected: 100000 * (0.10 + 5 * 0.015) = 100000 * 0.175 = 17500
        assert!((bonus - 17500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculator_missing_required_field() {
        let calculator = CustomBusinessCalculator::new();
        let inputs = create_test_inputs(vec![
            ("base_salary", FactValue::Float(100000.0)),
            // Missing employee_level
        ]);

        assert!(calculator.execute(&inputs).is_err());
    }

    #[test]
    fn test_calculator_invalid_employee_level() {
        let calculator = CustomBusinessCalculator::new();
        let inputs = create_test_inputs(vec![
            ("employee_level", FactValue::String("invalid".to_string())),
            ("base_salary", FactValue::Float(100000.0)),
        ]);

        assert!(calculator.execute(&inputs).is_err());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::engine::BingoEngine;
    use crate::types::{Rule, Condition, Action, ActionType};

    #[test]
    fn test_calculator_integration_in_rule_engine() {
        let mut engine = BingoEngine::new();
        
        // Register custom calculator
        let calculator = Box::new(CustomBusinessCalculator::new());
        engine.get_calculator_registry_mut()
            .register_calculator("custom_bonus".to_string(), calculator);
        
        // Create test rule
        let rule = Rule {
            id: 1,
            name: "Bonus Calculation".to_string(),
            conditions: vec![
                Condition::Simple {
                    field: "employee_type".to_string(),
                    operator: Operator::Equal,
                    value: FactValue::String("regular".to_string()),
                }
            ],
            actions: vec![
                Action {
                    action_type: ActionType::CallCalculator {
                        calculator_name: "custom_bonus".to_string(),
                        input_mapping: {
                            let mut mapping = HashMap::new();
                            mapping.insert("employee_level".to_string(), "level".to_string());
                            mapping.insert("base_salary".to_string(), "salary".to_string());
                            mapping
                        },
                        output_field: "calculated_bonus".to_string(),
                    }
                }
            ],
        };
        
        engine.add_rule(rule);
        
        // Create test fact
        let fact = Fact::new(1, FactData {
            fields: {
                let mut fields = HashMap::new();
                fields.insert("employee_type".to_string(), 
                            FactValue::String("regular".to_string()));
                fields.insert("level".to_string(), 
                            FactValue::String("senior".to_string()));
                fields.insert("salary".to_string(), 
                            FactValue::Float(120000.0));
                fields
            }
        });
        
        // Process fact
        let results = engine.process_facts(&[fact]).unwrap();
        
        // Verify calculator was called and result stored
        assert!(!results.is_empty());
        // Additional assertions based on expected results
    }
}
```

## Performance Optimization

### Calculator Caching

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct CachedCalculator<T: Calculator> {
    inner: T,
    cache: HashMap<String, String>,
    cache_hits: usize,
    cache_misses: usize,
}

impl<T: Calculator> CachedCalculator<T> {
    pub fn new(calculator: T) -> Self {
        Self {
            inner: calculator,
            cache: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    fn cache_key(&self, inputs: &CalculatorInputs) -> String {
        // Create deterministic cache key from inputs
        let mut key_parts: Vec<String> = inputs.fields
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v.as_string()))
            .collect();
        key_parts.sort();
        key_parts.join("|")
    }
    
    pub fn cache_stats(&self) -> (usize, usize, f64) {
        let total = self.cache_hits + self.cache_misses;
        let hit_rate = if total > 0 {
            (self.cache_hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        (self.cache_hits, self.cache_misses, hit_rate)
    }
}

impl<T: Calculator> Calculator for CachedCalculator<T> {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        let cache_key = self.cache_key(inputs);
        
        if let Some(cached_result) = self.cache.get(&cache_key) {
            self.cache_hits += 1;
            return Ok(cached_result.clone());
        }
        
        self.cache_misses += 1;
        let result = self.inner.execute(inputs)?;
        self.cache.insert(cache_key, result.clone());
        Ok(result)
    }
    
    fn validate_inputs(&self, inputs: &CalculatorInputs) -> Result<()> {
        self.inner.validate_inputs(inputs)
    }
    
    fn get_required_fields(&self) -> Vec<String> {
        self.inner.get_required_fields()
    }
    
    fn get_optional_fields(&self) -> Vec<String> {
        self.inner.get_optional_fields()
    }
}
```

### Async Calculator Support

```rust
use async_trait::async_trait;
use tokio::time::timeout;
use std::time::Duration;

#[async_trait]
pub trait AsyncCalculator {
    async fn execute_async(&self, inputs: &CalculatorInputs) -> Result<String>;
    async fn validate_inputs_async(&self, inputs: &CalculatorInputs) -> Result<()>;
}

pub struct AsyncCalculatorWrapper<T: AsyncCalculator> {
    inner: T,
    timeout_duration: Duration,
}

impl<T: AsyncCalculator> AsyncCalculatorWrapper<T> {
    pub fn new(calculator: T, timeout_seconds: u64) -> Self {
        Self {
            inner: calculator,
            timeout_duration: Duration::from_secs(timeout_seconds),
        }
    }
}

impl<T: AsyncCalculator + Send + Sync> Calculator for AsyncCalculatorWrapper<T> {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Run async calculator in blocking context with timeout
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            timeout(self.timeout_duration, self.inner.execute_async(inputs))
                .await
                .map_err(|_| anyhow!("Calculator execution timed out"))?
        })
    }
    
    fn validate_inputs(&self, inputs: &CalculatorInputs) -> Result<()> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            timeout(Duration::from_secs(5), self.inner.validate_inputs_async(inputs))
                .await
                .map_err(|_| anyhow!("Input validation timed out"))?
        })
    }
    
    fn get_required_fields(&self) -> Vec<String> {
        // These should be synchronous
        vec![]
    }
    
    fn get_optional_fields(&self) -> Vec<String> {
        vec![]
    }
}
```

## Configuration and Deployment

### External Configuration

```toml
# calculator_config.toml
[payroll_calculator]
overtime_multiplier = 1.5
double_time_threshold = 12.0
holiday_multiplier = 2.0

[payroll_calculator.tax_rates]
CA = 0.13
NY = 0.12
TX = 0.08
FL = 0.06

[bonus_calculator]
max_bonus_percent = 0.25
min_performance_rating = "satisfactory"

[bonus_calculator.department_multipliers]
sales = 1.1
engineering = 1.05
marketing = 1.0
support = 0.95
```

### Calculator Registration

```rust
impl CalculatorRegistry {
    pub fn register_from_config(&mut self, config_path: &str) -> Result<()> {
        let config_content = std::fs::read_to_string(config_path)?;
        let config: toml::Value = toml::from_str(&config_content)?;
        
        // Register payroll calculator
        if let Some(payroll_config) = config.get("payroll_calculator") {
            let calculator = PayrollCalculator::from_config(payroll_config)?;
            self.register_calculator(
                "payroll_calculator".to_string(), 
                Box::new(calculator)
            );
        }
        
        // Register other calculators from config
        // ...
        
        Ok(())
    }
}
```

### Dynamic Calculator Loading

```rust
use libloading::{Library, Symbol};

pub struct DynamicCalculatorLoader {
    libraries: HashMap<String, Library>,
}

impl DynamicCalculatorLoader {
    pub fn new() -> Self {
        Self {
            libraries: HashMap::new(),
        }
    }
    
    pub fn load_calculator(&mut self, 
                          library_path: &str, 
                          calculator_name: &str) -> Result<Box<dyn Calculator>> {
        // Load dynamic library
        let lib = unsafe { Library::new(library_path)? };
        
        // Get constructor function
        let create_calculator: Symbol<fn() -> Box<dyn Calculator>> = unsafe {
            lib.get(format!("create_{}", calculator_name).as_bytes())?
        };
        
        // Create calculator instance
        let calculator = create_calculator();
        
        // Store library to prevent unloading
        self.libraries.insert(calculator_name.to_string(), lib);
        
        Ok(calculator)
    }
}
```

## Best Practices and Guidelines

### 1. Calculator Design Principles

- **Single Responsibility**: Each calculator should have one clear purpose
- **Immutable**: Calculators should not modify external state
- **Deterministic**: Same inputs should always produce same outputs
- **Validated**: Always validate inputs before processing
- **Documented**: Provide clear documentation for fields and behavior

### 2. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum CalculatorError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid field type for {field}: expected {expected}, got {actual}")]
    InvalidFieldType { field: String, expected: String, actual: String },
    
    #[error("Field value out of range for {field}: {value}")]
    ValueOutOfRange { field: String, value: String },
    
    #[error("Business rule violation: {message}")]
    BusinessRuleViolation { message: String },
    
    #[error("Calculation overflow in {operation}")]
    CalculationOverflow { operation: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
}

impl Calculator for SafeCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Use custom error types for better error handling
        let value = inputs.fields.get("amount")
            .ok_or(CalculatorError::MissingField { 
                field: "amount".to_string() 
            })?;
            
        let numeric_value = value.as_f64()
            .ok_or(CalculatorError::InvalidFieldType {
                field: "amount".to_string(),
                expected: "number".to_string(),
                actual: value.type_name().to_string(),
            })?;
            
        if numeric_value < 0.0 {
            return Err(CalculatorError::ValueOutOfRange {
                field: "amount".to_string(),
                value: numeric_value.to_string(),
            }.into());
        }
        
        // Calculation logic...
        Ok(result)
    }
}
```

### 3. Performance Guidelines

- Use caching for expensive calculations
- Implement input validation early
- Consider async operations for external service calls
- Monitor calculator performance and execution times
- Use appropriate data types for calculations

### 4. Security Considerations

- Validate all inputs thoroughly
- Sanitize string inputs to prevent injection
- Implement rate limiting for expensive calculations
- Log security-relevant events
- Use secure communication for external services

```rust
impl Calculator for SecureCalculator {
    fn execute(&self, inputs: &CalculatorInputs) -> Result<String> {
        // Input sanitization
        if let Some(description) = inputs.fields.get("description") {
            if let Some(desc_str) = description.as_string() {
                if desc_str.len() > 1000 {
                    return Err(anyhow!("Description too long"));
                }
                if desc_str.contains("script") || desc_str.contains("javascript") {
                    return Err(anyhow!("Potentially unsafe content"));
                }
            }
        }
        
        // Rate limiting check
        if self.is_rate_limited(&inputs) {
            return Err(anyhow!("Rate limit exceeded"));
        }
        
        // Audit logging
        self.log_calculation_attempt(&inputs);
        
        // Actual calculation...
        let result = self.perform_calculation(inputs)?;
        
        // Log successful calculation
        self.log_calculation_success(&inputs, &result);
        
        Ok(result)
    }
}
```

This comprehensive guide provides the foundation for developing, integrating, and deploying custom business logic calculators within the Bingo RETE Rules Engine, enabling powerful and maintainable business rule implementations.