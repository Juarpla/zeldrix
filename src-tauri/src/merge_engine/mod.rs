// src-tauri/src/merge_engine/mod.rs

//! # Merge Engine
//!
//! A safe template merging engine for corporate documents.
//!
//! This module provides functionality to replace `{{variable}}` placeholders
//! in template strings with validated values, preserving typography structure.
//!
//! # Example
//!
//! ```
//! use zeldrix_lib::merge_engine::{merge, Variables};
//!
//! let template = "Estimado {{client_name}}:\n\nSu pedido #{{order_id}} está listo.";
//! let mut vars = Variables::new();
//! vars.insert("client_name", "Juan Pérez");
//! vars.insert("order_id", "12345");
//!
//! let result = merge(&template, &vars).unwrap();
//! // result contains "Estimado Juan Pérez:\n\nSu pedido #12345 está listo."
//! ```

mod error;
mod parser;
mod merger;

pub use error::MergeError;
pub use merger::MergeBuilder;

use std::collections::HashMap;

/// A collection of variable key-value pairs for template merging.
///
/// Use `Variables::new()` to create an empty collection,
/// then use `insert()` to add key-value pairs.
#[derive(Debug, Clone, Default)]
pub struct Variables(HashMap<String, String>);

impl Variables {
    /// Creates a new empty Variables collection.
    pub fn new() -> Self {
        Variables(HashMap::new())
    }

    /// Inserts a variable key-value pair.
    ///
    /// Both key and value will be converted to Strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use zeldrix_lib::merge_engine::Variables;
    ///
    /// let mut vars = Variables::new();
    /// vars.insert("name", "John");
    /// vars.insert("age", "30");
    /// ```
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// Gets a variable value by key.
    ///
    /// Returns `None` if the key does not exist.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }

    /// Returns the number of variables in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the collection contains no variables.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the variable key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl From<Variables> for HashMap<String, String> {
    fn from(vars: Variables) -> Self {
        vars.0
    }
}

impl From<HashMap<String, String>> for Variables {
    fn from(map: HashMap<String, String>) -> Self {
        Variables(map)
    }
}

/// Performs template merge with the given variables.
///
/// This is the main entry point for the merge engine.
/// It validates that all placeholders in the template have corresponding
/// variables provided, then performs the replacement safely.
///
/// # Arguments
///
/// * `template` - The template string containing `{{variable}}` placeholders
/// * `vars` - A `Variables` collection mapping variable names to their values
///
/// # Returns
///
/// Returns `Ok(String)` with the merged template, or `Err(MergeError)` if:
/// - A placeholder references a variable that was not provided
/// - The result contains unresolved placeholders (should not happen)
///
/// # Examples
///
/// ```
/// use zeldrix_lib::merge_engine::{merge, Variables};
///
/// let template = "Hello {{name}}!";
/// let mut vars = Variables::new();
/// vars.insert("name", "World");
///
/// let result = merge(&template, &vars).unwrap();
/// assert_eq!(result, "Hello World!");
/// ```
pub fn merge(template: &str, vars: &Variables) -> Result<String, MergeError> {
    merger::perform_merge(template, &vars.0)
}

/// Verifies that a template contains no unresolved placeholders.
///
/// Use this function to check if a template still has raw `{{...}}` tokens
/// that were not replaced during merge.
///
/// # Examples
///
/// ```
/// use zeldrix_lib::merge_engine::has_unresolved_placeholders;
///
/// let unresolved = "Hello {{name}}!";
/// assert!(has_unresolved_placeholders(unresolved));
///
/// let resolved = "Hello World!";
/// assert!(!has_unresolved_placeholders(resolved));
/// ```
pub fn has_unresolved_placeholders(template: &str) -> bool {
    parser::has_unresolved_placeholders(template)
}

/// Extracts all variable names from a template.
///
/// # Examples
///
/// ```
/// use zeldrix_lib::merge_engine::extract_variables;
///
/// let template = "Hello {{name}}, your order {{order_id}} is ready";
/// let vars = extract_variables(template);
/// assert_eq!(vars, vec!["name", "order_id"]);
/// ```
pub fn extract_variables(template: &str) -> Vec<&str> {
    parser::extract_variables(template)
}

/// Validates that a variable name conforms to the expected format.
pub fn is_valid_variable_name(name: &str) -> bool {
    parser::is_valid_variable_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variables_insert_and_get() {
        let mut vars = Variables::new();
        vars.insert("name", "John");
        assert_eq!(vars.get("name"), Some("John"));
        assert_eq!(vars.get("missing"), None);
    }

    #[test]
    fn variables_len_and_is_empty() {
        let mut vars = Variables::new();
        assert!(vars.is_empty());
        assert_eq!(vars.len(), 0);

        vars.insert("key", "value");
        assert!(!vars.is_empty());
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn variables_iter() {
        let mut vars = Variables::new();
        vars.insert("a", "1");
        vars.insert("b", "2");

        let items: Vec<_> = vars.iter().collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&("a", "1")));
        assert!(items.contains(&("b", "2")));
    }

    #[test]
    fn merge_basic_template() {
        let template = "Hello {{name}}!";
        let mut vars = Variables::new();
        vars.insert("name", "World");
        let result = merge(&template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn merge_multiple_variables() {
        let template = "{{greeting}} {{name}}!";
        let mut vars = Variables::new();
        vars.insert("greeting", "Hello");
        vars.insert("name", "World");
        let result = merge(&template, &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn merge_missing_variable_returns_error() {
        let template = "Hello {{name}}, your order {{order_id}} is ready";
        let mut vars = Variables::new();
        vars.insert("name", "John");
        let result = merge(&template, &vars);
        assert!(matches!(result, Err(MergeError::MissingVariable(_))));
    }

    #[test]
    fn merge_no_raw_tokens_remain() {
        let template = "{{a}} and {{b}} and {{c}}";
        let mut vars = Variables::new();
        vars.insert("a", "1");
        vars.insert("b", "2");
        vars.insert("c", "3");
        let result = merge(&template, &vars).unwrap();
        assert!(!result.contains("{{"));
        assert!(!result.contains("}}"));
    }

    #[test]
    fn merge_5_complex_variables() {
        let template = r#"Estimado {{client_name}},

Su solicitud de crédito por ${{amount}} ha sido {{status}}.

Detalles:
- Número de referencia: {{reference_id}}
- Fecha de aprobación: {{approval_date}}
- Monto total: {{total_amount}}

Atentamente,
{{agent_name}}"#;

        let mut vars = Variables::new();
        vars.insert("client_name", "Juan Pérez");
        vars.insert("amount", "50,000.00");
        vars.insert("status", "aprobada");
        vars.insert("reference_id", "CR-2024-001234");
        vars.insert("approval_date", "26 de mayo de 2026");
        vars.insert("total_amount", "$52,500.00");
        vars.insert("agent_name", "María González");

        let result = merge(&template, &vars).unwrap();

        // Verify no raw tokens
        assert!(!result.contains("{{"));
        assert!(!result.contains("}}"));

        // Verify all variables are present
        assert!(result.contains("Juan Pérez"));
        assert!(result.contains("50,000.00"));
        assert!(result.contains("aprobada"));
        assert!(result.contains("CR-2024-001234"));
        assert!(result.contains("26 de mayo de 2026"));
        assert!(result.contains("$52,500.00"));
        assert!(result.contains("María González"));

        // Verify structure preservation
        assert!(result.contains("\n\n")); // Preserves blank lines
        assert!(result.contains("- Número de referencia:")); // Preserves indentation
    }

    #[test]
    fn merge_preserves_typography_with_special_chars() {
        let template = "Precio: {{price}}\nImpuesto: {{tax}}%\nTotal: {{total}}";
        let mut vars = Variables::new();
        vars.insert("price", "$1,234.56");
        vars.insert("tax", "18");
        vars.insert("total", "$1,456.78");

        let result = merge(&template, &vars).unwrap();

        assert_eq!(result, "Precio: $1,234.56\nImpuesto: 18%\nTotal: $1,456.78");
    }

    #[test]
    fn merge_unicode_and_international_characters() {
        let template = "Nombre: {{name}}\nDirección: {{address}}";
        let mut vars = Variables::new();
        vars.insert("name", "José García");
        vars.insert("address", "Calle Mayor #123, Ñuñoa, Santiago");

        let result = merge(&template, &vars).unwrap();

        assert!(result.contains("José García"));
        assert!(result.contains("Calle Mayor #123, Ñuñoa, Santiago"));
    }

    #[test]
    fn merge_with_newlines_in_values() {
        let template = "Descripción:\n{{description}}";
        let mut vars = Variables::new();
        vars.insert("description", "Línea 1\nLínea 2\nLínea 3");

        let result = merge(&template, &vars).unwrap();

        assert!(result.contains("Descripción:\nLínea 1\nLínea 2\nLínea 3"));
    }

    #[test]
    fn merge_empty_value_preserves_context() {
        let template = "Name: {{name}}, Notes: {{notes}}";
        let mut vars = Variables::new();
        vars.insert("name", "John");
        vars.insert("notes", "");

        let result = merge(&template, &vars).unwrap();

        assert_eq!(result, "Name: John, Notes: ");
    }

    #[test]
    fn has_unresolved_placeholders_detects_raw_tokens() {
        assert!(has_unresolved_placeholders("Hello {{name}}"));
        assert!(has_unresolved_placeholders("{{a}} and {{b}}"));
        assert!(!has_unresolved_placeholders("Hello World"));
        assert!(!has_unresolved_placeholders("Already resolved name here"));
    }

    #[test]
    fn extract_variables_finds_all() {
        let template = "{{a}} {{b}} {{c}}";
        let vars = extract_variables(&template);
        assert_eq!(vars, vec!["a", "b", "c"]);
    }
}