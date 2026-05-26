//! Web search implementation using lightweight scraping
//!
//! Provides real-time web search capabilities by querying DuckDuckGo's
//! HTML results and extracting relevant snippets.

use regex::Regex;
use scraper::{Html, Selector};

/// Maximum number of snippets to return
const MAX_SNIPPETS: usize = 5;

/// Maximum characters per snippet
const MAX_SNIPPET_CHARS: usize = 200;

/// A single search result snippet
#[derive(Clone, Debug, serde::Serialize)]
pub struct SearchSnippet {
    pub title: String,
    pub url: String,
    pub content: String,
}

/// Complete web search result
#[derive(Clone, Debug, serde::Serialize)]
pub struct WebSearchResult {
    pub query: String,
    pub snippets: Vec<SearchSnippet>,
    pub timestamp: String,
}

/// Error type for web search failures
#[derive(Debug)]
pub enum WebSearchError {
    Network(String),
    Parse(String),
    NoResults,
}

/// Performs a web search and returns formatted snippets
pub async fn web_search(query: &str) -> Result<WebSearchResult, WebSearchError> {
    // URL encode the query
    let encoded_query = urlencoding::encode(query);
    let url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);

    // Make HTTP request
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| WebSearchError::Network(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| WebSearchError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(WebSearchError::Network(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| WebSearchError::Network(e.to_string()))?;

    // Parse HTML and extract snippets
    let snippets = parse_search_results(&html)?;

    if snippets.is_empty() {
        return Err(WebSearchError::NoResults);
    }

    Ok(WebSearchResult {
        query: query.to_string(),
        snippets,
        timestamp: chrono_lite_timestamp(),
    })
}

/// Parse DuckDuckGo HTML results into snippets
fn parse_search_results(html: &str) -> Result<Vec<SearchSnippet>, WebSearchError> {
    let document = Html::parse_document(html);
    let mut snippets = Vec::new();

    // Selector for result links (a.result__a)
    if let Ok(link_selector) = Selector::parse("a.result__a") {
        // Selector for result snippets (div.result__snippet)
        if let Ok(snippet_selector) = Selector::parse("div.result__snippet") {
            // Get all result links
            let links: Vec<_> = document.select(&link_selector).collect();
            let snippet_elements: Vec<_> = document.select(&snippet_selector).collect();

            for i in 0..links.len().min(MAX_SNIPPETS) {
                let title = links[i]
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string();

                let url = links[i]
                    .value()
                    .attr("href")
                    .unwrap_or("")
                    .to_string();

                // Clean URL (DuckDuckGo adds intermediate redirects)
                let clean_url = clean_duckduckgo_url(&url);

                // Get corresponding snippet if available
                let content = snippet_elements
                    .get(i)
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let content = truncate_snippet(&content);

                if !title.is_empty() && !clean_url.is_empty() {
                    snippets.push(SearchSnippet {
                        title,
                        url: clean_url,
                        content,
                    });
                }
            }
        }
    }

    // Fallback: If no structured results, try regex extraction
    if snippets.is_empty() {
        snippets = extract_with_regex(html)?;
    }

    Ok(snippets)
}

/// Clean DuckDuckGo redirect URLs
fn clean_duckduckgo_url(url: &str) -> String {
    // DuckDuckGo uses uddg= parameter for redirects
    if let Some(idx) = url.find("uddg=") {
        let after_uddg = &url[idx + 5..];
        // URL decode
        urlencoding::decode(after_uddg)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| after_uddg.to_string())
    } else if url.starts_with("http") {
        url.to_string()
    } else {
        String::new()
    }
}

/// Truncate snippet to MAX_SNIPPET_CHARS
fn truncate_snippet(text: &str) -> String {
    let text = text.trim();
    if text.len() <= MAX_SNIPPET_CHARS {
        text.to_string()
    } else {
        // Try to truncate at word boundary
        let truncated = &text[..MAX_SNIPPET_CHARS];
        if let Some(last_space) = truncated.rfind(' ') {
            format!("{}...", &truncated[..last_space])
        } else {
            format!("{}...", truncated)
        }
    }
}

/// Simple regex fallback when HTML parsing fails
fn extract_with_regex(html: &str) -> Result<Vec<SearchSnippet>, WebSearchError> {
    let mut snippets = Vec::new();

    // Pattern to match result entries
    let result_pattern = Regex::new(r#"<a class="result__a" href="([^"]+)">([^<]+)</a>"#)
        .map_err(|e| WebSearchError::Parse(e.to_string()))?;

    let snippet_pattern = Regex::new(r#"<div class="result__snippet">([^<]+)<"#)
        .map_err(|e| WebSearchError::Parse(e.to_string()))?;

    let urls: Vec<(String, String)> = result_pattern
        .captures_iter(html)
        .take(MAX_SNIPPETS)
        .map(|cap| {
            let url = clean_duckduckgo_url(&cap[1]);
            let title = cap[2].trim().to_string();
            (url, title)
        })
        .collect();

    let snippet_caps: Vec<String> = snippet_pattern
        .captures_iter(html)
        .take(MAX_SNIPPETS)
        .map(|cap| cap[1].trim().to_string())
        .collect();

    for i in 0..urls.len() {
        let (url, title) = &urls[i];
        let content = snippet_caps
            .get(i)
            .map(|s| truncate_snippet(s))
            .unwrap_or_default();

        if !url.is_empty() && !title.is_empty() {
            snippets.push(SearchSnippet {
                title: title.clone(),
                url: url.clone(),
                content,
            });
        }
    }

    Ok(snippets)
}

/// Get a simple timestamp (avoiding chrono dependency)
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // Simple RFC 2822 style date
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;

    format!("Unix timestamp: {} ({} hours, {} minutes ago)", secs, hours, minutes)
}

/// Format snippets as a string for injection into model context
pub fn format_snippets_for_context(result: &WebSearchResult) -> String {
    let mut output = String::new();
    output.push_str("[WEB SEARCH RESULTS]\n");
    output.push_str(&format!("Query: {}\n", result.query));
    output.push_str(&format!("Retrieved: {}\n\n", result.timestamp));

    for (i, snippet) in result.snippets.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, snippet.title));
        if !snippet.content.is_empty() {
            output.push_str(&format!("   {}\n", snippet.content));
        }
        output.push_str(&format!("   Source: {}\n\n", snippet.url));
    }

    output.push_str("[/WEB SEARCH RESULTS]\n");
    output
}

/// Format a single tool call result as a tool message for the model
pub fn format_tool_result(tool_call_id: &str, result: &WebSearchResult) -> serde_json::Value {
    serde_json::json!({
        "role": "tool",
        "tool_call_id": tool_call_id,
        "content": format_snippets_for_context(result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_snippet() {
        let long_text = "This is a very long snippet that should be truncated because it exceeds the maximum character limit.";
        let truncated = truncate_snippet(long_text);
        assert!(truncated.len() <= MAX_SNIPPET_CHARS + 3); // +3 for "..."
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_clean_duckduckgo_url() {
        let url = "https://html.duckduckgo.com/html/?q=test&uddg=https%3A%2F%2Fexample.com%2Fpage";
        let cleaned = clean_duckduckgo_url(url);
        assert_eq!(cleaned, "https://example.com/page");
    }
}