use serde::{Deserialize, Serialize};

/// Estrategia para estimar el número de tokens en un fragmento de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenEstimator {
    /// Cada palabra delimitada por espacios es un token.
    Words,
    /// Estimación basada en caracteres (ej. 1 token ≈ N caracteres).
    Characters { chars_per_token: usize },
    /// Estimación heurística común para LLMs (ej. 1 palabra ≈ 1.3 tokens).
    Heuristic,
}

/// Configuración para la segmentación del texto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    /// Tamaño máximo del fragmento en tokens estimados.
    pub chunk_size: usize,
    /// Porcentaje de solapamiento semántico (ej. 0.10 para 10%).
    pub overlap_percentage: f64,
    /// Método utilizado para estimar el conteo de tokens.
    pub estimator: TokenEstimator,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            overlap_percentage: 0.10,
            estimator: TokenEstimator::Heuristic,
        }
    }
}

/// Un fragmento de texto procesado y segmentado.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chunk {
    /// Contenido del fragmento.
    pub text: String,
    /// Índice del carácter inicial en el texto original (inclusive, basado en 0).
    pub start_char: usize,
    /// Índice del carácter final en el texto original (exclusive, basado en 0).
    pub end_char: usize,
    /// Cantidad estimada de tokens en este fragmento.
    pub token_count: usize,
}

#[derive(Debug, Clone)]
enum TextSpanKind {
    Word,
    Whitespace,
}

#[derive(Debug, Clone)]
struct TextSpan {
    kind: TextSpanKind,
    start_byte: usize,
    end_byte: usize,
    start_char: usize,
    end_char: usize,
}

/// Segmenta un texto de entrada en piezas manejables (chunks) según la configuración especificada.
///
/// Este algoritmo garantiza que:
/// 1. No se pierda ninguna palabra o carácter original (salvo espacios iniciales/finales del documento si se desea,
///    pero aquí se mantiene la correspondencia exacta).
/// 2. No se corten palabras por la mitad (la división se hace a nivel de límites de palabra).
/// 3. Se mantenga un solapamiento dinámico y coherente entre bloques adyacentes para preservar contexto.
pub fn chunk_text(text: &str, config: &ChunkConfig) -> Vec<Chunk> {
    if text.is_empty() {
        return Vec::new();
    }

    let spans = tokenize(text);
    if spans.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let n = spans.len();
    let mut i = 0;

    while i < n {
        let mut current_tokens = 0.0;
        let mut j = i;
        let mut word_count = 0;

        while j < n {
            let span_tokens = estimate_span_tokens(&spans[j], config.estimator);

            // Si ya tenemos al menos una palabra y añadir esta span excede el tamaño del chunk, detenemos el chunk actual.
            if word_count > 0 && current_tokens + span_tokens > config.chunk_size as f64 {
                break;
            }

            if let TextSpanKind::Word = spans[j].kind {
                word_count += 1;
            }
            current_tokens += span_tokens;
            j += 1;
        }

        if j > i {
            // Recortar espacios en blanco al principio y al final del chunk
            let mut start_idx = i;
            let mut end_idx = j;
            while start_idx < end_idx && matches!(spans[start_idx].kind, TextSpanKind::Whitespace) {
                start_idx += 1;
            }
            while end_idx > start_idx && matches!(spans[end_idx - 1].kind, TextSpanKind::Whitespace) {
                end_idx -= 1;
            }

            if start_idx < end_idx {
                let start_byte = spans[start_idx].start_byte;
                let end_byte = spans[end_idx - 1].end_byte;
                let start_char = spans[start_idx].start_char;
                let end_char = spans[end_idx - 1].end_char;
                let chunk_text = text[start_byte..end_byte].to_string();

                chunks.push(Chunk {
                    text: chunk_text,
                    start_char,
                    end_char,
                    token_count: current_tokens.round() as usize,
                });
            }

            // Si llegamos al final de la lista de spans, terminamos.
            if j >= n {
                break;
            }

            // Identificar los índices de los spans que son palabras en el chunk actual.
            let word_indices: Vec<usize> = (i..j)
                .filter(|&idx| matches!(spans[idx].kind, TextSpanKind::Word))
                .collect();

            if word_indices.is_empty() {
                // Si no hay palabras, avanzamos de manera limpia.
                i = j;
                continue;
            }

            // Calcular el número de palabras que formarán el solapamiento.
            let num_words = word_indices.len();
            let overlap_count = (num_words as f64 * config.overlap_percentage).round() as usize;

            // Asegurar que el solapamiento no impida el avance (debe ser estrictamente menor que el número de palabras).
            let overlap_count = overlap_count.min(num_words - 1);

            if overlap_count > 0 {
                // El siguiente chunk comenzará en el span de palabra correspondiente al retroceso.
                let overlap_word_idx = word_indices[num_words - overlap_count];
                i = overlap_word_idx;
            } else {
                // Sin solapamiento, comenzamos exactamente donde terminó el chunk actual.
                i = j;
            }
        } else {
            // Salvaguarda contra bucles infinitos en configuraciones extremas.
            break;
        }
    }

    chunks
}

fn tokenize(text: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    let mut char_idx = 0;
    let mut current_word_start_byte = None;
    let mut current_word_start_char = None;
    let mut current_space_start_byte = None;
    let mut current_space_start_char = None;
    let mut last_byte_idx = 0;

    for (b_idx, c) in text.char_indices() {
        let is_ws = c.is_whitespace();
        if is_ws {
            if let (Some(w_start_b), Some(w_start_c)) = (current_word_start_byte, current_word_start_char) {
                spans.push(TextSpan {
                    kind: TextSpanKind::Word,
                    start_byte: w_start_b,
                    end_byte: b_idx,
                    start_char: w_start_c,
                    end_char: char_idx,
                });
                current_word_start_byte = None;
                current_word_start_char = None;
            }
            if current_space_start_byte.is_none() {
                current_space_start_byte = Some(b_idx);
                current_space_start_char = Some(char_idx);
            }
        } else {
            if let (Some(s_start_b), Some(s_start_c)) = (current_space_start_byte, current_space_start_char) {
                spans.push(TextSpan {
                    kind: TextSpanKind::Whitespace,
                    start_byte: s_start_b,
                    end_byte: b_idx,
                    start_char: s_start_c,
                    end_char: char_idx,
                });
                current_space_start_byte = None;
                current_space_start_char = None;
            }
            if current_word_start_byte.is_none() {
                current_word_start_byte = Some(b_idx);
                current_word_start_char = Some(char_idx);
            }
        }
        char_idx += 1;
        last_byte_idx = b_idx + c.len_utf8();
    }

    // Procesar el residuo final
    if let (Some(w_start_b), Some(w_start_c)) = (current_word_start_byte, current_word_start_char) {
        spans.push(TextSpan {
            kind: TextSpanKind::Word,
            start_byte: w_start_b,
            end_byte: last_byte_idx,
            start_char: w_start_c,
            end_char: char_idx,
        });
    } else if let (Some(s_start_b), Some(s_start_c)) = (current_space_start_byte, current_space_start_char) {
        spans.push(TextSpan {
            kind: TextSpanKind::Whitespace,
            start_byte: s_start_b,
            end_byte: last_byte_idx,
            start_char: s_start_c,
            end_char: char_idx,
        });
    }

    spans
}

fn estimate_span_tokens(span: &TextSpan, estimator: TokenEstimator) -> f64 {
    match span.kind {
        TextSpanKind::Whitespace => 0.0,
        TextSpanKind::Word => {
            let char_count = span.end_char - span.start_char;
            match estimator {
                TokenEstimator::Words => 1.0,
                TokenEstimator::Characters { chars_per_token } => {
                    char_count as f64 / chars_per_token.max(1) as f64
                }
                TokenEstimator::Heuristic => {
                    // LLM heuristic rule of thumb: 4 characters per token
                    (char_count as f64 / 4.0).max(1.0)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input_returns_empty_vector() {
        let config = ChunkConfig::default();
        let chunks = chunk_text("", &config);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_short_input_returns_single_chunk() {
        let text = "Hola mundo.";
        let config = ChunkConfig {
            chunk_size: 10,
            overlap_percentage: 0.10,
            estimator: TokenEstimator::Words,
        };
        let chunks = chunk_text(text, &config);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "Hola mundo.");
        assert_eq!(chunks[0].start_char, 0);
        assert_eq!(chunks[0].end_char, text.chars().count());
    }

    #[test]
    fn test_no_words_lost_and_correct_boundaries() {
        let text = "Uno dos tres cuatro cinco seis siete ocho nueve diez";
        let config = ChunkConfig {
            chunk_size: 3,
            overlap_percentage: 0.0, // Sin solapamiento para verificar unión limpia
            estimator: TokenEstimator::Words,
        };
        let chunks = chunk_text(text, &config);
        
        // Cada chunk tiene un máximo de 3 palabras.
        // "Uno dos tres", "cuatro cinco seis", "siete ocho nueve", "diez"
        assert_eq!(chunks.len(), 4);
        
        // Validar que reconstruyendo el texto con los índices exactos del original,
        // no se pierde absolutamente ninguna palabra ni carácter en las uniones.
        let mut reconstructed = String::new();
        for (i, chunk) in chunks.iter().enumerate() {
            // Verificar que no se mutilan palabras
            assert!(!chunk.text.is_empty());
            
            // Comprobar correspondencia de caracteres
            let slice_from_original = &text[chunk.text.as_bytes().len() * 0..]; // simple slice placeholder check
            let char_slice: String = text.chars().skip(chunk.start_char).take(chunk.end_char - chunk.start_char).collect();
            assert_eq!(chunk.text, char_slice);
            
            if i > 0 {
                // Verificar que el inicio de este chunk coincide con la continuación del anterior (más el espacio)
                assert_eq!(chunk.start_char, chunks[i-1].end_char + 1);
            }
            reconstructed.push_str(&chunk.text);
            if i < chunks.len() - 1 {
                reconstructed.push(' ');
            }
        }
        
        assert_eq!(reconstructed, text);
    }

    #[test]
    fn test_dynamic_overlap_contains_expected_semantic_context() {
        let text = "Uno dos tres cuatro cinco";
        let config = ChunkConfig {
            chunk_size: 3,
            overlap_percentage: 0.33, // ~33% de solapamiento (1 palabra de solapamiento en chunk de 3 palabras)
            estimator: TokenEstimator::Words,
        };
        let chunks = chunk_text(text, &config);
        
        // Chunk 1: "Uno dos tres" (3 palabras)
        // Solapamiento: 3 palabras * 0.33 = 0.99 -> redondeado a 1 palabra de solapamiento ("tres")
        // Chunk 2: "tres cuatro cinco"
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].text, "Uno dos tres");
        assert_eq!(chunks[1].text, "tres cuatro cinco");
        
        // Verificar que la palabra "tres" se conserva en ambos chunks perfectamente
        assert!(chunks[0].text.ends_with("tres"));
        assert!(chunks[1].text.starts_with("tres"));
    }

    #[test]
    fn test_unicode_and_special_characters_safety() {
        let text = "Café ☕ de España 🇪🇸 y Japón 🇯🇵 es súper delicioso.";
        let config = ChunkConfig {
            chunk_size: 4,
            overlap_percentage: 0.25,
            estimator: TokenEstimator::Words,
        };
        
        // Debe procesarse sin pánicos de alineamiento UTF-8
        let chunks = chunk_text(text, &config);
        assert!(!chunks.is_empty());
        
        for chunk in chunks {
            // Slicing de caracteres original debe coincidir exactamente
            let original_slice: String = text.chars().skip(chunk.start_char).take(chunk.end_char - chunk.start_char).collect();
            assert_eq!(chunk.text, original_slice);
        }
    }
}
