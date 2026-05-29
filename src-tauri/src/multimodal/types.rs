//! Tipos para el módulo multimodal
//!
//! Define los tipos para contenido multimodal y comunicación con llama-server.

use serde::{Deserialize, Serialize};

// ============================================================================
// Tipos para contenido multimodal (frontend -> backend)
// ============================================================================

/// Wrapper for the nested image_url object from the frontend
#[derive(Clone, Debug, Deserialize)]
pub struct ImageUrlData {
    pub url: String,
}

/// Wrapper for the nested audio_url object from the frontend
#[derive(Clone, Debug, Deserialize)]
pub struct AudioUrlData {
    pub url: String,
}

/// Parte de contenido de un mensaje multimodal
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlData },
    #[serde(rename = "audio_url")]
    AudioUrl { audio_url: AudioUrlData },
}

/// Mensaje multimodal del frontend
#[derive(Clone, Debug, Deserialize)]
pub struct MultimodalMessage {
    pub role: String,
    pub content: Vec<ContentPart>,
}

/// Mensaje de tool (respuesta de función) para reinyección en contexto
#[derive(Clone, Debug, Serialize)]
pub struct ToolMessage {
    pub role: String,
    pub tool_call_id: String,
    pub content: String,
}

// ============================================================================
// Tipos para formato OpenAI (backend -> llama-server)
// ============================================================================

#[derive(Clone, Debug, Serialize)]
pub struct OpenAIMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<OpenAIContentPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrlContent },
    AudioUrl { audio_url: AudioUrlContent },
}

#[derive(Clone, Debug, Serialize)]
pub struct ImageUrlContent {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AudioUrlContent {
    pub url: String,
}

/// Tool call definition from the model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

// ============================================================================
// Implementaciones de conversión
// ============================================================================

impl ContentPart {
    /// Convierte una ContentPart del frontend al formato OpenAI
    pub fn to_openai(&self) -> OpenAIContentPart {
        match self {
            ContentPart::Text { text } => OpenAIContentPart::Text {
                text: text.clone(),
            },
            ContentPart::ImageUrl { image_url } => OpenAIContentPart::ImageUrl {
                image_url: ImageUrlContent {
                    url: image_url.url.clone(),
                    detail: Some("auto".to_string()),
                },
            },
            ContentPart::AudioUrl { audio_url } => OpenAIContentPart::AudioUrl {
                audio_url: AudioUrlContent {
                    url: audio_url.url.clone(),
                },
            },
        }
    }
}

impl MultimodalMessage {
    /// Convierte un mensaje multimodal al formato OpenAI
    pub fn to_openai(&self) -> OpenAIMessage {
        OpenAIMessage {
            role: self.role.clone(),
            content: Some(self.content.iter().map(|p| p.to_openai()).collect()),
            tool_calls: None,
            tool_call_id: None,
        }
    }
}