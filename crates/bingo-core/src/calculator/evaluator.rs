//! Expression evaluator for calculator DSL
//!
//! This module evaluates parsed AST expressions against fact contexts,
//! providing type-safe computation with comprehensive error handling.

use crate::calculator::ast::{BinaryOperator, Expression, UnaryOperator};
use crate::calculator::functions::FunctionRegistry;
use crate::calculator::{CalculatorResult, EvaluationContext};
use crate::types::FactValue;
use anyhow::{Result, anyhow};

/// Evaluate an expression in the given context
pub fn evaluate_expression(
    expr: &Expression,
    context: &EvaluationContext,
    functions: &FunctionRegistry,
) -> Result<CalculatorResult> {
    let value = evaluate_to_value(expr, context, functions)?;
    Ok(CalculatorResult::Value(value))
}

/// Evaluate an expression to a single value
fn evaluate_to_value(
    expr: &Expression,
    context: &EvaluationContext,
    functions: &FunctionRegistry,
) -> Result<FactValue> {
    match expr {
        Expression::Literal(value) => Ok(value.clone()),

        Expression::Variable(name) => {
            // Check current fact fields first
            if let Some(value) = context.current_fact.data.fields.get(name) {
                return Ok(value.clone());
            }

            // Check global variables
            if let Some(value) = context.globals.get(name) {
                return Ok(value.clone());
            }

            Err(anyhow!("Variable '{}' not found in context", name))
        }

        Expression::BinaryOp { left, operator, right } => {
            let left_val = evaluate_to_value(left, context, functions)?;
            let right_val = evaluate_to_value(right, context, functions)?;
            evaluate_binary_op(&left_val, operator, &right_val)
        }

        Expression::UnaryOp { operator, operand } => {
            let operand_val = evaluate_to_value(operand, context, functions)?;
            evaluate_unary_op(operator, &operand_val)
        }

        Expression::FunctionCall { name, args } => {
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(evaluate_to_value(arg, context, functions)?);
            }

            functions.call_with_context(name, &arg_values, context)
        }

        Expression::Conditional { condition, then_expr, else_expr } => {
            let condition_val = evaluate_to_value(condition, context, functions)?;

            if is_truthy(&condition_val) {
                evaluate_to_value(then_expr, context, functions)
            } else {
                evaluate_to_value(else_expr, context, functions)
            }
        }

        Expression::FieldAccess { object, field } => {
            let object_val = evaluate_to_value(object, context, functions)?;

            // For now, field access is only supported on facts
            // In the future, we could extend this to support complex objects
            match object_val {
                FactValue::String(fact_id_str) => {
                    // Try to find fact by ID string
                    if let Ok(fact_id) = fact_id_str.parse::<u64>() {
                        if let Some(fact) = context.facts.iter().find(|f| f.id == fact_id) {
                            if let Some(field_value) = fact.data.fields.get(field) {
                                Ok(field_value.clone())
                            } else {
                                Err(anyhow!("Field '{}' not found on fact {}", field, fact_id))
                            }
                        } else {
                            Err(anyhow!("Fact with ID {} not found", fact_id))
                        }
                    } else {
                        Err(anyhow!("Cannot access field '{}' on string value", field))
                    }
                }
                _ => Err(anyhow!("Field access not supported on {:?}", object_val)),
            }
        }

        Expression::ConditionalSet { conditions, default_value } => {
            // Evaluate conditions in order and return the first matching value
            for (condition, value) in conditions {
                let condition_val = evaluate_to_value(condition, context, functions)?;
                if is_truthy(&condition_val) {
                    return evaluate_to_value(value, context, functions);
                }
            }

            // If no conditions matched, use default value or return error
            if let Some(default) = default_value {
                evaluate_to_value(default, context, functions)
            } else {
                Err(anyhow!(
                    "No conditions matched in conditional set and no default value provided"
                ))
            }
        }

        Expression::ArrayLiteral { elements } => {
            let mut array_values = Vec::new();
            for element in elements {
                array_values.push(evaluate_to_value(element, context, functions)?);
            }
            Ok(FactValue::Array(array_values))
        }

        Expression::ObjectLiteral { fields } => {
            let mut object_fields = std::collections::HashMap::new();
            for (key, value_expr) in fields {
                let value = evaluate_to_value(value_expr, context, functions)?;
                object_fields.insert(key.clone(), value);
            }
            Ok(FactValue::Object(object_fields))
        }

        Expression::ArrayIndex { array, index } => {
            let array_val = evaluate_to_value(array, context, functions)?;
            let index_val = evaluate_to_value(index, context, functions)?;

            match (array_val, index_val) {
                (FactValue::Array(arr), FactValue::Integer(idx)) => {
                    let index = if idx < 0 {
                        // Negative indexing from end
                        (arr.len() as i64 + idx) as usize
                    } else {
                        idx as usize
                    };

                    arr.get(index)
                        .cloned()
                        .ok_or_else(|| anyhow!("Array index {} out of bounds", idx))
                }
                (FactValue::Object(obj), FactValue::String(key)) => {
                    Ok(obj.get(&key).cloned().unwrap_or(FactValue::Null))
                }
                _ => Err(anyhow!("Invalid array/object indexing operation")),
            }
        }

        Expression::DateLiteral { iso_string } => FactValue::date_from_iso(iso_string)
            .map_err(|e| anyhow!("Invalid date format '{}': {}", iso_string, e)),
    }
}

/// Evaluate a binary operation
fn evaluate_binary_op(
    left: &FactValue,
    operator: &BinaryOperator,
    right: &FactValue,
) -> Result<FactValue> {
    use {BinaryOperator::*, FactValue::*};

    match (left, right, operator) {
        // Arithmetic operations
        (Integer(a), Integer(b), Add) => Ok(Integer(a + b)),
        (Integer(a), Integer(b), Subtract) => Ok(Integer(a - b)),
        (Integer(a), Integer(b), Multiply) => Ok(Integer(a * b)),
        (Integer(a), Integer(b), Divide) => {
            if *b == 0 {
                Err(anyhow!("Division by zero"))
            } else {
                Ok(Integer(a / b))
            }
        }
        (Integer(a), Integer(b), Modulo) => {
            if *b == 0 {
                Err(anyhow!("Modulo by zero"))
            } else {
                Ok(Integer(a % b))
            }
        }
        (Integer(a), Integer(b), Power) => {
            if *b < 0 {
                Ok(Float((*a as f64).powf(*b as f64)))
            } else {
                Ok(Integer(a.pow(*b as u32)))
            }
        }

        // Float arithmetic
        (Float(a), Float(b), Add) => Ok(Float(a + b)),
        (Float(a), Float(b), Subtract) => Ok(Float(a - b)),
        (Float(a), Float(b), Multiply) => Ok(Float(a * b)),
        (Float(a), Float(b), Divide) => {
            if *b == 0.0 {
                Err(anyhow!("Division by zero"))
            } else {
                Ok(Float(a / b))
            }
        }
        (Float(a), Float(b), Modulo) => Ok(Float(a % b)),
        (Float(a), Float(b), Power) => Ok(Float(a.powf(*b))),

        // Comparison operations
        (Integer(a), Integer(b), Equal) => Ok(Boolean(a == b)),
        (Integer(a), Integer(b), NotEqual) => Ok(Boolean(a != b)),
        (Integer(a), Integer(b), LessThan) => Ok(Boolean(a < b)),
        (Integer(a), Integer(b), LessThanOrEqual) => Ok(Boolean(a <= b)),
        (Integer(a), Integer(b), GreaterThan) => Ok(Boolean(a > b)),
        (Integer(a), Integer(b), GreaterThanOrEqual) => Ok(Boolean(a >= b)),

        (Float(a), Float(b), Equal) => Ok(Boolean((a - b).abs() < f64::EPSILON)),
        (Float(a), Float(b), NotEqual) => Ok(Boolean((a - b).abs() >= f64::EPSILON)),
        (Float(a), Float(b), LessThan) => Ok(Boolean(a < b)),
        (Float(a), Float(b), LessThanOrEqual) => Ok(Boolean(a <= b)),
        (Float(a), Float(b), GreaterThan) => Ok(Boolean(a > b)),
        (Float(a), Float(b), GreaterThanOrEqual) => Ok(Boolean(a >= b)),

        // Mixed numeric operations (comparisons and arithmetic)
        (Integer(a), Float(b), Equal) => Ok(Boolean((*a as f64 - b).abs() < f64::EPSILON)),
        (Integer(a), Float(b), NotEqual) => Ok(Boolean((*a as f64 - b).abs() >= f64::EPSILON)),
        (Integer(a), Float(b), LessThan) => Ok(Boolean((*a as f64) < *b)),
        (Integer(a), Float(b), LessThanOrEqual) => Ok(Boolean((*a as f64) <= *b)),
        (Integer(a), Float(b), GreaterThan) => Ok(Boolean((*a as f64) > *b)),
        (Integer(a), Float(b), GreaterThanOrEqual) => Ok(Boolean((*a as f64) >= *b)),
        (Integer(a), Float(b), op) => evaluate_binary_op(&Float(*a as f64), op, &Float(*b)),

        (Float(a), Integer(b), Equal) => Ok(Boolean((a - *b as f64).abs() < f64::EPSILON)),
        (Float(a), Integer(b), NotEqual) => Ok(Boolean((a - *b as f64).abs() >= f64::EPSILON)),
        (Float(a), Integer(b), LessThan) => Ok(Boolean(*a < (*b as f64))),
        (Float(a), Integer(b), LessThanOrEqual) => Ok(Boolean(*a <= (*b as f64))),
        (Float(a), Integer(b), GreaterThan) => Ok(Boolean(*a > (*b as f64))),
        (Float(a), Integer(b), GreaterThanOrEqual) => Ok(Boolean(*a >= (*b as f64))),
        (Float(a), Integer(b), op) => evaluate_binary_op(&Float(*a), op, &Float(*b as f64)),

        // String operations
        (String(a), String(b), Equal) => Ok(Boolean(a == b)),
        (String(a), String(b), NotEqual) => Ok(Boolean(a != b)),
        (String(a), String(b), LessThan) => Ok(Boolean(a < b)),
        (String(a), String(b), LessThanOrEqual) => Ok(Boolean(a <= b)),
        (String(a), String(b), GreaterThan) => Ok(Boolean(a > b)),
        (String(a), String(b), GreaterThanOrEqual) => Ok(Boolean(a >= b)),
        (String(a), String(b), Concat) => Ok(String(format!("{}{}", a, b))),
        (String(a), String(b), Contains) => Ok(Boolean(a.contains(b))),
        (String(a), String(b), StartsWith) => Ok(Boolean(a.starts_with(b))),
        (String(a), String(b), EndsWith) => Ok(Boolean(a.ends_with(b))),

        // Boolean operations
        (Boolean(a), Boolean(b), Equal) => Ok(Boolean(a == b)),
        (Boolean(a), Boolean(b), NotEqual) => Ok(Boolean(a != b)),
        (Boolean(a), Boolean(b), And) => Ok(Boolean(*a && *b)),
        (Boolean(a), Boolean(b), Or) => Ok(Boolean(*a || *b)),

        // Array operations
        (Array(a), Array(b), Equal) => Ok(Boolean(a == b)),
        (Array(a), Array(b), NotEqual) => Ok(Boolean(a != b)),
        (Array(a), Array(b), LessThan) => Ok(Boolean(a.len() < b.len())),
        (Array(a), Array(b), LessThanOrEqual) => Ok(Boolean(a.len() <= b.len())),
        (Array(a), Array(b), GreaterThan) => Ok(Boolean(a.len() > b.len())),
        (Array(a), Array(b), GreaterThanOrEqual) => Ok(Boolean(a.len() >= b.len())),
        (Array(a), Array(b), Concat) => {
            let mut result = a.clone();
            result.extend(b.clone());
            Ok(Array(result))
        }
        (element, Array(arr), In) => Ok(Boolean(arr.contains(element))),
        (Array(arr), element, Push) => {
            let mut new_arr = arr.clone();
            new_arr.push(element.clone());
            Ok(Array(new_arr))
        }

        // Object operations
        (Object(a), Object(b), Equal) => Ok(Boolean(a == b)),
        (Object(a), Object(b), NotEqual) => Ok(Boolean(a != b)),

        // Date operations
        (Date(a), Date(b), Equal) => Ok(Boolean(a == b)),
        (Date(a), Date(b), NotEqual) => Ok(Boolean(a != b)),
        (Date(a), Date(b), LessThan) => Ok(Boolean(a < b)),
        (Date(a), Date(b), LessThanOrEqual) => Ok(Boolean(a <= b)),
        (Date(a), Date(b), GreaterThan) => Ok(Boolean(a > b)),
        (Date(a), Date(b), GreaterThanOrEqual) => Ok(Boolean(a >= b)),

        // Null operations
        (Null, Null, Equal) => Ok(Boolean(true)),
        (Null, Null, NotEqual) => Ok(Boolean(false)),
        (Null, _, Equal) | (_, Null, Equal) => Ok(Boolean(false)),
        (Null, _, NotEqual) | (_, Null, NotEqual) => Ok(Boolean(true)),

        // Cross-type equality
        (a, b, Equal) if std::mem::discriminant(a) != std::mem::discriminant(b) => {
            Ok(Boolean(false))
        }
        (a, b, NotEqual) if std::mem::discriminant(a) != std::mem::discriminant(b) => {
            Ok(Boolean(true))
        }

        _ => Err(anyhow!(
            "Unsupported operation: {:?} {:?} {:?}",
            left,
            operator,
            right
        )),
    }
}

/// Evaluate a unary operation
fn evaluate_unary_op(operator: &UnaryOperator, operand: &FactValue) -> Result<FactValue> {
    use {FactValue::*, UnaryOperator::*};

    match (operator, operand) {
        (Negate, Integer(n)) => Ok(Integer(-n)),
        (Negate, Float(f)) => Ok(Float(-f)),
        (Not, Boolean(b)) => Ok(Boolean(!b)),
        (Abs, Integer(n)) => Ok(Integer(n.abs())),
        (Abs, Float(f)) => Ok(Float(f.abs())),
        (Not, Array(a)) => Ok(Boolean(a.is_empty())),
        (Not, Object(o)) => Ok(Boolean(o.is_empty())),
        (Not, Null) => Ok(Boolean(true)),
        _ => Err(anyhow!(
            "Unsupported unary operation: {:?} {:?}",
            operator,
            operand
        )),
    }
}

/// Check if a value is considered "truthy" for conditional evaluation
fn is_truthy(value: &FactValue) -> bool {
    value.is_truthy()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculator::functions::FunctionRegistry;
    use crate::types::{Fact, FactData};
    use std::collections::HashMap;

    fn create_test_fact() -> Fact {
        let mut fields = HashMap::new();
        fields.insert("amount".to_string(), FactValue::Float(100.0));
        fields.insert("rate".to_string(), FactValue::Float(0.15));
        fields.insert(
            "status".to_string(),
            FactValue::String("active".to_string()),
        );
        fields.insert("count".to_string(), FactValue::Integer(42));
        fields.insert("enabled".to_string(), FactValue::Boolean(true));

        Fact { id: 1, data: FactData { fields } }
    }

    #[test]
    fn test_literal_evaluation() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        let expr = Expression::int(42);
        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Integer(42));
    }

    #[test]
    fn test_variable_evaluation() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        let expr = Expression::var("amount");
        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Float(100.0));
    }

    #[test]
    fn test_arithmetic_evaluation() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // amount * rate
        let expr = Expression::binary(
            Expression::var("amount"),
            BinaryOperator::Multiply,
            Expression::var("rate"),
        );

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        if let FactValue::Float(value) = result {
            assert!((value - 15.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_comparison_evaluation() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // amount > 50
        let expr = Expression::binary(
            Expression::var("amount"),
            BinaryOperator::GreaterThan,
            Expression::float(50.0),
        );

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Boolean(true));
    }

    #[test]
    fn test_conditional_evaluation() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // if enabled then amount else 0
        let expr = Expression::conditional(
            Expression::var("enabled"),
            Expression::var("amount"),
            Expression::float(0.0),
        );

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Float(100.0));
    }

    #[test]
    fn test_string_operations() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // status contains "act"
        let expr = Expression::binary(
            Expression::var("status"),
            BinaryOperator::Contains,
            Expression::string("act".to_string()),
        );

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Boolean(true));
    }

    #[test]
    fn test_unary_operations() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // -amount
        let expr = Expression::unary(UnaryOperator::Negate, Expression::var("amount"));

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Float(-100.0));
    }

    #[test]
    fn test_conditional_set_evaluation() {
        let functions = FunctionRegistry::with_builtins();

        // Create test fact with performance_rating
        let mut fields = HashMap::new();
        fields.insert("performance_rating".to_string(), FactValue::Float(4.2));

        let test_fact = Fact { id: 1, data: FactData { fields } };

        let context =
            EvaluationContext { current_fact: &test_fact, facts: &[], globals: HashMap::new() };

        // Create a conditional set for performance-based bonus calculation
        // rating >= 4.5 -> 15%, rating >= 4.0 -> 10%, rating >= 3.5 -> 5%, else -> 0%
        let expr = Expression::conditional_set(
            vec![
                (
                    Expression::binary(
                        Expression::var("performance_rating"),
                        BinaryOperator::GreaterThanOrEqual,
                        Expression::float(4.5),
                    ),
                    Expression::float(0.15),
                ),
                (
                    Expression::binary(
                        Expression::var("performance_rating"),
                        BinaryOperator::GreaterThanOrEqual,
                        Expression::float(4.0),
                    ),
                    Expression::float(0.10),
                ),
                (
                    Expression::binary(
                        Expression::var("performance_rating"),
                        BinaryOperator::GreaterThanOrEqual,
                        Expression::float(3.5),
                    ),
                    Expression::float(0.05),
                ),
            ],
            Some(Expression::float(0.0)),
        );

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::Float(0.10)); // Should get 10% bonus
    }

    #[test]
    fn test_conditional_set_no_match() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // Conditional set with conditions that won't match our test data
        let expr = Expression::conditional_set(
            vec![(
                Expression::binary(
                    Expression::var("count"),
                    BinaryOperator::GreaterThan,
                    Expression::int(100), // count is 42 in test context
                ),
                Expression::string("high".to_string()),
            )],
            Some(Expression::string("low".to_string())),
        );

        let result = evaluate_to_value(&expr, &context, &functions).unwrap();
        assert_eq!(result, FactValue::String("low".to_string())); // Should get default
    }

    #[test]
    fn test_error_handling() {
        let fact = create_test_fact();
        let context =
            EvaluationContext { current_fact: &fact, facts: &[], globals: HashMap::new() };
        let functions = FunctionRegistry::with_builtins();

        // Reference non-existent variable
        let expr = Expression::var("nonexistent");
        let result = evaluate_to_value(&expr, &context, &functions);
        assert!(result.is_err());

        // Division by zero
        let expr = Expression::binary(
            Expression::int(10),
            BinaryOperator::Divide,
            Expression::int(0),
        );
        let result = evaluate_to_value(&expr, &context, &functions);
        assert!(result.is_err());
    }
}
