//! Tool call handler for function calling
//!
//! Parses tool_calls from model responses and executes the corresponding
//! functions, returning results for context injection.

use serde::{Deserialize, Serialize};

pub use super::web_search::{format_tool_result};
pub use super::schema::get_web_search_tool; // Re-exported for API

/// Maximum iterations in the tool call loop (prevent infinite loops)
pub const MAX_TOOL_CALL_ITERATIONS: usize = 3;

/// Represents a tool call from the model
#[derive(Clone, Debug, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,  // JSON string of arguments
}

/// Parse a JSON string into a ToolCall
pub fn parse_tool_call(raw: &serde_json::Value) -> Option<ToolCall> {
    serde_json::from_value(raw.clone()).ok()
}

/// Parse arguments from a tool call
pub fn parse_arguments<T: for<'de> Deserialize<'de>>(arguments: &str) -> Result<T, String> {
    serde_json::from_str(arguments).map_err(|e| format!("Failed to parse arguments: {}", e))
}

/// Execute a tool call and return the result
pub async fn execute_tool_call(tool_call: &ToolCall) -> Result<serde_json::Value, String> {
    match tool_call.function.name.as_str() {
        "web_search" => {
            let args: WebSearchArgs = parse_arguments(&tool_call.function.arguments)?;
            let result = super::web_search::web_search(&args.query).await
                .map_err(|e| format!("Web search failed: {:?}", e))?;
            Ok(format_tool_result(&tool_call.id, &result))
        }
        other => Err(format!("Unknown tool: {}", other)),
    }
}

/// Arguments for web_search function
#[derive(Clone, Debug, Deserialize)]
pub struct WebSearchArgs {
    pub query: String,
}

/// Check if a response contains tool calls
pub fn has_tool_calls(response: &serde_json::Value) -> bool {
    response
        .pointer("/choices/0/message/tool_calls")
        .map(|v| v.is_array() && !v.as_array().unwrap().is_empty())
        .unwrap_or(false)
}

/// Extract tool calls from a response
pub fn extract_tool_calls(response: &serde_json::Value) -> Vec<ToolCall> {
    let tool_calls = match response.pointer("/choices/0/message/tool_calls") {
        Some(v) => v,
        None => return Vec::new(),
    };

    let array = match tool_calls.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    array
        .iter()
        .filter_map(|v| parse_tool_call(v))
        .collect()
}

/// Extract content from a response (non-tool call response)
pub fn extract_content(response: &serde_json::Value) -> Option<String> {
    response
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Build a tool result message for re-injection
#[derive(Clone, Debug, Serialize)]
pub struct ToolResultMessage {
    pub role: String,
    pub tool_call_id: String,
    pub content: String,
}

impl ToolResultMessage {
    pub fn new(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            tool_call_id,
            content,
        }
    }
}

/// Format tool results as a JSON message for the API
pub fn format_tool_message(tool_call_id: &str, content: &str) -> serde_json::Value {
    serde_json::json!({
        "role": "tool",
        "tool_call_id": tool_call_id,
        "content": content
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_call_example() {
        let json = serde_json::json!({
            "id": "call_123",
            "type": "function",
            "function": {
                "name": "web_search",
                "arguments": "{\"query\": \"Apple stock price\"}"
            }
        });

        let tool_call = parse_tool_call(&json);
        assert!(tool_call.is_some());
        let tc = tool_call.unwrap();
        assert_eq!(tc.function.name, "web_search");
    }

    #[test]
    fn test_has_tool_calls() {
        let with_tools = serde_json::json!({
            "choices": [{
                "message": {
                    "tool_calls": [{"id": "call_1"}]
                }
            }]
        });

        let without_tools = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Hello"
                }
            }]
        });

        assert!(has_tool_calls(&with_tools));
        assert!(!has_tool_calls(&without_tools));
    }
}