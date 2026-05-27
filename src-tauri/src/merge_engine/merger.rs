// src-tauri/src/merge_engine/merger.rs

//! Core merge logic for replacing placeholders with values.

use std::collections::HashMap;

use crate::merge_engine::error::MergeError;
use super::parser;

/// Performs the merge operation, replacing all placeholders with their values.
///
/// This function preserves the original template's whitespace and typography.
/// It validates that all required variables are provided and that no placeholders
/// remain unresolved after the merge.
///
/// # Errors
///
/// Returns `MergeError::MissingVariable` if a placeholder references a variable
/// that was not provided in the Variables map.
///
/// Returns `MergeError::UnresolvedPlaceholders` if the result still contains
/// unresolved placeholders (should not happen if all variables are provided).
///
/// # Examples
///
/// ```
/// use zeldrix_lib::merge_engine::{merge, Variables};
///
/// let template = "Hello {{name}}!";
/// let mut vars = Variables::new();
/// vars.insert("name", "World");
/// let result = merge(&template, &vars).unwrap();
/// assert_eq!(result, "Hello World!");
/// ```
pub fn perform_merge(template: &str, vars: &HashMap<String, String>) -> Result<String, MergeError> {
    let variable_names = parser::extract_variables(template);

    // Check for missing variables
    for var_name in &variable_names {
        if !vars.contains_key(*var_name) {
            return Err(MergeError::MissingVariable(var_name.to_string()));
        }
    }

    // Perform the replacement
    let result = parser::re::PLACEHOLDER_RE.replace_all(template, |caps: &regex::Captures| {
        let var_name = caps.get(1).unwrap().as_str();
        vars.get(var_name).unwrap().as_str()
    });

    // Final validation: ensure no raw tokens remain
    if parser::has_unresolved_placeholders(&result) {
        // This should not happen if all variables were provided, but defensively check
        let unresolved: Vec<_> = parser::extract_variables(&result);
        return Err(MergeError::UnresolvedPlaceholders(
            unresolved.iter().map(|s| format!("{{{{{s}}}}}", s=s)).collect::<Vec<_>>().join(", ")
        ));
    }

    Ok(result.into_owned())
}

/// Builds a merge result by processing template parts and replacements in order.
///
/// This is an alternative to using `replace_all` directly when more control
/// over the process is needed.
pub struct MergeBuilder<'a> {
    template: &'a str,
    vars: &'a HashMap<String, String>,
    result: String,
    last_end: usize,
}

impl<'a> MergeBuilder<'a> {
    /// Creates a new MergeBuilder for the given template and variables.
    pub fn new(template: &'a str, vars: &'a HashMap<String, String>) -> Self {
        MergeBuilder {
            template,
            vars,
            result: String::with_capacity(template.len()),
            last_end: 0,
        }
    }

    /// Processes the template, replacing all placeholders.
    ///
    /// Returns the merged string or an error if a variable is missing.
    pub fn build(mut self) -> Result<String, MergeError> {
        for placeholder in parser::placeholders(self.template) {
            // Append text before this placeholder
            self.result.push_str(&self.template[self.last_end..placeholder.start]);

            // Get the value and append it
            let value = self.vars.get(placeholder.name)
                .ok_or_else(|| MergeError::MissingVariable(placeholder.name.to_string()))?;
            self.result.push_str(value);

            self.last_end = placeholder.end;
        }

        // Append remaining text after last placeholder
        self.result.push_str(&self.template[self.last_end..]);

        Ok(self.result)
    }
}

#[cfg(test)]
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($key.to_string(), $value.to_string());)*
        m
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perform_merge_replaces_single_variable() {
        let template = "Hello {{name}}!";
        let vars = hashmap! { "name".to_string() => "World".to_string() };
        let result = perform_merge(&template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn perform_merge_replaces_multiple_variables() {
        let template = "{{greeting}} {{name}}!";
        let vars = hashmap! {
            "greeting".to_string() => "Hello".to_string(),
            "name".to_string() => "World".to_string()
        };
        let result = perform_merge(&template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn perform_merge_preserves_whitespace() {
        let template = "  {{indent}}\n    {{nested}}";
        let vars = hashmap! {
            "indent".to_string() => "level1".to_string(),
            "nested".to_string() => "level2".to_string()
        };
        let result = perform_merge(&template, &vars).unwrap();
        assert!(result.starts_with("  level1"));
        assert!(result.contains("\n    level2"));
    }

    #[test]
    fn perform_merge_preserves_newlines() {
        let template = "Line 1\nLine 2\nLine 3";
        let vars = hashmap! { "none".to_string() => "".to_string() };
        let result = perform_merge(&template, &vars).unwrap();
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn perform_merge_handles_special_characters() {
        let template = "Price: {{price}}";
        let vars = hashmap! { "price".to_string() => "$1,234.56".to_string() };
        let result = perform_merge(&template, &vars).unwrap();
        assert_eq!(result, "Price: $1,234.56");
    }

    #[test]
    fn perform_merge_handles_empty_value() {
        let template = "Hello {{name}}, welcome!";
        let vars = hashmap! { "name".to_string() => "".to_string() };
        let result = perform_merge(&template, &vars).unwrap();
        assert_eq!(result, "Hello , welcome!");
    }

    #[test]
    fn perform_merge_returns_error_for_missing_variable() {
        let template = "Hello {{name}}, your order is {{order_id}}";
        let vars = hashmap! { "name".to_string() => "John".to_string() };
        let result = perform_merge(&template, &vars);
        assert!(matches!(result, Err(MergeError::MissingVariable(_))));
        let err = result.unwrap_err();
        assert!(err.to_string().contains("order_id"));
    }

    #[test]
    fn perform_merge_no_raw_tokens_remain() {
        let template = "{{a}} and {{b}} and {{c}}";
        let vars = hashmap! {
            "a".to_string() => "1".to_string(),
            "b".to_string() => "2".to_string(),
            "c".to_string() => "3".to_string()
        };
        let result = perform_merge(&template, &vars).unwrap();
        assert!(!result.contains("{{"));
        assert!(!result.contains("}}"));
        assert_eq!(result, "1 and 2 and 3");
    }

    #[test]
    fn perform_merge_unicode_preserved() {
        let template = "Nombre: {{name}}";
        let vars = hashmap! { "name".to_string() => "José García".to_string() };
        let result = perform_merge(&template, &vars).unwrap();
        assert_eq!(result, "Nombre: José García");
    }

    #[test]
    fn perform_merge_newlines_in_values() {
        let template = "Texto:\n{{multiline}}";
        let vars = hashmap! {
            "multiline".to_string() => "Línea 1\nLínea 2\nLínea 3".to_string()
        };
        let result = perform_merge(&template, &vars).unwrap();
        assert!(result.contains("Texto:\nLínea 1\nLínea 2\nLínea 3"));
    }

    #[test]
    fn merge_builder_produces_same_result() {
        let template = "{{greeting}} {{name}}!";
        let vars = hashmap! {
            "greeting".to_string() => "Hello".to_string(),
            "name".to_string() => "World".to_string()
        };
        let result = MergeBuilder::new(template, &vars).build().unwrap();
        assert_eq!(result, "Hello World!");
    }
}
