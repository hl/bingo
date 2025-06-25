//! Abstract Syntax Tree for calculator expressions

use crate::types::FactValue;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// AST node representing an expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Literal value (number, string, boolean)
    Literal(FactValue),

    /// Variable reference (field name or global)
    Variable(String),

    /// Binary operation (a + b, a > b, etc.)
    BinaryOp { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },

    /// Unary operation (-a, !a)
    UnaryOp { operator: UnaryOperator, operand: Box<Expression> },

    /// Function call (max(a, b), round(x, 2))
    FunctionCall { name: String, args: Vec<Expression> },

    /// Conditional expression (if condition then expr else expr)
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },

    /// Field access for complex objects (customer.age, order.items[0])
    FieldAccess { object: Box<Expression>, field: String },

    /// Conditional set with multiple condition-value pairs
    /// Evaluates conditions in order and returns the first matching value
    ConditionalSet {
        conditions: Vec<(Expression, Expression)>, // (condition, value) pairs
        default_value: Option<Box<Expression>>,    // default if no conditions match
    },

    /// Array literal expression ([1, 2, 3])
    ArrayLiteral { elements: Vec<Expression> },

    /// Object literal expression ({key: value, ...})
    ObjectLiteral { fields: Vec<(String, Expression)> },

    /// Array indexing (array[index])
    ArrayIndex { array: Box<Expression>, index: Box<Expression> },

    /// Date literal from string
    DateLiteral { iso_string: String },
}

/// Binary operators supported by the calculator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // Logical
    And,
    Or,

    // String operations
    Concat,
    Contains,
    StartsWith,
    EndsWith,

    // Array operations
    In,     // element in array
    Push,   // array push element
    Filter, // array filter
    Map,    // array map
}

/// Unary operators supported by the calculator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Numeric negation (-x)
    Negate,
    /// Logical negation (!x)
    Not,
    /// Absolute value (abs x)
    Abs,
}

impl Expression {
    /// Create a literal integer expression
    pub fn int(value: i64) -> Self {
        Self::Literal(FactValue::Integer(value))
    }

    /// Create a literal float expression
    pub fn float(value: f64) -> Self {
        Self::Literal(FactValue::Float(value))
    }

    /// Create a literal string expression
    pub fn string(value: String) -> Self {
        Self::Literal(FactValue::String(value))
    }

    /// Create a literal boolean expression
    pub fn bool(value: bool) -> Self {
        Self::Literal(FactValue::Boolean(value))
    }

    /// Create an array literal expression
    pub fn array(elements: Vec<Expression>) -> Self {
        Self::ArrayLiteral { elements }
    }

    /// Create an object literal expression
    pub fn object(fields: Vec<(String, Expression)>) -> Self {
        Self::ObjectLiteral { fields }
    }

    /// Create a date literal expression
    pub fn date(iso_string: String) -> Self {
        Self::DateLiteral { iso_string }
    }

    /// Create a null literal expression
    pub fn null() -> Self {
        Self::Literal(FactValue::Null)
    }

    /// Create an array indexing expression
    pub fn index(array: Expression, index: Expression) -> Self {
        Self::ArrayIndex { array: Box::new(array), index: Box::new(index) }
    }

    /// Create a variable reference
    pub fn var(name: &str) -> Self {
        Self::Variable(name.to_string())
    }

    /// Create a binary operation
    pub fn binary(left: Expression, op: BinaryOperator, right: Expression) -> Self {
        Self::BinaryOp { left: Box::new(left), operator: op, right: Box::new(right) }
    }

    /// Create a unary operation
    pub fn unary(op: UnaryOperator, operand: Expression) -> Self {
        Self::UnaryOp { operator: op, operand: Box::new(operand) }
    }

    /// Create a function call
    pub fn call(name: &str, args: Vec<Expression>) -> Self {
        Self::FunctionCall { name: name.to_string(), args }
    }

    /// Create a conditional expression
    pub fn conditional(
        condition: Expression,
        then_expr: Expression,
        else_expr: Expression,
    ) -> Self {
        Self::Conditional {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
        }
    }

    /// Create a field access expression
    pub fn field(object: Expression, field: &str) -> Self {
        Self::FieldAccess { object: Box::new(object), field: field.to_string() }
    }

    /// Create a conditional set expression
    pub fn conditional_set(
        conditions: Vec<(Expression, Expression)>,
        default_value: Option<Expression>,
    ) -> Self {
        Self::ConditionalSet { conditions, default_value: default_value.map(Box::new) }
    }
}

impl BinaryOperator {
    /// Get the precedence of this operator (higher = tighter binding)
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual => 3,
            BinaryOperator::Contains
            | BinaryOperator::StartsWith
            | BinaryOperator::EndsWith
            | BinaryOperator::In => 4,
            BinaryOperator::Push | BinaryOperator::Filter | BinaryOperator::Map => 4,
            BinaryOperator::Concat => 5,
            BinaryOperator::Add | BinaryOperator::Subtract => 6,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 7,
            BinaryOperator::Power => 8,
        }
    }

    /// Check if this operator is right-associative
    pub fn is_right_associative(&self) -> bool {
        matches!(self, BinaryOperator::Power)
    }
}

/// Extract all variable names referenced in an expression
pub fn extract_variables(expr: &Expression) -> Vec<String> {
    let mut variables = HashSet::new();
    extract_variables_recursive(expr, &mut variables);
    let mut result: Vec<String> = variables.into_iter().collect();
    result.sort();
    result
}

fn extract_variables_recursive(expr: &Expression, variables: &mut HashSet<String>) {
    match expr {
        Expression::Variable(name) => {
            variables.insert(name.clone());
        }
        Expression::BinaryOp { left, right, .. } => {
            extract_variables_recursive(left, variables);
            extract_variables_recursive(right, variables);
        }
        Expression::UnaryOp { operand, .. } => {
            extract_variables_recursive(operand, variables);
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                extract_variables_recursive(arg, variables);
            }
        }
        Expression::Conditional { condition, then_expr, else_expr } => {
            extract_variables_recursive(condition, variables);
            extract_variables_recursive(then_expr, variables);
            extract_variables_recursive(else_expr, variables);
        }
        Expression::FieldAccess { object, .. } => {
            extract_variables_recursive(object, variables);
        }
        Expression::ConditionalSet { conditions, default_value } => {
            for (condition, value) in conditions {
                extract_variables_recursive(condition, variables);
                extract_variables_recursive(value, variables);
            }
            if let Some(default) = default_value {
                extract_variables_recursive(default, variables);
            }
        }
        Expression::ArrayLiteral { elements } => {
            for element in elements {
                extract_variables_recursive(element, variables);
            }
        }
        Expression::ObjectLiteral { fields } => {
            for (_, value) in fields {
                extract_variables_recursive(value, variables);
            }
        }
        Expression::ArrayIndex { array, index } => {
            extract_variables_recursive(array, variables);
            extract_variables_recursive(index, variables);
        }
        Expression::DateLiteral { .. } | Expression::Literal(_) => {
            // Literals don't contain variables
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_creation() {
        let expr = Expression::binary(
            Expression::var("amount"),
            BinaryOperator::Multiply,
            Expression::float(1.15),
        );

        match expr {
            Expression::BinaryOp { left, operator, right } => {
                assert_eq!(left.as_ref(), &Expression::Variable("amount".to_string()));
                assert_eq!(operator, BinaryOperator::Multiply);
                assert_eq!(right.as_ref(), &Expression::Literal(FactValue::Float(1.15)));
            }
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_variable_extraction() {
        let expr = Expression::binary(
            Expression::var("amount"),
            BinaryOperator::Add,
            Expression::binary(
                Expression::var("tax"),
                BinaryOperator::Multiply,
                Expression::var("rate"),
            ),
        );

        let variables = extract_variables(&expr);
        assert_eq!(variables, vec!["amount", "rate", "tax"]);
    }

    #[test]
    fn test_operator_precedence() {
        assert!(BinaryOperator::Multiply.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::Power.precedence() > BinaryOperator::Multiply.precedence());
        assert!(BinaryOperator::And.precedence() > BinaryOperator::Or.precedence());
    }

    #[test]
    fn test_complex_expression() {
        let expr = Expression::conditional(
            Expression::binary(
                Expression::var("status"),
                BinaryOperator::Equal,
                Expression::string("active".to_string()),
            ),
            Expression::call(
                "max",
                vec![Expression::var("base_amount"), Expression::int(100)],
            ),
            Expression::int(0),
        );

        let variables = extract_variables(&expr);
        assert_eq!(variables, vec!["base_amount", "status"]);
    }

    #[test]
    fn test_conditional_set_expression() {
        let expr = Expression::conditional_set(
            vec![
                (
                    Expression::binary(
                        Expression::var("performance_rating"),
                        BinaryOperator::GreaterThanOrEqual,
                        Expression::float(4.5),
                    ),
                    Expression::float(0.15), // 15% bonus
                ),
                (
                    Expression::binary(
                        Expression::var("performance_rating"),
                        BinaryOperator::GreaterThanOrEqual,
                        Expression::float(4.0),
                    ),
                    Expression::float(0.10), // 10% bonus
                ),
                (
                    Expression::binary(
                        Expression::var("performance_rating"),
                        BinaryOperator::GreaterThanOrEqual,
                        Expression::float(3.5),
                    ),
                    Expression::float(0.05), // 5% bonus
                ),
            ],
            Some(Expression::float(0.0)), // no bonus
        );

        let variables = extract_variables(&expr);
        assert_eq!(variables, vec!["performance_rating"]);
    }
}
