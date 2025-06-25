# Calculator DSL User Guide

## Overview

The Calculator DSL provides a business-friendly way to write mathematical and logical expressions for the Bingo Rules Engine. It enables domain experts to author complex calculations without deep technical knowledge while maintaining type safety and performance.

## Quick Start

### Basic Arithmetic

```rust
// Simple calculations
amount * 1.15                    // Apply 15% markup
base_salary * 0.10               // Calculate 10% bonus
hours_worked * hourly_rate       // Calculate gross pay
```

### Comparisons and Logic

```rust
// Boolean logic
performance_rating >= 4.0 && tenure_years > 1
status == "active" || priority_level > 3
!(disabled || suspended)
```

### Conditional Expressions

```rust
// Simple if-then-else
if performance_rating >= 4.0 then bonus_eligible else false
if hours_worked > 40 then overtime_rate else regular_rate
```

## Language Reference

### Data Types

The calculator supports four core data types:

- **Integer**: Whole numbers (`42`, `-17`, `1000`)
- **Float**: Decimal numbers (`3.14`, `-0.5`, `123.456`)
- **String**: Text values (`"active"`, `"John Doe"`, `"Manager"`)
- **Boolean**: True/false values (`true`, `false`)

### Variables

Variables reference fields from the current fact or global values:

```rust
employee_id        // Field from current fact
base_salary        // Numeric field
status            // String field
is_active         // Boolean field
```

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `salary + bonus` |
| `-` | Subtraction | `gross_pay - taxes` |
| `*` | Multiplication | `hours * rate` |
| `/` | Division | `total / count` |
| `%` | Modulo | `employee_id % 10` |
| `**` | Power | `amount ** 2` |

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `status == "active"` |
| `!=` | Not equal | `department != "HR"` |
| `<` | Less than | `age < 65` |
| `<=` | Less than or equal | `hours <= 40` |
| `>` | Greater than | `salary > 50000` |
| `>=` | Greater than or equal | `rating >= 4.0` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `active && eligible` |
| `\|\|` | Logical OR | `manager \|\| director` |
| `!` | Logical NOT | `!suspended` |

### String Operations

| Operator | Description | Example |
|----------|-------------|---------|
| `++` | Concatenation | `first_name ++ " " ++ last_name` |
| `contains` | Contains substring | `email contains "@company.com"` |
| `starts_with` | Starts with prefix | `job_code starts_with "ENG"` |
| `ends_with` | Ends with suffix | `file_name ends_with ".pdf"` |

### Unary Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `-` | Numeric negation | `-adjustment_amount` |
| `!` | Logical negation | `!is_temporary` |

## Advanced Features

### Conditional Expressions

For simple binary decisions:

```rust
if condition then value_if_true else value_if_false
```

Examples:
```rust
if hours_worked > 40 then "overtime" else "regular"
if performance_rating >= 4.5 then base_salary * 1.15 else base_salary
if department == "Sales" then commission_rate else 0.0
```

### Conditional Sets

For multiple conditions with different outcomes:

```rust
cond when condition1 then value1 
     when condition2 then value2 
     when condition3 then value3 
     default default_value
```

Examples:

#### Performance Bonus Calculation
```rust
cond when performance_rating >= 4.5 then 0.15
     when performance_rating >= 4.0 then 0.10  
     when performance_rating >= 3.5 then 0.05
     default 0.0
```

#### Employee Classification
```rust
cond when tenure_years >= 20 && performance_score >= 7 then "senior"
     when tenure_years >= 10 && performance_score >= 5 then "mid"
     when tenure_years >= 3 && performance_score >= 4 then "junior" 
     default "trainee"
```

#### Tax Bracket Calculation
```rust
cond when annual_income >= 200000 then annual_income * 0.35
     when annual_income >= 100000 then annual_income * 0.28
     when annual_income >= 50000 then annual_income * 0.22
     default annual_income * 0.12
```

#### Shipping Cost Calculation
```rust
cond when weight > 50 && priority == "express" then 25.00
     when weight > 50 then 15.00
     when priority == "express" then 12.00
     when order_total >= 100 then 0.0
     default 5.00
```

### Function Calls

The calculator includes built-in functions for common operations:

#### Mathematical Functions
```rust
max(salary, minimum_wage)           // Maximum of two values
min(hours_worked, 40)              // Minimum of two values  
abs(adjustment_amount)             // Absolute value
round(calculated_amount, 2)        // Round to 2 decimal places
floor(bonus_percentage * 100)      // Round down
ceil(days_worked / 7)             // Round up
sqrt(area)                        // Square root
```

#### String Functions
```rust
upper(last_name)                   // Convert to uppercase
lower(email)                       // Convert to lowercase
length(description)                // String length
substring(employee_id, 0, 3)       // Extract substring
```

#### Utility Functions
```rust
coalesce(overtime_rate, regular_rate)  // First non-null value
typeof(field_value)                    // Get type name
```

### Nested Expressions

Expressions can be nested for complex logic:

```rust
// Nested conditional sets
cond when department == "engineering" && level >= 5 then 
         cond when location == "sf" then 200000 
              when location == "ny" then 180000 
              default 150000
     when department == "sales" && level >= 3 then 120000
     default 80000
```

```rust
// Complex calculations
max(
    base_salary * performance_multiplier,
    minimum_salary + (years_experience * annual_increase)
)
```

### Field Access

Access fields from related objects (when supported):

```rust
employee.department          // Access department field
order.customer.credit_limit  // Nested field access
```

## Best Practices

### 1. Use Descriptive Field Names

✅ **Good:**
```rust
performance_rating >= 4.0
annual_salary * bonus_percentage
```

❌ **Avoid:**
```rust
pr >= 4.0
sal * bp
```

### 2. Group Related Conditions

✅ **Good:**
```rust
cond when (department == "sales" || department == "marketing") && performance_rating >= 4.0 then high_bonus
     when department == "engineering" && years_experience >= 5 then tech_bonus
     default standard_bonus
```

### 3. Use Meaningful Default Values

✅ **Good:**
```rust
cond when hours_worked > 40 then overtime_rate
     default regular_rate  // Clear fallback
```

❌ **Avoid:**
```rust
cond when hours_worked > 40 then overtime_rate
     default 0  // Unclear what 0 means
```

### 4. Break Complex Expressions Into Steps

For very complex calculations, consider breaking them into multiple simpler expressions rather than one large nested expression.

✅ **Good:** Use multiple rules with intermediate fields
❌ **Avoid:** Single expression with 5+ levels of nesting

## Common Patterns

### Rate Calculations

```rust
// Hourly rate calculation
hours_worked * hourly_rate

// Overtime calculation  
if hours_worked > 40 then (hours_worked - 40) * overtime_rate else 0

// Commission calculation
if sales_amount >= quota then sales_amount * commission_rate else 0
```

### Tiered Calculations

```rust
// Progressive tax calculation
cond when income > 100000 then income * 0.30
     when income > 50000 then income * 0.25  
     when income > 25000 then income * 0.20
     default income * 0.15
```

### Eligibility Checks

```rust
// Bonus eligibility
performance_rating >= 4.0 && tenure_months >= 12 && !on_probation

// Vacation eligibility  
cond when years_employed >= 10 then 25
     when years_employed >= 5 then 20
     when years_employed >= 1 then 15
     default 10
```

### Date and Time Calculations

```rust
// Age calculation (if date functions available)
years_between(birth_date, current_date)

// Service years
years_between(hire_date, current_date)
```

## Error Handling

### Common Errors

1. **Division by Zero**
```rust
// ❌ This will error if hours_worked is 0
total_amount / hours_worked

// ✅ Better approach
if hours_worked > 0 then total_amount / hours_worked else 0
```

2. **Missing Fields**
```rust
// ❌ Error if field doesn't exist
non_existent_field * 2

// ✅ Use coalesce for defaults
coalesce(optional_field, 0) * 2
```

3. **Type Mismatches**
```rust
// ❌ Can't multiply string by number
employee_name * 2

// ✅ Ensure correct types
length(employee_name) * 2
```

### Conditional Set Without Default

When using conditional sets without a default value, ensure at least one condition will match:

```rust
// ❌ Risky - what if no conditions match?
cond when status == "active" then 1.0
     when status == "pending" then 0.5

// ✅ Safe with default
cond when status == "active" then 1.0
     when status == "pending" then 0.5
     default 0.0
```

## Integration with Rules Engine

### Expression Rules

Calculator expressions integrate with the rules engine through the `Formula` action type:

```json
{
  "conditions": [
    {
      "field": "employee_type",
      "operator": "Equal", 
      "value": "hourly"
    }
  ],
  "actions": [
    {
      "action_type": "Formula",
      "target_field": "gross_pay",
      "expression": "hours_worked * hourly_rate"
    }
  ]
}
```

### Multiple Expressions

Rules can contain multiple formula actions:

```json
{
  "actions": [
    {
      "action_type": "Formula",
      "target_field": "bonus_rate", 
      "expression": "cond when performance >= 4.5 then 0.15 when performance >= 4.0 then 0.10 default 0.0"
    },
    {
      "action_type": "Formula",
      "target_field": "bonus_amount",
      "expression": "base_salary * bonus_rate"
    }
  ]
}
```

## Performance Considerations

### Optimization Tips

1. **Simple expressions are faster** - Avoid unnecessary complexity
2. **Field access is optimized** - Direct field references are efficient  
3. **Function calls have overhead** - Use built-in operators when possible
4. **Conditional sets short-circuit** - Put most likely conditions first

### Example Optimizations

✅ **Optimized:**
```rust
// Most common case first
cond when status == "active" then calculate_active_bonus()
     when status == "pending" then 0
     default 0
```

❌ **Less Efficient:**
```rust  
// Rare case first
cond when status == "suspended" then 0
     when status == "pending" then 0  
     when status == "active" then calculate_active_bonus()
     default 0
```

## Debugging Tips

### Testing Expressions

1. **Start simple** - Test basic arithmetic before adding conditions
2. **Test edge cases** - Zero values, empty strings, null fields
3. **Verify data types** - Ensure fields contain expected types
4. **Check operator precedence** - Use parentheses when in doubt

### Common Debugging Patterns

```rust
// Add debugging output (if supported)
debug("Processing employee: " ++ employee_id)

// Validate inputs
if salary > 0 then salary * rate else debug("Invalid salary: " ++ salary)

// Test conditions separately  
performance_rating >= 4.0  // Test this condition first
tenure_months >= 12        // Then test this one
```

This guide covers the essential features of the Calculator DSL. For more advanced use cases and integration details, refer to the technical specifications and API documentation.