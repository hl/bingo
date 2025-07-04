//! Cross-fact aggregation and grouping capabilities
//!
//! This module provides sophisticated aggregation operations that can operate
//! across multiple facts, enabling complex business logic like payroll overtime
//! calculations that require summing hours across shifts within time windows.

use crate::types::{
    AggregationType, AggregationWindow, Condition, Fact, FactData, FactValue, LogicalOperator,
    Operator,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{info, instrument};

/// Aggregator for cross-fact operations
pub struct FactAggregator {
    /// Cache for grouped facts to avoid repeated grouping
    group_cache: HashMap<String, Vec<Vec<Fact>>>,
}

impl FactAggregator {
    /// Create a new fact aggregator
    pub fn new() -> Self {
        Self { group_cache: HashMap::new() }
    }

    /// Aggregate facts using the specified aggregation configuration
    #[instrument(skip(self, facts))]
    pub fn aggregate_facts(
        &mut self,
        facts: &[Fact],
        aggregation: &AggregationSpec,
    ) -> Result<Vec<AggregationResult>> {
        info!(
            total_facts = facts.len(),
            group_by_fields = aggregation.group_by.len(),
            aggregation_type = ?aggregation.aggregation_type,
            "Starting fact aggregation"
        );

        // Filter facts if filter condition is specified
        let filtered_facts: Vec<&Fact> = if let Some(filter) = &aggregation.filter {
            facts
                .iter()
                .filter(|fact| self.evaluate_condition(fact, filter).unwrap_or(false))
                .collect()
        } else {
            facts.iter().collect()
        };

        // Group facts by the specified fields
        let grouped_facts = self.group_facts(&filtered_facts, &aggregation.group_by)?;

        // Apply aggregation to each group
        let mut results = Vec::new();
        for (group_key, group_facts) in grouped_facts {
            let aggregated_value = self.apply_aggregation(
                &group_facts,
                &aggregation.source_field,
                &aggregation.aggregation_type,
            )?;

            let result = AggregationResult {
                group_key: group_key.clone(),
                aggregated_value,
                fact_count: group_facts.len(),
                source_field: aggregation.source_field.clone(),
                aggregation_type: aggregation.aggregation_type.clone(),
            };

            results.push(result);
        }

        info!(
            groups_processed = results.len(),
            facts_processed = filtered_facts.len(),
            "Fact aggregation completed"
        );

        Ok(results)
    }

    /// Group facts by specified fields
    fn group_facts(
        &self,
        facts: &[&Fact],
        group_by_fields: &[String],
    ) -> Result<HashMap<String, Vec<Fact>>> {
        let mut groups: HashMap<String, Vec<Fact>> = HashMap::new();

        for fact in facts {
            let group_key = self.generate_group_key(fact, group_by_fields)?;
            groups.entry(group_key).or_default().push((*fact).clone());
        }

        Ok(groups)
    }

    /// Generate a group key from fact fields
    fn generate_group_key(&self, fact: &Fact, group_by_fields: &[String]) -> Result<String> {
        let mut key_parts = Vec::new();

        for field in group_by_fields {
            let value = fact.data.fields.get(field).context(format!(
                "Group by field '{}' not found in fact {}",
                field, fact.id
            ))?;
            key_parts.push(value.as_string_direct());
        }

        Ok(key_parts.join("|"))
    }

    /// Apply aggregation function to a group of facts
    fn apply_aggregation(
        &self,
        facts: &[Fact],
        source_field: &str,
        aggregation_type: &AggregationType,
    ) -> Result<FactValue> {
        if facts.is_empty() {
            return Ok(FactValue::Null);
        }

        // Extract values from the source field
        let values: Result<Vec<f64>, _> = facts
            .iter()
            .map(|fact| {
                fact.data
                    .fields
                    .get(source_field)
                    .context(format!(
                        "Source field '{}' not found in fact {}",
                        source_field, fact.id
                    ))
                    .and_then(|value| self.extract_numeric_value(value))
            })
            .collect();

        let values = values?;

        if values.is_empty() {
            return Ok(FactValue::Null);
        }

        let result = match aggregation_type {
            AggregationType::Sum => FactValue::Float(values.iter().sum::<f64>()),
            AggregationType::Count => FactValue::Integer(values.len() as i64),
            AggregationType::Average => {
                let sum: f64 = values.iter().sum();
                FactValue::Float(sum / values.len() as f64)
            }
            AggregationType::Min => {
                let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                FactValue::Float(min)
            }
            AggregationType::Max => {
                let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                FactValue::Float(max)
            }
            AggregationType::StandardDeviation => {
                let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
                let variance: f64 =
                    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                FactValue::Float(variance.sqrt())
            }
            AggregationType::Percentile(p) => {
                let mut sorted_values = values.clone();
                sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let index = ((*p / 100.0) * (sorted_values.len() - 1) as f64) as usize;
                FactValue::Float(sorted_values.get(index).copied().unwrap_or(0.0))
            }
        };

        Ok(result)
    }

    /// Extract numeric value from FactValue
    fn extract_numeric_value(&self, value: &FactValue) -> Result<f64> {
        match value {
            FactValue::Integer(i) => Ok(*i as f64),
            FactValue::Float(f) => Ok(*f),
            FactValue::String(s) => {
                s.parse::<f64>().context(format!("Could not parse '{}' as numeric value", s))
            }
            _ => Err(anyhow::anyhow!("Value {:?} is not numeric", value)),
        }
    }

    /// Evaluate a condition against a fact
    fn evaluate_condition(&self, fact: &Fact, condition: &Condition) -> Result<bool> {
        match condition {
            Condition::Simple { field, operator, value } => {
                let fact_value = fact.data.fields.get(field);
                if let Some(fact_value) = fact_value {
                    self.evaluate_simple_condition(fact_value, operator, value)
                } else {
                    Ok(false) // Field not found
                }
            }
            Condition::Complex { operator, conditions } => match operator {
                LogicalOperator::And => conditions.iter().try_fold(true, |acc, cond| {
                    Ok(acc && self.evaluate_condition(fact, cond)?)
                }),
                LogicalOperator::Or => conditions.iter().try_fold(false, |acc, cond| {
                    Ok(acc || self.evaluate_condition(fact, cond)?)
                }),
                LogicalOperator::Not => {
                    // NOT operator should negate the first condition
                    if let Some(first_condition) = conditions.first() {
                        Ok(!self.evaluate_condition(fact, first_condition)?)
                    } else {
                        Ok(true) // Empty NOT condition defaults to true
                    }
                }
            },
            _ => {
                // For now, return false for unsupported condition types
                Ok(false)
            }
        }
    }

    /// Evaluate a simple condition
    fn evaluate_simple_condition(
        &self,
        fact_value: &FactValue,
        operator: &Operator,
        condition_value: &FactValue,
    ) -> Result<bool> {
        match operator {
            Operator::Equal => Ok(fact_value == condition_value),
            Operator::NotEqual => Ok(fact_value != condition_value),
            Operator::GreaterThan => {
                Ok(fact_value.partial_cmp(condition_value) == Some(std::cmp::Ordering::Greater))
            }
            Operator::LessThan => {
                Ok(fact_value.partial_cmp(condition_value) == Some(std::cmp::Ordering::Less))
            }
            Operator::GreaterThanOrEqual => {
                let cmp = fact_value.partial_cmp(condition_value);
                Ok(cmp == Some(std::cmp::Ordering::Greater)
                    || cmp == Some(std::cmp::Ordering::Equal))
            }
            Operator::LessThanOrEqual => {
                let cmp = fact_value.partial_cmp(condition_value);
                Ok(cmp == Some(std::cmp::Ordering::Less) || cmp == Some(std::cmp::Ordering::Equal))
            }
            Operator::Contains => {
                // For string contains operation
                match (fact_value, condition_value) {
                    (FactValue::String(s1), FactValue::String(s2)) => Ok(s1.contains(s2)),
                    _ => Ok(false),
                }
            }
        }
    }

    /// Clear the internal cache
    pub fn clear_cache(&mut self) {
        self.group_cache.clear();
    }
}

impl Default for FactAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Specification for an aggregation operation
#[derive(Debug, Clone)]
pub struct AggregationSpec {
    /// Fields to group facts by
    pub group_by: Vec<String>,
    /// Field to aggregate
    pub source_field: String,
    /// Type of aggregation to perform
    pub aggregation_type: AggregationType,
    /// Optional filter condition
    pub filter: Option<Condition>,
    /// Optional time window for temporal aggregations
    pub time_window: Option<AggregationWindow>,
}

/// Result of an aggregation operation
#[derive(Debug, Clone)]
pub struct AggregationResult {
    /// The group key that identifies this aggregation group
    pub group_key: String,
    /// The aggregated value
    pub aggregated_value: FactValue,
    /// Number of facts that contributed to this aggregation
    pub fact_count: usize,
    /// Source field that was aggregated
    pub source_field: String,
    /// Type of aggregation performed
    pub aggregation_type: AggregationType,
}

impl AggregationResult {
    /// Convert the aggregation result to a new fact
    pub fn to_fact(&self, fact_id: u64, target_field: &str) -> Fact {
        let mut fields = HashMap::new();

        // Add the aggregated value
        fields.insert(target_field.to_string(), self.aggregated_value.clone());

        // Add metadata about the aggregation
        fields.insert(
            "aggregation_type".to_string(),
            FactValue::String(format!("{:?}", self.aggregation_type)),
        );
        fields.insert(
            "source_field".to_string(),
            FactValue::String(self.source_field.clone()),
        );
        fields.insert(
            "fact_count".to_string(),
            FactValue::Integer(self.fact_count as i64),
        );
        fields.insert(
            "group_key".to_string(),
            FactValue::String(self.group_key.clone()),
        );

        Fact {
            timestamp: chrono::Utc::now(),
            id: fact_id,
            external_id: None,

            data: FactData { fields },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_fact(id: u64, employee: &str, hours: f64) -> Fact {
        let mut fields = HashMap::new();
        fields.insert(
            "employee_number".to_string(),
            FactValue::String(employee.to_string()),
        );
        fields.insert("hours".to_string(), FactValue::Float(hours));
        fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );

        Fact { timestamp: chrono::Utc::now(), id, external_id: None, data: FactData { fields } }
    }

    #[test]
    fn test_basic_aggregation() {
        let mut aggregator = FactAggregator::new();

        let facts = vec![
            create_test_fact(1, "EMP001", 8.0),
            create_test_fact(2, "EMP001", 9.0),
            create_test_fact(3, "EMP002", 7.5),
        ];

        let spec = AggregationSpec {
            group_by: vec!["employee_number".to_string()],
            source_field: "hours".to_string(),
            aggregation_type: AggregationType::Sum,
            filter: None,
            time_window: None,
        };

        let results = aggregator.aggregate_facts(&facts, &spec).unwrap();

        assert_eq!(results.len(), 2);

        // Find EMP001 result
        let emp001_result = results.iter().find(|r| r.group_key == "EMP001").unwrap();

        assert_eq!(emp001_result.aggregated_value, FactValue::Float(17.0));
        assert_eq!(emp001_result.fact_count, 2);
    }

    #[test]
    fn test_filtered_aggregation() {
        let mut aggregator = FactAggregator::new();

        let facts = vec![
            create_test_fact(1, "EMP001", 8.0),
            create_test_fact(2, "EMP001", 9.0),
            create_test_fact(3, "EMP002", 7.5),
        ];

        // Only aggregate active facts
        let filter = Condition::Simple {
            field: "status".to_string(),
            operator: Operator::Equal,
            value: FactValue::String("active".to_string()),
        };

        let spec = AggregationSpec {
            group_by: vec!["employee_number".to_string()],
            source_field: "hours".to_string(),
            aggregation_type: AggregationType::Sum,
            filter: Some(filter),
            time_window: None,
        };

        let results = aggregator.aggregate_facts(&facts, &spec).unwrap();

        assert_eq!(results.len(), 2); // All facts have status "active"
    }
}
