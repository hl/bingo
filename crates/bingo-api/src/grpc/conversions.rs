use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::generated::*;
use bingo_core::{
    Action as CoreAction, ActionResult as CoreActionResult, ActionType as CoreActionType,
    Condition as CoreCondition, Fact as CoreFact, FactData as CoreFactData,
    FactValue as CoreFactValue, LogicalOperator as CoreLogicalOperator, Operator, Rule as CoreRule,
    RuleExecutionResult as CoreResult,
};

pub fn from_proto_fact(proto_fact: Fact) -> Result<CoreFact> {
    let mut fields = HashMap::new();

    for (key, value) in proto_fact.data {
        let core_value = from_proto_value(value)?;
        fields.insert(key, core_value);
    }

    Ok(CoreFact {
        id: proto_fact.id.parse().unwrap_or(0),
        external_id: Some(proto_fact.id.clone()),
        timestamp: chrono::DateTime::from_timestamp(proto_fact.created_at, 0)
            .unwrap_or_else(chrono::Utc::now),
        data: CoreFactData { fields },
    })
}

pub fn from_proto_value(value: Value) -> Result<CoreFactValue> {
    match value.value {
        Some(value::Value::StringValue(s)) => Ok(CoreFactValue::String(s)),
        Some(value::Value::NumberValue(n)) => Ok(CoreFactValue::Float(n)),
        Some(value::Value::BoolValue(b)) => Ok(CoreFactValue::Boolean(b)),
        Some(value::Value::IntValue(i)) => Ok(CoreFactValue::Integer(i)),
        None => Ok(CoreFactValue::Null),
    }
}

pub fn to_proto_value(core_value: &CoreFactValue) -> Value {
    let value = match core_value {
        CoreFactValue::String(s) => value::Value::StringValue(s.clone()),
        CoreFactValue::Integer(i) => value::Value::IntValue(*i),
        CoreFactValue::Float(f) => value::Value::NumberValue(*f),
        CoreFactValue::Boolean(b) => value::Value::BoolValue(*b),
        CoreFactValue::Date(dt) => value::Value::StringValue(dt.to_rfc3339()),
        CoreFactValue::Null => value::Value::StringValue("null".to_string()),
        CoreFactValue::Array(_) | CoreFactValue::Object(_) => {
            // For complex types, serialize to JSON string for now
            value::Value::StringValue(serde_json::to_string(core_value).unwrap_or_default())
        }
    };

    Value { value: Some(value) }
}

pub fn to_proto_fact(core_fact: &CoreFact) -> Fact {
    let mut data = HashMap::new();

    for (key, value) in &core_fact.data.fields {
        data.insert(key.clone(), to_proto_value(value));
    }

    Fact {
        id: core_fact.external_id.clone().unwrap_or_else(|| core_fact.id.to_string()),
        data,
        created_at: core_fact.timestamp.timestamp(),
    }
}

pub fn from_proto_rule(proto_rule: Rule) -> Result<CoreRule> {
    let conditions = proto_rule
        .conditions
        .into_iter()
        .map(from_proto_condition)
        .collect::<Result<Vec<_>>>()?;

    let actions = proto_rule
        .actions
        .into_iter()
        .map(from_proto_action)
        .collect::<Result<Vec<_>>>()?;

    // Convert string ID to u64
    let id = proto_rule
        .id
        .parse::<u64>()
        .map_err(|_| anyhow!("Invalid rule ID: {}", proto_rule.id))?;

    Ok(CoreRule { id, name: proto_rule.name, conditions, actions })
}

pub fn from_proto_condition(proto_condition: Condition) -> Result<CoreCondition> {
    match proto_condition.condition_type {
        Some(condition::ConditionType::Simple(simple)) => {
            let operator = match simple.operator() {
                SimpleOperator::Equal => Operator::Equal,
                SimpleOperator::NotEqual => Operator::NotEqual,
                SimpleOperator::GreaterThan => Operator::GreaterThan,
                SimpleOperator::LessThan => Operator::LessThan,
                SimpleOperator::GreaterThanOrEqual => Operator::GreaterThanOrEqual,
                SimpleOperator::LessThanOrEqual => Operator::LessThanOrEqual,
                SimpleOperator::Contains => Operator::Contains,
                SimpleOperator::StartsWith => Operator::Contains, // Map to available operator
                SimpleOperator::EndsWith => Operator::Contains,   // Map to available operator
            };

            let value = simple.value.ok_or_else(|| anyhow!("Missing value in simple condition"))?;
            let core_value = from_proto_value(value)?;

            Ok(CoreCondition::Simple { field: simple.field, operator, value: core_value })
        }
        Some(condition::ConditionType::Complex(complex)) => {
            let logical_op = match complex.operator() {
                LogicalOperator::And => CoreLogicalOperator::And,
                LogicalOperator::Or => CoreLogicalOperator::Or,
                LogicalOperator::Not => CoreLogicalOperator::Not,
            };

            let sub_conditions = complex
                .conditions
                .into_iter()
                .map(from_proto_condition)
                .collect::<Result<Vec<_>>>()?;

            Ok(CoreCondition::Complex { operator: logical_op, conditions: sub_conditions })
        }
        None => Err(anyhow!("Missing condition type")),
    }
}

pub fn from_proto_action(proto_action: Action) -> Result<CoreAction> {
    match proto_action.action_type {
        Some(action::ActionType::CreateFact(create_fact)) => {
            let mut fields = HashMap::new();
            for (key, value) in create_fact.fields {
                fields.insert(key, from_proto_value(value)?);
            }

            Ok(CoreAction {
                action_type: CoreActionType::CreateFact { data: CoreFactData { fields } },
            })
        }
        Some(action::ActionType::CallCalculator(calc)) => Ok(CoreAction {
            action_type: CoreActionType::CallCalculator {
                calculator_name: calc.calculator_name,
                input_mapping: calc.input_mapping,
                output_field: calc.output_field,
            },
        }),
        Some(action::ActionType::Formula(formula)) => Ok(CoreAction {
            action_type: CoreActionType::Formula {
                expression: formula.formula,
                output_field: formula.output_field,
            },
        }),
        None => Err(anyhow!("Missing action type")),
    }
}

pub fn to_proto_result(core_result: CoreResult) -> Result<RuleExecutionResult> {
    // Create a dummy fact since the core result only has fact_id
    let dummy_fact = Fact {
        id: core_result.fact_id.to_string(),
        data: HashMap::new(),
        created_at: chrono::Utc::now().timestamp(),
    };

    let action_results = core_result
        .actions_executed
        .into_iter()
        .map(to_proto_action_result)
        .collect::<Result<Vec<_>>>()?;

    let mut metadata = HashMap::new();
    metadata.insert("fact_id".to_string(), core_result.fact_id.to_string());

    Ok(RuleExecutionResult {
        rule_id: core_result.rule_id.to_string(),
        rule_name: format!("rule_{}", core_result.rule_id),
        matched_fact: Some(dummy_fact),
        action_results,
        execution_time_ns: 0,
        metadata,
    })
}

pub fn to_proto_action_result(core_action_result: CoreActionResult) -> Result<ActionResult> {
    let (success, error_message, result) = match &core_action_result {
        CoreActionResult::FieldSet { field, value, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "{field}={value}"
            ))),
        ),
        CoreActionResult::CalculatorResult { calculator, result, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "{calculator}:{result}"
            ))),
        ),
        CoreActionResult::Logged { message } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "logged:{message}"
            ))),
        ),
        CoreActionResult::LazyLogged { template, args } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "lazy_logged:{template}:{args:?}"
            ))),
        ),
        CoreActionResult::FactCreated { fact_id, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "fact_created:{fact_id}"
            ))),
        ),
        CoreActionResult::FactUpdated { fact_id, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "fact_updated:{fact_id}"
            ))),
        ),
        CoreActionResult::FactDeleted { fact_id } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "fact_deleted:{fact_id}"
            ))),
        ),
        CoreActionResult::FieldIncremented { field, new_value, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "field_incremented:{field}={new_value}"
            ))),
        ),
        CoreActionResult::ArrayAppended { field, appended_value, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "array_appended:{field}=[{appended_value}]"
            ))),
        ),
        CoreActionResult::NotificationSent { notification_type, .. } => (
            true,
            String::new(),
            Some(action_result::Result::FormulaResult(format!(
                "notification_sent:{notification_type:?}"
            ))),
        ),
    };

    Ok(ActionResult { action_id: "action_0".to_string(), success, error_message, result })
}
