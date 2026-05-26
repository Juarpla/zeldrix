//! Schema definitions for function calling tools
//!
//! Defines the JSON schema for web_search tool that Gemma 4 can call
//! when low confidence or real-time data is detected.

use serde::{Deserialize, Serialize};

/// Tool definition for JSON schema export
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    #[serde(rename = "function")]
    pub function: FunctionDefinition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: std::collections::HashMap<String, PropertySchema>,
    pub required: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
}

/// Returns the web_search tool definition for Gemma 4
pub fn get_web_search_tool() -> ToolDefinition {
    let mut properties = std::collections::HashMap::new();
    properties.insert(
        "query".to_string(),
        PropertySchema {
            prop_type: "string".to_string(),
            description: "The search query to look up on the web. Be specific and concise.".to_string(),
        },
    );

    ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: "web_search".to_string(),
            description: "Search the web for current information. Use when the user's query requires real-time data, stock prices, weather, news, or any information that may have changed after your training date. The query should be concise and focused.".to_string(),
            parameters: FunctionParameters {
                param_type: "object".to_string(),
                properties,
                required: vec!["query".to_string()],
            },
        },
    }
}

/// System prompt fragment to inject for low-confidence detection
pub fn get_low_confidence_prompt() -> &'static str {
    r#"When answering questions about current events, stock prices, weather, sports scores, or any information that may have changed after your training data, you MUST use the web_search function.
If you are unsure about the current value of something, use web_search.
Always cite the source and date of the information you provide."#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_tool_schema() {
        let tool = get_web_search_tool();
        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.function.name, "web_search");
        assert!(tool.function.parameters.required.contains(&"query".to_string()));
    }
}