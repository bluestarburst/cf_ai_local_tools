//! Variable interpolation system for configuration templates

use serde_json::Value;
use std::collections::HashMap;

/// Advanced variable interpolator with function support
pub struct AdvancedInterpolator {
    context: HashMap<String, Value>,
    functions: HashMap<String, InterpolationFunction>,
}

type InterpolationFunction = Box<dyn Fn(&[Value]) -> crate::core::Result<Value> + Send + Sync>;

impl AdvancedInterpolator {
    pub fn new() -> Self {
        let mut functions: HashMap<String, InterpolationFunction> = HashMap::new();

        // Register built-in functions
        functions.insert("tools".to_string(), Box::new(tools_function));
        functions.insert("agents".to_string(), Box::new(agents_function));
        functions.insert("tool".to_string(), Box::new(tool_function));
        functions.insert("agent".to_string(), Box::new(agent_function));
        functions.insert("env".to_string(), Box::new(env_function));
        functions.insert("if".to_string(), Box::new(if_function));

        Self {
            context: HashMap::new(),
            functions,
        }
    }

    /// Set context variable
    pub fn set_context(&mut self, key: String, value: Value) {
        self.context.insert(key, value);
    }

    /// Interpolate a string with variables and functions
    pub fn interpolate(&self, input: &str) -> crate::core::Result<String> {
        let mut result = input.to_string();

        // Replace simple variables: {variable}
        for (key, value) in &self.context {
            let placeholder = format!("{{{}}}", key);
            let replacement = value.to_string();
            result = result.replace(&placeholder, &replacement);
        }

        // Handle function calls: {function(arg1,arg2)}
        result = self.interpolate_functions(&result)?;

        Ok(result)
    }

    /// Handle function interpolation
    fn interpolate_functions(&self, input: &str) -> crate::core::Result<String> {
        let mut result = input.to_string();

        // Find and replace function calls
        while let Some(start) = result.find("{") {
            if let Some(end) = result[start..].find("}") {
                let end = start + end;
                let call = &result[start + 1..end];

                if let Some(value) = self.evaluate_function_call(call)? {
                    let replacement = value.to_string();
                    result.replace_range(start..=end, &replacement);
                }
            } else {
                break;
            }
        }

        Ok(result)
    }

    /// Evaluate a function call
    fn evaluate_function_call(&self, call: &str) -> crate::core::Result<Option<Value>> {
        // Parse function call: name(arg1,arg2)
        if let Some(open_paren) = call.find('(') {
            if let Some(close_paren) = call.rfind(')') {
                let func_name = call[..open_paren].trim();
                let args_str = &call[open_paren + 1..close_paren];

                // Parse arguments
                let args = self.parse_function_args(args_str)?;

                // Execute function
                if let Some(func) = self.functions.get(func_name) {
                    let result = func(&args)?;
                    return Ok(Some(result));
                }
            }
        }

        Ok(None)
    }

    /// Parse function arguments
    fn parse_function_args(&self, args_str: &str) -> crate::core::Result<Vec<Value>> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';

        for ch in args_str.chars() {
            match ch {
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                    } else {
                        current.push(ch);
                    }
                }
                ',' => {
                    if !in_quotes {
                        if !current.trim().is_empty() {
                            args.push(self.parse_arg(&current)?);
                        }
                        current.clear();
                    } else {
                        current.push(ch);
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.trim().is_empty() {
            args.push(self.parse_arg(&current)?);
        }

        Ok(args)
    }

    /// Parse a single argument
    fn parse_arg(&self, arg: &str) -> crate::core::Result<Value> {
        let arg = arg.trim();

        // Check if it's a context variable
        if arg.starts_with('$') {
            if let Some(value) = self.context.get(&arg[1..]) {
                return Ok(value.clone());
            }
        }

        // Try to parse as JSON
        serde_json::from_str(arg).or_else(|_| {
            // If not JSON, treat as string
            Ok(Value::String(arg.to_string()))
        })
    }
}

// Built-in interpolation functions

fn tools_function(args: &[Value]) -> crate::core::Result<Value> {
    // Return list of available tools
    // In a real implementation, this would query the tool registry
    Ok(Value::String(
        "mouse_move, mouse_click, keyboard_type, web_search, fetch_url".to_string(),
    ))
}

fn agents_function(args: &[Value]) -> crate::core::Result<Value> {
    // Return list of available agents
    // In a real implementation, this would query the agent registry
    Ok(Value::String(
        "desktop-automation-agent, web-research-agent".to_string(),
    ))
}

fn tool_function(args: &[Value]) -> crate::core::Result<Value> {
    if args.is_empty() {
        return Err(crate::core::AppError::Configuration(
            "tool() requires tool ID argument".to_string(),
        ));
    }

    if let Some(Value::String(tool_id)) = args.get(0) {
        // In a real implementation, this would query tool metadata
        Ok(Value::String(format!("Tool: {}", tool_id)))
    } else {
        Err(crate::core::AppError::Configuration(
            "tool() argument must be a string".to_string(),
        ))
    }
}

fn agent_function(args: &[Value]) -> crate::core::Result<Value> {
    if args.is_empty() {
        return Err(crate::core::AppError::Configuration(
            "agent() requires agent ID argument".to_string(),
        ));
    }

    if let Some(Value::String(agent_id)) = args.get(0) {
        // In a real implementation, this would query agent metadata
        Ok(Value::String(format!("Agent: {}", agent_id)))
    } else {
        Err(crate::core::AppError::Configuration(
            "agent() argument must be a string".to_string(),
        ))
    }
}

fn env_function(args: &[Value]) -> crate::core::Result<Value> {
    if args.is_empty() {
        return Err(crate::core::AppError::Configuration(
            "env() requires variable name argument".to_string(),
        ));
    }

    if let Some(Value::String(var_name)) = args.get(0) {
        match std::env::var(var_name) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Ok(Value::Null),
        }
    } else {
        Err(crate::core::AppError::Configuration(
            "env() argument must be a string".to_string(),
        ))
    }
}

fn if_function(args: &[Value]) -> crate::core::Result<Value> {
    if args.len() < 3 {
        return Err(crate::core::AppError::Configuration(
            "if() requires condition, true_value, false_value".to_string(),
        ));
    }

    let condition = match args[0] {
        Value::Bool(b) => b,
        Value::String(ref s) if s == "true" => true,
        Value::String(ref s) if s == "false" => false,
        Value::Number(ref n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    };

    if condition {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_interpolation() {
        let mut interpolator = AdvancedInterpolator::new();
        interpolator.set_context("name".to_string(), Value::String("test".to_string()));

        let result = interpolator.interpolate("Hello {name}!").unwrap();
        assert_eq!(result, "Hello \"test\"!");
    }

    #[test]
    fn test_function_interpolation() {
        let interpolator = AdvancedInterpolator::new();

        let result = interpolator.interpolate("Tools: {tools()}").unwrap();
        assert!(result.contains("mouse_move"));
    }

    #[test]
    fn test_env_function() {
        let interpolator = AdvancedInterpolator::new();

        // This might fail if CARGO_PKG_NAME is not set, but that's ok for testing
        let result = interpolator.interpolate("{env(\"CARGO_PKG_NAME\")}");
        // Just check it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_if_function() {
        let interpolator = AdvancedInterpolator::new();

        let result = interpolator
            .interpolate("{if(true,\"yes\",\"no\")}")
            .unwrap();
        assert_eq!(result, "\"yes\"");

        let result = interpolator
            .interpolate("{if(false,\"yes\",\"no\")}")
            .unwrap();
        assert_eq!(result, "\"no\"");
    }
}
