//! Built-in functions for calculator DSL
//!
//! This module provides a registry of mathematical, logical, and utility functions
//! that can be called from calculator expressions.

use crate::calculator::EvaluationContext;
use crate::types::FactValue;
use anyhow::{Result, anyhow};
use chrono::Datelike;
use std::collections::HashMap;

/// Trait for functions that can be called from calculator expressions
pub trait CalculatorFunction: Send + Sync {
    /// Call the function with the given arguments
    fn call(&self, args: &[FactValue]) -> Result<FactValue>;

    /// Get the expected number of arguments (None for variadic)
    fn arity(&self) -> Option<usize>;

    /// Get a description of this function
    fn description(&self) -> &'static str;
}

/// Trait for context-aware functions that need access to evaluation context
pub trait ContextAwareFunction: Send + Sync {
    /// Call the function with arguments and context
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue>;

    /// Get the expected number of arguments (None for variadic)
    fn arity(&self) -> Option<usize>;

    /// Get a description of this function
    fn description(&self) -> &'static str;
}

/// Registry for calculator functions
#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, Box<dyn CalculatorFunction>>,
    context_functions: HashMap<String, Box<dyn ContextAwareFunction>>,
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .field(
                "context_functions",
                &self.context_functions.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl FunctionRegistry {
    /// Create a new empty function registry
    pub fn new() -> Self {
        Self { functions: HashMap::new(), context_functions: HashMap::new() }
    }

    /// Create a function registry with built-in functions
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        // Mathematical functions
        registry.register("abs", Box::new(AbsFunction));
        registry.register("min", Box::new(MinFunction));
        registry.register("max", Box::new(MaxFunction));
        registry.register("round", Box::new(RoundFunction));
        registry.register("floor", Box::new(FloorFunction));
        registry.register("ceil", Box::new(CeilFunction));
        registry.register("sqrt", Box::new(SqrtFunction));
        registry.register("pow", Box::new(PowerFunction));

        // String functions
        registry.register("len", Box::new(LengthFunction));
        registry.register("upper", Box::new(UpperFunction));
        registry.register("lower", Box::new(LowerFunction));
        registry.register("trim", Box::new(TrimFunction));
        registry.register("substring", Box::new(SubstringFunction));
        registry.register("replace", Box::new(ReplaceFunction));

        // Utility functions
        registry.register("if", Box::new(IfFunction));
        registry.register("coalesce", Box::new(CoalesceFunction));
        registry.register("type_of", Box::new(TypeOfFunction));

        // Conversion functions
        registry.register("to_int", Box::new(ToIntFunction));
        registry.register("to_float", Box::new(ToFloatFunction));
        registry.register("to_string", Box::new(ToStringFunction));

        // Array functions
        registry.register("array_len", Box::new(ArrayLenFunction));
        registry.register("array_push", Box::new(ArrayPushFunction));
        registry.register("array_pop", Box::new(ArrayPopFunction));
        registry.register("array_contains", Box::new(ArrayContainsFunction));
        registry.register("array_slice", Box::new(ArraySliceFunction));
        registry.register("array_join", Box::new(ArrayJoinFunction));
        registry.register("array_sum", Box::new(ArraySumFunction));
        registry.register("array_avg", Box::new(ArrayAvgFunction));
        registry.register("array_min", Box::new(ArrayMinFunction));
        registry.register("array_max", Box::new(ArrayMaxFunction));

        // Object functions
        registry.register("object_keys", Box::new(ObjectKeysFunction));
        registry.register("object_values", Box::new(ObjectValuesFunction));
        registry.register("object_has_key", Box::new(ObjectHasKeyFunction));
        registry.register("object_get", Box::new(ObjectGetFunction));
        registry.register("object_merge", Box::new(ObjectMergeFunction));

        // Date functions
        registry.register("date_now", Box::new(DateNowFunction));
        registry.register("date_parse", Box::new(DateParseFunction));
        registry.register("date_format", Box::new(DateFormatFunction));
        registry.register("date_add_days", Box::new(DateAddDaysFunction));
        registry.register("date_diff_days", Box::new(DateDiffDaysFunction));
        registry.register("date_year", Box::new(DateYearFunction));
        registry.register("date_month", Box::new(DateMonthFunction));
        registry.register("date_day", Box::new(DateDayFunction));

        // Advanced string functions
        registry.register("regex_match", Box::new(RegexMatchFunction));
        registry.register("split", Box::new(SplitFunction));
        registry.register("format", Box::new(FormatFunction));

        // Cross-fact aggregation functions (context-aware)
        registry.register_context_function("fact_count", Box::new(FactCountFunction));
        registry.register_context_function("fact_sum", Box::new(FactSumFunction));
        registry.register_context_function("fact_avg", Box::new(FactAvgFunction));
        registry.register_context_function("fact_min", Box::new(FactMinFunction));
        registry.register_context_function("fact_max", Box::new(FactMaxFunction));
        registry.register_context_function("fact_field", Box::new(FactFieldFunction));
        registry.register_context_function("fact_exists", Box::new(FactExistsFunction));

        registry
    }

    /// Register a new function
    pub fn register(&mut self, name: &str, function: Box<dyn CalculatorFunction>) {
        self.functions.insert(name.to_lowercase(), function);
    }

    /// Register a new context-aware function
    pub fn register_context_function(
        &mut self,
        name: &str,
        function: Box<dyn ContextAwareFunction>,
    ) {
        self.context_functions.insert(name.to_lowercase(), function);
    }

    /// Call a function by name
    pub fn call(&self, name: &str, args: &[FactValue]) -> Result<FactValue> {
        let function = self
            .functions
            .get(&name.to_lowercase())
            .ok_or_else(|| anyhow!("Unknown function: {}", name))?;

        // Check arity if the function specifies one
        if let Some(expected_arity) = function.arity() {
            if args.len() != expected_arity {
                return Err(anyhow!(
                    "Function '{}' expects {} arguments, got {}",
                    name,
                    expected_arity,
                    args.len()
                ));
            }
        }

        function.call(args)
    }

    /// Call a context-aware function by name
    pub fn call_with_context(
        &self,
        name: &str,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        // Try context-aware functions first
        if let Some(function) = self.context_functions.get(&name.to_lowercase()) {
            // Check arity if the function specifies one
            if let Some(expected_arity) = function.arity() {
                if args.len() != expected_arity {
                    return Err(anyhow!(
                        "Function '{}' expects {} arguments, got {}",
                        name,
                        expected_arity,
                        args.len()
                    ));
                }
            }
            return function.call_with_context(args, context);
        }

        // Fall back to regular functions
        self.call(name, args)
    }

    /// Get list of available functions
    pub fn list_functions(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }
}

// Mathematical functions

struct AbsFunction;
impl CalculatorFunction for AbsFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Integer(n) => Ok(FactValue::Integer(n.abs())),
            FactValue::Float(f) => Ok(FactValue::Float(f.abs())),
            _ => Err(anyhow!("abs() requires a numeric argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the absolute value of a number"
    }
}

struct MinFunction;
impl CalculatorFunction for MinFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        if args.is_empty() {
            return Err(anyhow!("min() requires at least one argument"));
        }

        let mut min_val = &args[0];
        for arg in &args[1..] {
            match (min_val, arg) {
                (FactValue::Integer(a), FactValue::Integer(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Float(b)) => {
                    if b < a {
                        min_val = arg;
                    }
                }
                (FactValue::Integer(a), FactValue::Float(b)) => {
                    if b < &(*a as f64) {
                        min_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Integer(b)) => {
                    if (*b as f64) < *a {
                        min_val = arg;
                    }
                }
                _ => return Err(anyhow!("min() requires numeric arguments")),
            }
        }

        Ok(min_val.clone())
    }

    fn arity(&self) -> Option<usize> {
        None
    } // Variadic
    fn description(&self) -> &'static str {
        "Returns the minimum of the given numbers"
    }
}

struct MaxFunction;
impl CalculatorFunction for MaxFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        if args.is_empty() {
            return Err(anyhow!("max() requires at least one argument"));
        }

        let mut max_val = &args[0];
        for arg in &args[1..] {
            match (max_val, arg) {
                (FactValue::Integer(a), FactValue::Integer(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Float(b)) => {
                    if b > a {
                        max_val = arg;
                    }
                }
                (FactValue::Integer(a), FactValue::Float(b)) => {
                    if b > &(*a as f64) {
                        max_val = arg;
                    }
                }
                (FactValue::Float(a), FactValue::Integer(b)) => {
                    if (*b as f64) > *a {
                        max_val = arg;
                    }
                }
                _ => return Err(anyhow!("max() requires numeric arguments")),
            }
        }

        Ok(max_val.clone())
    }

    fn arity(&self) -> Option<usize> {
        None
    } // Variadic
    fn description(&self) -> &'static str {
        "Returns the maximum of the given numbers"
    }
}

struct RoundFunction;
impl CalculatorFunction for RoundFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let precision = if args.len() > 1 {
            match &args[1] {
                FactValue::Integer(p) => *p as u32,
                _ => return Err(anyhow!("round() precision must be an integer")),
            }
        } else {
            0
        };

        match &args[0] {
            FactValue::Integer(n) => Ok(FactValue::Integer(*n)),
            FactValue::Float(f) => {
                let multiplier = 10.0_f64.powi(precision as i32);
                let rounded = (f * multiplier).round() / multiplier;
                if precision == 0 {
                    Ok(FactValue::Integer(rounded as i64))
                } else {
                    Ok(FactValue::Float(rounded))
                }
            }
            _ => Err(anyhow!("round() requires a numeric argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        None
    } // 1 or 2 arguments
    fn description(&self) -> &'static str {
        "Rounds a number to specified decimal places (default 0)"
    }
}

struct FloorFunction;
impl CalculatorFunction for FloorFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Integer(n) => Ok(FactValue::Integer(*n)),
            FactValue::Float(f) => Ok(FactValue::Integer(f.floor() as i64)),
            _ => Err(anyhow!("floor() requires a numeric argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the largest integer less than or equal to the number"
    }
}

struct CeilFunction;
impl CalculatorFunction for CeilFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Integer(n) => Ok(FactValue::Integer(*n)),
            FactValue::Float(f) => Ok(FactValue::Integer(f.ceil() as i64)),
            _ => Err(anyhow!("ceil() requires a numeric argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the smallest integer greater than or equal to the number"
    }
}

struct SqrtFunction;
impl CalculatorFunction for SqrtFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Integer(n) => {
                if *n < 0 {
                    return Err(anyhow!("sqrt() of negative number"));
                }
                Ok(FactValue::Float((*n as f64).sqrt()))
            }
            FactValue::Float(f) => {
                if *f < 0.0 {
                    return Err(anyhow!("sqrt() of negative number"));
                }
                Ok(FactValue::Float(f.sqrt()))
            }
            _ => Err(anyhow!("sqrt() requires a numeric argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the square root of a number"
    }
}

struct PowerFunction;
impl CalculatorFunction for PowerFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match (&args[0], &args[1]) {
            (FactValue::Integer(base), FactValue::Integer(exp)) => {
                if *exp < 0 {
                    Ok(FactValue::Float((*base as f64).powf(*exp as f64)))
                } else {
                    Ok(FactValue::Integer(base.pow(*exp as u32)))
                }
            }
            (FactValue::Float(base), FactValue::Float(exp)) => {
                Ok(FactValue::Float(base.powf(*exp)))
            }
            (FactValue::Integer(base), FactValue::Float(exp)) => {
                Ok(FactValue::Float((*base as f64).powf(*exp)))
            }
            (FactValue::Float(base), FactValue::Integer(exp)) => {
                Ok(FactValue::Float(base.powf(*exp as f64)))
            }
            _ => Err(anyhow!("pow() requires numeric arguments")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Returns base raised to the power of exponent"
    }
}

// String functions

struct LengthFunction;
impl CalculatorFunction for LengthFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::String(s) => Ok(FactValue::Integer(s.len() as i64)),
            _ => Err(anyhow!("len() requires a string argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the length of a string"
    }
}

struct UpperFunction;
impl CalculatorFunction for UpperFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::String(s) => Ok(FactValue::String(s.to_uppercase())),
            _ => Err(anyhow!("upper() requires a string argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Converts a string to uppercase"
    }
}

struct LowerFunction;
impl CalculatorFunction for LowerFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::String(s) => Ok(FactValue::String(s.to_lowercase())),
            _ => Err(anyhow!("lower() requires a string argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Converts a string to lowercase"
    }
}

struct TrimFunction;
impl CalculatorFunction for TrimFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::String(s) => Ok(FactValue::String(s.trim().to_string())),
            _ => Err(anyhow!("trim() requires a string argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Removes whitespace from the beginning and end of a string"
    }
}

struct SubstringFunction;
impl CalculatorFunction for SubstringFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let s = match &args[0] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("substring() requires a string as first argument")),
        };

        let start = match &args[1] {
            FactValue::Integer(n) => *n as usize,
            _ => return Err(anyhow!("substring() requires an integer start index")),
        };

        let end = if args.len() > 2 {
            match &args[2] {
                FactValue::Integer(n) => Some(*n as usize),
                _ => return Err(anyhow!("substring() requires an integer end index")),
            }
        } else {
            None
        };

        let chars: Vec<char> = s.chars().collect();
        let start = start.min(chars.len());
        let end = end.unwrap_or(chars.len()).min(chars.len());

        if start <= end {
            let result: String = chars[start..end].iter().collect();
            Ok(FactValue::String(result))
        } else {
            Ok(FactValue::String(String::new()))
        }
    }

    fn arity(&self) -> Option<usize> {
        None
    } // 2 or 3 arguments
    fn description(&self) -> &'static str {
        "Extracts a substring from start index (optionally to end index)"
    }
}

struct ReplaceFunction;
impl CalculatorFunction for ReplaceFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let s = match &args[0] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("replace() requires a string as first argument")),
        };

        let pattern = match &args[1] {
            FactValue::String(p) => p,
            _ => return Err(anyhow!("replace() requires a string pattern")),
        };

        let replacement = match &args[2] {
            FactValue::String(r) => r,
            _ => return Err(anyhow!("replace() requires a string replacement")),
        };

        Ok(FactValue::String(s.replace(pattern, replacement)))
    }

    fn arity(&self) -> Option<usize> {
        Some(3)
    }
    fn description(&self) -> &'static str {
        "Replaces all occurrences of pattern with replacement in string"
    }
}

// Utility functions

struct IfFunction;
impl CalculatorFunction for IfFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let condition = args[0].is_truthy();

        if condition {
            Ok(args[1].clone())
        } else {
            Ok(args[2].clone())
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(3)
    }
    fn description(&self) -> &'static str {
        "Returns second argument if first is truthy, otherwise third argument"
    }
}

struct CoalesceFunction;
impl CalculatorFunction for CoalesceFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        for arg in args {
            match arg {
                FactValue::String(s) if !s.is_empty() => return Ok(arg.clone()),
                FactValue::Integer(_) | FactValue::Float(_) | FactValue::Boolean(_) => {
                    return Ok(arg.clone());
                }
                _ => continue,
            }
        }

        // If all values are "empty", return the last one
        Ok(args.last().unwrap_or(&FactValue::String(String::new())).clone())
    }

    fn arity(&self) -> Option<usize> {
        None
    } // Variadic
    fn description(&self) -> &'static str {
        "Returns the first non-empty value from the arguments"
    }
}

struct TypeOfFunction;
impl CalculatorFunction for TypeOfFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        Ok(FactValue::String(args[0].type_name().to_string()))
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the type of a value as a string"
    }
}

// Conversion functions

struct ToIntFunction;
impl CalculatorFunction for ToIntFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Integer(n) => Ok(FactValue::Integer(*n)),
            FactValue::Float(f) => Ok(FactValue::Integer(*f as i64)),
            FactValue::String(s) => {
                let parsed = s
                    .trim()
                    .parse::<i64>()
                    .map_err(|_| anyhow!("Cannot convert '{}' to integer", s))?;
                Ok(FactValue::Integer(parsed))
            }
            FactValue::Boolean(b) => Ok(FactValue::Integer(if *b { 1 } else { 0 })),
            FactValue::Date(d) => Ok(FactValue::Integer(d.timestamp())),
            FactValue::Array(arr) => Ok(FactValue::Integer(arr.len() as i64)),
            FactValue::Object(obj) => Ok(FactValue::Integer(obj.len() as i64)),
            FactValue::Null => Ok(FactValue::Integer(0)),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Converts a value to an integer"
    }
}

struct ToFloatFunction;
impl CalculatorFunction for ToFloatFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Integer(n) => Ok(FactValue::Float(*n as f64)),
            FactValue::Float(f) => Ok(FactValue::Float(*f)),
            FactValue::String(s) => {
                let parsed = s
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| anyhow!("Cannot convert '{}' to float", s))?;
                Ok(FactValue::Float(parsed))
            }
            FactValue::Boolean(b) => Ok(FactValue::Float(if *b { 1.0 } else { 0.0 })),
            FactValue::Date(d) => Ok(FactValue::Float(d.timestamp() as f64)),
            FactValue::Array(arr) => Ok(FactValue::Float(arr.len() as f64)),
            FactValue::Object(obj) => Ok(FactValue::Float(obj.len() as f64)),
            FactValue::Null => Ok(FactValue::Float(0.0)),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Converts a value to a float"
    }
}

struct ToStringFunction;
impl CalculatorFunction for ToStringFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        Ok(FactValue::String(args[0].as_string()))
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Converts a value to a string"
    }
}

// Array functions

struct ArrayLenFunction;
impl CalculatorFunction for ArrayLenFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Array(arr) => Ok(FactValue::Integer(arr.len() as i64)),
            FactValue::Object(obj) => Ok(FactValue::Integer(obj.len() as i64)),
            FactValue::String(s) => Ok(FactValue::Integer(s.len() as i64)),
            _ => Err(anyhow!("array_len() requires an array, object, or string")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the length of an array, object, or string"
    }
}

struct ArrayPushFunction;
impl CalculatorFunction for ArrayPushFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Array(arr) => {
                let mut new_arr = arr.clone();
                new_arr.push(args[1].clone());
                Ok(FactValue::Array(new_arr))
            }
            _ => Err(anyhow!("array_push() requires an array as first argument")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Adds an element to the end of an array"
    }
}

struct ArrayPopFunction;
impl CalculatorFunction for ArrayPopFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Array(arr) => {
                if arr.is_empty() {
                    Ok(FactValue::Null)
                } else {
                    Ok(arr[arr.len() - 1].clone())
                }
            }
            _ => Err(anyhow!("array_pop() requires an array")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the last element of an array"
    }
}

struct ArrayContainsFunction;
impl CalculatorFunction for ArrayContainsFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Array(arr) => Ok(FactValue::Boolean(arr.contains(&args[1]))),
            _ => Err(anyhow!(
                "array_contains() requires an array as first argument"
            )),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Checks if an array contains a specific value"
    }
}

struct ArraySliceFunction;
impl CalculatorFunction for ArraySliceFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let arr = match &args[0] {
            FactValue::Array(arr) => arr,
            _ => return Err(anyhow!("array_slice() requires an array as first argument")),
        };

        let start = match &args[1] {
            FactValue::Integer(i) => *i as usize,
            _ => return Err(anyhow!("array_slice() requires an integer start index")),
        };

        let end = if args.len() > 2 {
            match &args[2] {
                FactValue::Integer(i) => Some(*i as usize),
                _ => return Err(anyhow!("array_slice() requires an integer end index")),
            }
        } else {
            None
        };

        let start = start.min(arr.len());
        let end = end.unwrap_or(arr.len()).min(arr.len());

        if start <= end {
            Ok(FactValue::Array(arr[start..end].to_vec()))
        } else {
            Ok(FactValue::Array(vec![]))
        }
    }

    fn arity(&self) -> Option<usize> {
        None
    } // 2 or 3 arguments
    fn description(&self) -> &'static str {
        "Extracts a slice of an array from start index (optionally to end index)"
    }
}

struct ArrayJoinFunction;
impl CalculatorFunction for ArrayJoinFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let arr = match &args[0] {
            FactValue::Array(arr) => arr,
            _ => return Err(anyhow!("array_join() requires an array as first argument")),
        };

        let separator = if args.len() > 1 {
            match &args[1] {
                FactValue::String(s) => s.clone(),
                _ => return Err(anyhow!("array_join() separator must be a string")),
            }
        } else {
            ",".to_string()
        };

        let string_elements: Vec<String> = arr.iter().map(|v| v.as_string()).collect();
        Ok(FactValue::String(string_elements.join(&separator)))
    }

    fn arity(&self) -> Option<usize> {
        None
    } // 1 or 2 arguments
    fn description(&self) -> &'static str {
        "Joins array elements into a string with optional separator"
    }
}

struct ArraySumFunction;
impl CalculatorFunction for ArraySumFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let arr = match &args[0] {
            FactValue::Array(arr) => arr,
            _ => return Err(anyhow!("array_sum() requires an array")),
        };

        let mut sum = 0.0;
        for value in arr {
            match value.as_float() {
                Some(f) => sum += f,
                None => return Err(anyhow!("array_sum() requires numeric array elements")),
            }
        }

        if sum.fract() == 0.0 && sum.abs() <= i64::MAX as f64 {
            Ok(FactValue::Integer(sum as i64))
        } else {
            Ok(FactValue::Float(sum))
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Calculates the sum of numeric array elements"
    }
}

struct ArrayAvgFunction;
impl CalculatorFunction for ArrayAvgFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let arr = match &args[0] {
            FactValue::Array(arr) => arr,
            _ => return Err(anyhow!("array_avg() requires an array")),
        };

        if arr.is_empty() {
            return Ok(FactValue::Null);
        }

        let mut sum = 0.0;
        for value in arr {
            match value.as_float() {
                Some(f) => sum += f,
                None => return Err(anyhow!("array_avg() requires numeric array elements")),
            }
        }

        Ok(FactValue::Float(sum / arr.len() as f64))
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Calculates the average of numeric array elements"
    }
}

struct ArrayMinFunction;
impl CalculatorFunction for ArrayMinFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let arr = match &args[0] {
            FactValue::Array(arr) => arr,
            _ => return Err(anyhow!("array_min() requires an array")),
        };

        if arr.is_empty() {
            return Ok(FactValue::Null);
        }

        let mut min_val = &arr[0];
        for value in &arr[1..] {
            if let (Some(a), Some(b)) = (min_val.to_comparable(), value.to_comparable()) {
                if b < a {
                    min_val = value;
                }
            }
        }

        Ok(min_val.clone())
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the minimum value from an array"
    }
}

struct ArrayMaxFunction;
impl CalculatorFunction for ArrayMaxFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let arr = match &args[0] {
            FactValue::Array(arr) => arr,
            _ => return Err(anyhow!("array_max() requires an array")),
        };

        if arr.is_empty() {
            return Ok(FactValue::Null);
        }

        let mut max_val = &arr[0];
        for value in &arr[1..] {
            if let (Some(a), Some(b)) = (max_val.to_comparable(), value.to_comparable()) {
                if b > a {
                    max_val = value;
                }
            }
        }

        Ok(max_val.clone())
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns the maximum value from an array"
    }
}

// Object functions

struct ObjectKeysFunction;
impl CalculatorFunction for ObjectKeysFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Object(obj) => {
                let keys: Vec<FactValue> =
                    obj.keys().map(|k| FactValue::String(k.clone())).collect();
                Ok(FactValue::Array(keys))
            }
            _ => Err(anyhow!("object_keys() requires an object")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns an array of object keys"
    }
}

struct ObjectValuesFunction;
impl CalculatorFunction for ObjectValuesFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Object(obj) => {
                let values: Vec<FactValue> = obj.values().cloned().collect();
                Ok(FactValue::Array(values))
            }
            _ => Err(anyhow!("object_values() requires an object")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Returns an array of object values"
    }
}

struct ObjectHasKeyFunction;
impl CalculatorFunction for ObjectHasKeyFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let obj = match &args[0] {
            FactValue::Object(obj) => obj,
            _ => {
                return Err(anyhow!(
                    "object_has_key() requires an object as first argument"
                ));
            }
        };

        let key = match &args[1] {
            FactValue::String(key) => key,
            _ => return Err(anyhow!("object_has_key() requires a string key")),
        };

        Ok(FactValue::Boolean(obj.contains_key(key)))
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Checks if an object has a specific key"
    }
}

struct ObjectGetFunction;
impl CalculatorFunction for ObjectGetFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let obj = match &args[0] {
            FactValue::Object(obj) => obj,
            _ => return Err(anyhow!("object_get() requires an object as first argument")),
        };

        let key = match &args[1] {
            FactValue::String(key) => key,
            _ => return Err(anyhow!("object_get() requires a string key")),
        };

        let default_value = if args.len() > 2 {
            &args[2]
        } else {
            &FactValue::Null
        };

        Ok(obj.get(key).cloned().unwrap_or_else(|| default_value.clone()))
    }

    fn arity(&self) -> Option<usize> {
        None
    } // 2 or 3 arguments
    fn description(&self) -> &'static str {
        "Gets a value from an object by key, with optional default"
    }
}

struct ObjectMergeFunction;
impl CalculatorFunction for ObjectMergeFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let mut result = std::collections::HashMap::new();

        for arg in args {
            match arg {
                FactValue::Object(obj) => {
                    for (key, value) in obj {
                        result.insert(key.clone(), value.clone());
                    }
                }
                _ => return Err(anyhow!("object_merge() requires object arguments")),
            }
        }

        Ok(FactValue::Object(result))
    }

    fn arity(&self) -> Option<usize> {
        None
    } // Variadic
    fn description(&self) -> &'static str {
        "Merges multiple objects into one"
    }
}

// Date functions

struct DateNowFunction;
impl CalculatorFunction for DateNowFunction {
    fn call(&self, _args: &[FactValue]) -> Result<FactValue> {
        Ok(FactValue::Date(chrono::Utc::now()))
    }

    fn arity(&self) -> Option<usize> {
        Some(0)
    }
    fn description(&self) -> &'static str {
        "Returns the current date and time"
    }
}

struct DateParseFunction;
impl CalculatorFunction for DateParseFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let date_str = match &args[0] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("date_parse() requires a string")),
        };

        FactValue::date_from_iso(date_str)
            .map_err(|e| anyhow!("Failed to parse date '{}': {}", date_str, e))
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Parses an ISO date string into a date value"
    }
}

struct DateFormatFunction;
impl CalculatorFunction for DateFormatFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let date = match &args[0] {
            FactValue::Date(d) => d,
            _ => return Err(anyhow!("date_format() requires a date as first argument")),
        };

        let format_str = if args.len() > 1 {
            match &args[1] {
                FactValue::String(s) => s,
                _ => return Err(anyhow!("date_format() format must be a string")),
            }
        } else {
            "%Y-%m-%d"
        };

        Ok(FactValue::String(date.format(format_str).to_string()))
    }

    fn arity(&self) -> Option<usize> {
        None
    } // 1 or 2 arguments
    fn description(&self) -> &'static str {
        "Formats a date using a format string"
    }
}

struct DateAddDaysFunction;
impl CalculatorFunction for DateAddDaysFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let date = match &args[0] {
            FactValue::Date(d) => d,
            _ => return Err(anyhow!("date_add_days() requires a date as first argument")),
        };

        let days = match &args[1] {
            FactValue::Integer(i) => *i,
            _ => {
                return Err(anyhow!(
                    "date_add_days() requires an integer number of days"
                ));
            }
        };

        let new_date = *date + chrono::Duration::days(days);
        Ok(FactValue::Date(new_date))
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Adds a number of days to a date"
    }
}

struct DateDiffDaysFunction;
impl CalculatorFunction for DateDiffDaysFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let date1 = match &args[0] {
            FactValue::Date(d) => d,
            _ => return Err(anyhow!("date_diff_days() requires dates as arguments")),
        };

        let date2 = match &args[1] {
            FactValue::Date(d) => d,
            _ => return Err(anyhow!("date_diff_days() requires dates as arguments")),
        };

        let diff = (*date2 - *date1).num_days();
        Ok(FactValue::Integer(diff))
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Calculates the difference in days between two dates"
    }
}

struct DateYearFunction;
impl CalculatorFunction for DateYearFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Date(d) => Ok(FactValue::Integer(d.year() as i64)),
            _ => Err(anyhow!("date_year() requires a date")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Extracts the year from a date"
    }
}

struct DateMonthFunction;
impl CalculatorFunction for DateMonthFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Date(d) => Ok(FactValue::Integer(d.month() as i64)),
            _ => Err(anyhow!("date_month() requires a date")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Extracts the month from a date"
    }
}

struct DateDayFunction;
impl CalculatorFunction for DateDayFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        match &args[0] {
            FactValue::Date(d) => Ok(FactValue::Integer(d.day() as i64)),
            _ => Err(anyhow!("date_day() requires a date")),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }
    fn description(&self) -> &'static str {
        "Extracts the day from a date"
    }
}

// Advanced string functions

struct RegexMatchFunction;
impl CalculatorFunction for RegexMatchFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let text = match &args[0] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("regex_match() requires a string as first argument")),
        };

        let pattern = match &args[1] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("regex_match() requires a string pattern")),
        };

        // Simple pattern matching - in production you'd use regex crate
        let matches = if pattern.contains("*") {
            // Simple wildcard matching
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                text.starts_with(pattern_parts[0]) && text.ends_with(pattern_parts[1])
            } else {
                false
            }
        } else {
            text.contains(pattern)
        };

        Ok(FactValue::Boolean(matches))
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Simple pattern matching (supports wildcards with *)"
    }
}

struct SplitFunction;
impl CalculatorFunction for SplitFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let text = match &args[0] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("split() requires a string as first argument")),
        };

        let delimiter = match &args[1] {
            FactValue::String(s) => s,
            _ => return Err(anyhow!("split() requires a string delimiter")),
        };

        let parts: Vec<FactValue> =
            text.split(delimiter).map(|s| FactValue::String(s.to_string())).collect();

        Ok(FactValue::Array(parts))
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }
    fn description(&self) -> &'static str {
        "Splits a string by a delimiter into an array"
    }
}

struct FormatFunction;
impl CalculatorFunction for FormatFunction {
    fn call(&self, args: &[FactValue]) -> Result<FactValue> {
        let template = match &args[0] {
            FactValue::String(s) => s,
            _ => {
                return Err(anyhow!(
                    "format() requires a string template as first argument"
                ));
            }
        };

        let mut result = template.clone();
        for (i, arg) in args[1..].iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, &arg.as_string());
        }

        Ok(FactValue::String(result))
    }

    fn arity(&self) -> Option<usize> {
        None
    } // Variadic
    fn description(&self) -> &'static str {
        "Formats a string template with arguments using {0}, {1}, etc."
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    #[test]
    fn test_math_functions() {
        let registry = FunctionRegistry::with_builtins();

        // Test abs
        let result = registry.call("abs", &[FactValue::Integer(-5)]).unwrap();
        assert_eq!(result, FactValue::Integer(5));

        // Test min
        let result = registry
            .call(
                "min",
                &[FactValue::Integer(10), FactValue::Integer(5), FactValue::Integer(15)],
            )
            .unwrap();
        assert_eq!(result, FactValue::Integer(5));

        // Test max
        let result = registry
            .call(
                "max",
                &[FactValue::Float(10.5), FactValue::Float(15.2), FactValue::Float(8.1)],
            )
            .unwrap();
        assert_eq!(result, FactValue::Float(15.2));

        // Test round
        let result = registry
            .call(
                "round",
                &[FactValue::Float(std::f64::consts::PI), FactValue::Integer(2)],
            )
            .unwrap();
        // PI rounded to 2 decimal places is 3.14
        assert_eq!(
            result,
            FactValue::Float((std::f64::consts::PI * 100.0).round() / 100.0)
        );
    }

    #[test]
    fn test_string_functions() {
        let registry = FunctionRegistry::with_builtins();

        // Test len
        let result = registry.call("len", &[FactValue::String("hello".to_string())]).unwrap();
        assert_eq!(result, FactValue::Integer(5));

        // Test upper
        let result = registry.call("upper", &[FactValue::String("hello".to_string())]).unwrap();
        assert_eq!(result, FactValue::String("HELLO".to_string()));

        // Test substring
        let result = registry
            .call(
                "substring",
                &[
                    FactValue::String("hello world".to_string()),
                    FactValue::Integer(6),
                    FactValue::Integer(11),
                ],
            )
            .unwrap();
        assert_eq!(result, FactValue::String("world".to_string()));
    }

    #[test]
    fn test_utility_functions() {
        let registry = FunctionRegistry::with_builtins();

        // Test if function
        let result = registry
            .call(
                "if",
                &[
                    FactValue::Boolean(true),
                    FactValue::String("yes".to_string()),
                    FactValue::String("no".to_string()),
                ],
            )
            .unwrap();
        assert_eq!(result, FactValue::String("yes".to_string()));

        // Test coalesce
        let result = registry
            .call(
                "coalesce",
                &[FactValue::String("".to_string()), FactValue::String("default".to_string())],
            )
            .unwrap();
        assert_eq!(result, FactValue::String("default".to_string()));
    }

    #[test]
    fn test_conversion_functions() {
        let registry = FunctionRegistry::with_builtins();

        // Test to_int
        let result = registry.call("to_int", &[FactValue::String("42".to_string())]).unwrap();
        assert_eq!(result, FactValue::Integer(42));

        // Test to_float
        let result = registry.call("to_float", &[FactValue::Integer(42)]).unwrap();
        assert_eq!(result, FactValue::Float(42.0));

        // Test to_string
        let result = registry.call("to_string", &[FactValue::Boolean(true)]).unwrap();
        assert_eq!(result, FactValue::String("true".to_string()));
    }

    #[test]
    fn test_error_handling() {
        let registry = FunctionRegistry::with_builtins();

        // Unknown function
        let result = registry.call("unknown", &[]);
        assert!(result.is_err());

        // Wrong arity
        let result = registry.call("abs", &[]);
        assert!(result.is_err());

        // Wrong argument type
        let result = registry.call("abs", &[FactValue::String("not a number".to_string())]);
        assert!(result.is_err());
    }
}

// ============================================================================
// Cross-Fact Aggregation Functions
// ============================================================================

/// Count all facts in the current evaluation context
struct FactCountFunction;

impl ContextAwareFunction for FactCountFunction {
    fn call_with_context(
        &self,
        _args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        Ok(FactValue::Integer(context.facts.len() as i64))
    }

    fn arity(&self) -> Option<usize> {
        Some(0)
    }

    fn description(&self) -> &'static str {
        "Count the number of facts in the current context"
    }
}

/// Sum a field across all facts
struct FactSumFunction;

impl ContextAwareFunction for FactSumFunction {
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        let field_name = match &args[0] {
            FactValue::String(name) => name,
            _ => return Err(anyhow!("Field name must be a string")),
        };

        let mut sum = 0.0;
        for fact in context.facts {
            if let Some(value) = fact.data.fields.get(field_name) {
                match value {
                    FactValue::Integer(i) => sum += *i as f64,
                    FactValue::Float(f) => sum += f,
                    _ => continue,
                }
            }
        }

        Ok(FactValue::Float(sum))
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }

    fn description(&self) -> &'static str {
        "Sum a numeric field across all facts"
    }
}

/// Average a field across all facts
struct FactAvgFunction;

impl ContextAwareFunction for FactAvgFunction {
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        let field_name = match &args[0] {
            FactValue::String(name) => name,
            _ => return Err(anyhow!("Field name must be a string")),
        };

        let mut sum = 0.0;
        let mut count = 0;
        for fact in context.facts {
            if let Some(value) = fact.data.fields.get(field_name) {
                match value {
                    FactValue::Integer(i) => {
                        sum += *i as f64;
                        count += 1;
                    }
                    FactValue::Float(f) => {
                        sum += f;
                        count += 1;
                    }
                    _ => continue,
                }
            }
        }

        if count == 0 {
            Ok(FactValue::Float(0.0))
        } else {
            Ok(FactValue::Float(sum / count as f64))
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }

    fn description(&self) -> &'static str {
        "Average a numeric field across all facts"
    }
}

/// Find minimum value of a field across all facts
struct FactMinFunction;

impl ContextAwareFunction for FactMinFunction {
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        let field_name = match &args[0] {
            FactValue::String(name) => name,
            _ => return Err(anyhow!("Field name must be a string")),
        };

        let mut min_val: Option<f64> = None;
        for fact in context.facts {
            if let Some(value) = fact.data.fields.get(field_name) {
                let numeric_val = match value {
                    FactValue::Integer(i) => *i as f64,
                    FactValue::Float(f) => *f,
                    _ => continue,
                };
                min_val = Some(min_val.map_or(numeric_val, |current| current.min(numeric_val)));
            }
        }

        match min_val {
            Some(val) => Ok(FactValue::Float(val)),
            None => Err(anyhow!(
                "No numeric values found for field '{}'",
                field_name
            )),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }

    fn description(&self) -> &'static str {
        "Find minimum value of a numeric field across all facts"
    }
}

/// Find maximum value of a field across all facts
struct FactMaxFunction;

impl ContextAwareFunction for FactMaxFunction {
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        let field_name = match &args[0] {
            FactValue::String(name) => name,
            _ => return Err(anyhow!("Field name must be a string")),
        };

        let mut max_val: Option<f64> = None;
        for fact in context.facts {
            if let Some(value) = fact.data.fields.get(field_name) {
                let numeric_val = match value {
                    FactValue::Integer(i) => *i as f64,
                    FactValue::Float(f) => *f,
                    _ => continue,
                };
                max_val = Some(max_val.map_or(numeric_val, |current| current.max(numeric_val)));
            }
        }

        match max_val {
            Some(val) => Ok(FactValue::Float(val)),
            None => Err(anyhow!(
                "No numeric values found for field '{}'",
                field_name
            )),
        }
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }

    fn description(&self) -> &'static str {
        "Find maximum value of a numeric field across all facts"
    }
}

/// Get a field value from a specific fact by ID
struct FactFieldFunction;

impl ContextAwareFunction for FactFieldFunction {
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        let fact_id = match &args[0] {
            FactValue::Integer(id) => *id as u64,
            FactValue::String(id_str) => id_str.parse::<u64>()?,
            _ => return Err(anyhow!("Fact ID must be an integer or string")),
        };

        let field_name = match &args[1] {
            FactValue::String(name) => name,
            _ => return Err(anyhow!("Field name must be a string")),
        };

        for fact in context.facts {
            if fact.id == fact_id {
                if let Some(value) = fact.data.fields.get(field_name) {
                    return Ok(value.clone());
                } else {
                    return Err(anyhow!(
                        "Field '{}' not found on fact {}",
                        field_name,
                        fact_id
                    ));
                }
            }
        }

        Err(anyhow!("Fact with ID {} not found", fact_id))
    }

    fn arity(&self) -> Option<usize> {
        Some(2)
    }

    fn description(&self) -> &'static str {
        "Get a field value from a specific fact by ID"
    }
}

/// Check if a fact with a specific ID exists
struct FactExistsFunction;

impl ContextAwareFunction for FactExistsFunction {
    fn call_with_context(
        &self,
        args: &[FactValue],
        context: &EvaluationContext,
    ) -> Result<FactValue> {
        let fact_id = match &args[0] {
            FactValue::Integer(id) => *id as u64,
            FactValue::String(id_str) => id_str.parse::<u64>()?,
            _ => return Err(anyhow!("Fact ID must be an integer or string")),
        };

        let exists = context.facts.iter().any(|fact| fact.id == fact_id);
        Ok(FactValue::Boolean(exists))
    }

    fn arity(&self) -> Option<usize> {
        Some(1)
    }

    fn description(&self) -> &'static str {
        "Check if a fact with the given ID exists in the context"
    }
}
