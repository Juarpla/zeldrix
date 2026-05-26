// src-tauri/src/merge_engine/parser.rs

//! Parsing utilities for template placeholders.

/// Regex pattern matching {{variable_name}} where variable_name is alphanumeric with underscores.
/// Using lazy_static for efficient regex compilation once.
pub mod re {
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        /// Matches template placeholders like {{variable_name}}.
        /// Captures the variable name without the braces.
        pub static ref PLACEHOLDER_RE: Regex = Regex::new(r"\{\{([a-zA-Z0-9_]+)\}\}").expect("Invalid regex pattern");

        /// Matches any remaining raw placeholder pattern for validation.
        pub static ref RAW_TOKEN_RE: Regex = Regex::new(r"\{\{[a-zA-Z0-9_]*\}\}").expect("Invalid regex pattern");
    }
}

/// Extracts all variable names from a template string.
///
/// # Examples
///
/// ```
/// use merge_engine::parser::extract_variables;
///
/// let template = "Hello {{name}}, your order {{order_id}} is ready";
/// let vars = extract_variables(template);
/// assert_eq!(vars, vec!["name", "order_id"]);
/// ```
pub fn extract_variables(template: &str) -> Vec<&str> {
    re::PLACEHOLDER_RE
        .captures_iter(template)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str()))
        .collect()
}

/// Checks if the template contains any unresolved placeholders.
///
/// Returns `true` if raw `{{...}}` tokens are still present.
pub fn has_unresolved_placeholders(template: &str) -> bool {
    re::RAW_TOKEN_RE.is_match(template)
}

/// Returns an iterator over all placeholder matches with their positions.
pub fn placeholders<'a>(template: &'a str) -> impl Iterator<Item = PlaceholderMatch<'a>> {
    re::PLACEHOLDER_RE.captures_iter(template).map(|caps| {
        let full_match = caps.get(0).unwrap();
        let name = caps.get(1).unwrap().as_str();
        PlaceholderMatch {
            start: full_match.start(),
            end: full_match.end(),
            name,
        }
    })
}

/// Information about a placeholder match in the template.
#[derive(Debug, Clone, Copy)]
pub struct PlaceholderMatch<'a> {
    /// Start byte offset of the placeholder.
    pub start: usize,
    /// End byte offset of the placeholder.
    pub end: usize,
    /// The variable name (without braces).
    pub name: &'a str,
}

/// Validates that a variable name conforms to the expected format.
pub fn is_valid_variable_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_variables_finds_all_placeholders() {
        let template = "Hello {{name}}, your order {{order_id}} is ready";
        let vars = extract_variables(template);
        assert_eq!(vars, vec!["name", "order_id"]);
    }

    #[test]
    fn extract_variables_handles_duplicate_names() {
        let template = "{{name}} and {{name}} again";
        let vars = extract_variables(template);
        assert_eq!(vars, vec!["name", "name"]);
    }

    #[test]
    fn extract_variables_returns_empty_for_no_placeholders() {
        let template = "No placeholders here";
        let vars = extract_variables(template);
        assert!(vars.is_empty());
    }

    #[test]
    fn extract_variables_handles_complex_template() {
        let template = "Estimado {{client_name}},\n\nSu solicitud por ${{amount}} ha sido {{status}}.";
        let vars = extract_variables(template);
        assert_eq!(vars, vec!["client_name", "amount", "status"]);
    }

    #[test]
    fn has_unresolved_placeholders_returns_true_when_present() {
        let template = "Hello {{name}}, your order is {{order_id}}";
        assert!(has_unresolved_placeholders(template));
    }

    #[test]
    fn has_unresolved_placeholders_returns_false_when_resolved() {
        let template = "Hello Juan, your order is 12345";
        assert!(!has_unresolved_placeholders(template));
    }

    #[test]
    fn has_unresolved_placeholders_handles_empty_braces() {
        let template = "Hello {{}}";
        assert!(has_unresolved_placeholders(template));
    }

    #[test]
    fn is_valid_variable_name_accepts_valid_names() {
        assert!(is_valid_variable_name("name"));
        assert!(is_valid_variable_name("order_id"));
        assert!(is_valid_variable_name("var123"));
        assert!(is_valid_variable_name("_private"));
    }

    #[test]
    fn is_valid_variable_name_rejects_invalid_names() {
        assert!(!is_valid_variable_name(""));
        assert!(!is_valid_variable_name("foo-bar"));
        assert!(!is_valid_variable_name("foo.bar"));
        assert!(!is_valid_variable_name("foo bar"));
    }

    #[test]
    fn placeholders_returns_correct_positions() {
        let template = "Hello {{name}}";
        let matches: Vec<_> = placeholders(template).collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "name");
        assert_eq!(matches[0].start, 6);
        assert_eq!(matches[0].end, 14); // {{name}} is 8 chars
    }
}