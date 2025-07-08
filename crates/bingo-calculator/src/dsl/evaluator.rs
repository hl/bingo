//! Expression evaluator for calculator DSL
//!
//! This module evaluates parsed AST expressions against fact contexts,
//! providing type-safe computation with comprehensive error handling.

use crate::dsl::ast::{BinaryOperator, Expression, UnaryOperator};
use crate::dsl::functions::FunctionRegistry;
use crate::dsl::{CalculatorResult, EvaluationContext};
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

        Expression::DateLiteral { iso_string } => {
            use chrono::{DateTime, Utc};
            match iso_string.parse::<DateTime<Utc>>() {
                Ok(dt) => Ok(FactValue::Date(dt)),
                Err(e) => Err(anyhow!("Invalid date format '{}': {}", iso_string, e)),
            }
        }
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
        (String(a), String(b), Concat) => Ok(String(format!("{a}{b}"))),
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
    match value {
        FactValue::Boolean(b) => *b,
        FactValue::Integer(i) => *i != 0,
        FactValue::Float(f) => *f != 0.0,
        FactValue::String(s) => !s.is_empty(),
        FactValue::Array(arr) => !arr.is_empty(),
        FactValue::Object(obj) => !obj.is_empty(),
        FactValue::Date(_) => true,
        FactValue::Null => false,
    }
}
