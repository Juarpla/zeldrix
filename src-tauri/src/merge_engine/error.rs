// src-tauri/src/merge_engine/error.rs

//! Error types for the merge engine.

use thiserror::Error;

/// Errors that can occur during template merge operations.
#[derive(Debug, Error)]
pub enum MergeError {
    /// A required variable was not provided in the Variables map.
    #[error("Variable '{0}' not provided")]
    MissingVariable(String),

    /// The template contains unresolved placeholders after merge.
    #[error("Template contains unresolved placeholders: {0}")]
    UnresolvedPlaceholders(String),

    /// Invalid variable name format.
    #[error("Invalid variable name format: '{0}'")]
    InvalidVariableName(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_variable_error_message() {
        let err = MergeError::MissingVariable("client_name".to_string());
        assert_eq!(err.to_string(), "Variable 'client_name' not provided");
    }

    #[test]
    fn unresolved_placeholders_error_message() {
        let err = MergeError::UnresolvedPlaceholders("{{foo}}, {{bar}}".to_string());
        assert_eq!(
            err.to_string(),
            "Template contains unresolved placeholders: {{foo}}, {{bar}}"
        );
    }

    #[test]
    fn invalid_variable_name_error_message() {
        let err = MergeError::InvalidVariableName("foo-bar".to_string());
        assert_eq!(
            err.to_string(),
            "Invalid variable name format: 'foo-bar'"
        );
    }
}