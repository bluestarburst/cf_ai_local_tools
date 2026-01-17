//! Configuration validation system

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// Configuration validator with comprehensive validation rules
pub struct ConfigValidator {
    custom_validators: std::collections::HashMap<String, Box<dyn CustomValidator>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: String,
    pub parameters: Value,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub rule: String,
    pub message: String,
}

pub trait CustomValidator: Send + Sync {
    fn validate(&self, value: &Value, params: &Value) -> crate::core::Result<bool>;
    fn error_message(&self, field: &str, params: &Value) -> String;
}

impl ConfigValidator {
    pub fn new() -> Self {
        Self {
            custom_validators: std::collections::HashMap::new(),
        }
    }

    /// Add a custom validator
    pub fn add_validator(&mut self, name: String, validator: Box<dyn CustomValidator>) {
        self.custom_validators.insert(name, validator);
    }

    /// Validate configuration against rules
    pub fn validate(&self, config: &Value, rules: &[ValidationRule]) -> ValidationResult {
        let mut errors = Vec::new();

        for rule in rules {
            if let Err(error) = self.validate_rule(config, rule) {
                errors.push(error);
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    /// Validate a single rule
    fn validate_rule(&self, config: &Value, rule: &ValidationRule) -> Result<(), ValidationError> {
        let field_value = config.get(&rule.field);

        match rule.rule_type.as_str() {
            "required" => self.validate_required(field_value, rule),
            "type" => self.validate_type(field_value, rule),
            "range" => self.validate_range(field_value, rule),
            "enum" => self.validate_enum(field_value, rule),
            "pattern" => self.validate_pattern(field_value, rule),
            "length" => self.validate_length(field_value, rule),
            "custom" => self.validate_custom(field_value, rule),
            _ => {
                // Unknown rule type - ignore or warn
                Ok(())
            }
        }
    }

    fn validate_required(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        if value.is_none() {
            let message = rule
                .message
                .clone()
                .unwrap_or_else(|| format!("Field '{}' is required", rule.field));
            return Err(ValidationError {
                field: rule.field.clone(),
                rule: rule.rule_type.clone(),
                message,
            });
        }
        Ok(())
    }

    fn validate_type(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        let Some(value) = value else { return Ok(()) };

        let expected_type = rule.parameters.as_str().ok_or_else(|| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: "Invalid type parameter".to_string(),
        })?;

        let is_valid = match expected_type {
            "string" => value.is_string(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => false,
        };

        if !is_valid {
            let message = rule.message.clone().unwrap_or_else(|| {
                format!("Field '{}' must be of type {}", rule.field, expected_type)
            });
            return Err(ValidationError {
                field: rule.field.clone(),
                rule: rule.rule_type.clone(),
                message,
            });
        }

        Ok(())
    }

    fn validate_range(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        let Some(value) = value else { return Ok(()) };

        let num = value.as_f64().ok_or_else(|| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: format!(
                "Field '{}' must be a number for range validation",
                rule.field
            ),
        })?;

        if let Some(min) = rule.parameters.get("min").and_then(|v| v.as_f64()) {
            if num < min {
                let message = rule.message.clone().unwrap_or_else(|| {
                    format!(
                        "Field '{}' value {} is below minimum {}",
                        rule.field, num, min
                    )
                });
                return Err(ValidationError {
                    field: rule.field.clone(),
                    rule: rule.rule_type.clone(),
                    message,
                });
            }
        }

        if let Some(max) = rule.parameters.get("max").and_then(|v| v.as_f64()) {
            if num > max {
                let message = rule.message.clone().unwrap_or_else(|| {
                    format!(
                        "Field '{}' value {} is above maximum {}",
                        rule.field, num, max
                    )
                });
                return Err(ValidationError {
                    field: rule.field.clone(),
                    rule: rule.rule_type.clone(),
                    message,
                });
            }
        }

        Ok(())
    }

    fn validate_enum(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        let Some(value) = value else { return Ok(()) };

        let allowed_values = rule.parameters.as_array().ok_or_else(|| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: "Enum rule requires array of allowed values".to_string(),
        })?;

        let allowed_set: HashSet<&Value> = allowed_values.iter().collect();
        if !allowed_set.contains(value) {
            let message = rule.message.clone().unwrap_or_else(|| {
                format!(
                    "Field '{}' value {} is not in allowed values",
                    rule.field, value
                )
            });
            return Err(ValidationError {
                field: rule.field.clone(),
                rule: rule.rule_type.clone(),
                message,
            });
        }

        Ok(())
    }

    fn validate_pattern(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        let Some(value) = value else { return Ok(()) };

        let pattern = rule.parameters.as_str().ok_or_else(|| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: "Pattern rule requires string parameter".to_string(),
        })?;

        let text = value.as_str().ok_or_else(|| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: format!(
                "Field '{}' must be a string for pattern validation",
                rule.field
            ),
        })?;

        let regex = regex::Regex::new(pattern).map_err(|_| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: format!("Invalid regex pattern: {}", pattern),
        })?;

        if !regex.is_match(text) {
            let message = rule.message.clone().unwrap_or_else(|| {
                format!("Field '{}' does not match pattern {}", rule.field, pattern)
            });
            return Err(ValidationError {
                field: rule.field.clone(),
                rule: rule.rule_type.clone(),
                message,
            });
        }

        Ok(())
    }

    fn validate_length(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        let Some(value) = value else { return Ok(()) };

        let length = match value {
            Value::String(s) => s.len(),
            Value::Array(a) => a.len(),
            Value::Object(o) => o.len(),
            _ => {
                return Err(ValidationError {
                    field: rule.field.clone(),
                    rule: rule.rule_type.clone(),
                    message: format!(
                        "Field '{}' type does not support length validation",
                        rule.field
                    ),
                });
            }
        };

        let length_num = length as f64;

        if let Some(min) = rule.parameters.get("min").and_then(|v| v.as_f64()) {
            if length_num < min {
                let message = rule.message.clone().unwrap_or_else(|| {
                    format!(
                        "Field '{}' length {} is below minimum {}",
                        rule.field, length, min
                    )
                });
                return Err(ValidationError {
                    field: rule.field.clone(),
                    rule: rule.rule_type.clone(),
                    message,
                });
            }
        }

        if let Some(max) = rule.parameters.get("max").and_then(|v| v.as_f64()) {
            if length_num > max {
                let message = rule.message.clone().unwrap_or_else(|| {
                    format!(
                        "Field '{}' length {} is above maximum {}",
                        rule.field, length, max
                    )
                });
                return Err(ValidationError {
                    field: rule.field.clone(),
                    rule: rule.rule_type.clone(),
                    message,
                });
            }
        }

        Ok(())
    }

    fn validate_custom(
        &self,
        value: Option<&Value>,
        rule: &ValidationRule,
    ) -> Result<(), ValidationError> {
        let validator_name = rule.parameters.as_str().ok_or_else(|| ValidationError {
            field: rule.field.clone(),
            rule: rule.rule_type.clone(),
            message: "Custom rule requires validator name".to_string(),
        })?;

        let validator =
            self.custom_validators
                .get(validator_name)
                .ok_or_else(|| ValidationError {
                    field: rule.field.clone(),
                    rule: rule.rule_type.clone(),
                    message: format!("Unknown custom validator: {}", validator_name),
                })?;

        let is_valid = validator
            .validate(value.unwrap_or(&Value::Null), &rule.parameters)
            .map_err(|e| ValidationError {
                field: rule.field.clone(),
                rule: rule.rule_type.clone(),
                message: format!("Validator error: {}", e),
            })?;
        if !is_valid {
            let message = rule
                .message
                .clone()
                .unwrap_or_else(|| validator.error_message(&rule.field, &rule.parameters));
            return Err(ValidationError {
                field: rule.field.clone(),
                rule: rule.rule_type.clone(),
                message,
            });
        }

        Ok(())
    }
}

/// Built-in custom validators
pub struct UrlValidator;

impl CustomValidator for UrlValidator {
    fn validate(&self, value: &Value, _params: &Value) -> crate::core::Result<bool> {
        if let Some(url) = value.as_str() {
            Ok(url.starts_with("http://") || url.starts_with("https://"))
        } else {
            Ok(false)
        }
    }

    fn error_message(&self, field: &str, _params: &Value) -> String {
        format!("Field '{}' must be a valid HTTP/HTTPS URL", field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validation() {
        let validator = ConfigValidator::new();

        let rule = ValidationRule {
            field: "name".to_string(),
            rule_type: "required".to_string(),
            parameters: Value::Null,
            message: None,
        };

        // Missing field
        let config = serde_json::json!({});
        let result = validator.validate(&config, &[rule.clone()]);
        assert!(!result.is_valid);

        // Present field
        let config = serde_json::json!({"name": "test"});
        let result = validator.validate(&config, &[rule]);
        assert!(result.is_valid);
    }

    #[test]
    fn test_type_validation() {
        let validator = ConfigValidator::new();

        let rule = ValidationRule {
            field: "count".to_string(),
            rule_type: "type".to_string(),
            parameters: Value::String("number".to_string()),
            message: None,
        };

        // Correct type
        let config = serde_json::json!({"count": 42});
        let result = validator.validate(&config, &[rule.clone()]);
        assert!(result.is_valid);

        // Wrong type
        let config = serde_json::json!({"count": "42"});
        let result = validator.validate(&config, &[rule]);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_range_validation() {
        let validator = ConfigValidator::new();

        let rule = ValidationRule {
            field: "count".to_string(),
            rule_type: "range".to_string(),
            parameters: serde_json::json!({"min": 0, "max": 100}),
            message: None,
        };

        // Valid range
        let config = serde_json::json!({"count": 50});
        let result = validator.validate(&config, &[rule.clone()]);
        assert!(result.is_valid);

        // Below minimum
        let config = serde_json::json!({"count": -1});
        let result = validator.validate(&config, &[rule.clone()]);
        assert!(!result.is_valid);

        // Above maximum
        let config = serde_json::json!({"count": 101});
        let result = validator.validate(&config, &[rule]);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_enum_validation() {
        let validator = ConfigValidator::new();

        let rule = ValidationRule {
            field: "color".to_string(),
            rule_type: "enum".to_string(),
            parameters: serde_json::json!(["red", "green", "blue"]),
            message: None,
        };

        // Valid enum value
        let config = serde_json::json!({"color": "red"});
        let result = validator.validate(&config, &[rule.clone()]);
        assert!(result.is_valid);

        // Invalid enum value
        let config = serde_json::json!({"color": "yellow"});
        let result = validator.validate(&config, &[rule]);
        assert!(!result.is_valid);
    }
}
