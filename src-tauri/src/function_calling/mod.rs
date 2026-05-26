//! Function calling module for Gemma 4 tool integration
//!
//! Provides web search fallback capabilities when the model detects
//! low confidence or real-time data queries.

pub mod schema;
pub mod web_search;
pub mod handler;

pub use schema::{get_web_search_tool, get_low_confidence_prompt};
pub use handler::{
    has_tool_calls, execute_tool_call, extract_tool_calls, extract_content,
    MAX_TOOL_CALL_ITERATIONS,
};

/// Tool definitions array for the API request
pub fn get_all_tools() -> Vec<serde_json::Value> {
    vec![serde_json::to_value(get_web_search_tool()).unwrap()]
}

/// System message with tool instructions
pub fn get_system_message_with_tools() -> String {
    format!(
        "You are a helpful AI assistant. {}",
        get_low_confidence_prompt()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_tools() {
        let tools = get_all_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["function"]["name"], "web_search");
    }

    #[test]
    fn test_system_message() {
        let msg = get_system_message_with_tools();
        assert!(msg.contains("web_search"));
        assert!(msg.contains("current events"));
    }
}