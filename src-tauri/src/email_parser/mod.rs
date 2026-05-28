// src-tauri/src/email_parser/mod.rs

use regex::Regex;

lazy_static::lazy_static! {
    static ref HEADER_REGEX: Regex = Regex::new(
        r"(?i)^(from|to|sent|date|cc|subject|de|para|enviado|fecha|asunto|responder a|reply-to|enviado el)\s*:.*"
    ).expect("Failed to compile HEADER_REGEX");

    static ref SEPARATOR_REGEX: Regex = Regex::new(
        r"(?i)^(-{3,}|_{3,}|={3,}|\*{3,}|-{2,}\s*(original message|mensaje original)\s*-{2,})$"
    ).expect("Failed to compile SEPARATOR_REGEX");

    static ref PHONE_REGEX: Regex = Regex::new(
        r"(?i)(tel|cel|ph|phone|mob|movil|telf|teléfono|telefono|fax|t\s*\.\s*\+?|m\s*\.\s*\+?)\s*:\s*\+?[\d\s\-\(\)\.]+"
    ).expect("Failed to compile PHONE_REGEX");

    static ref EMAIL_LINK_REGEX: Regex = Regex::new(
        r"(?i)(www\.|https?://|[a-zA-Z0-9\._%+-]+@[a-zA-Z0-9\.-]+\.[a-zA-Z]{2,})"
    ).expect("Failed to compile EMAIL_LINK_REGEX");

    static ref CLOSING_REGEX: Regex = Regex::new(
        r"(?i)^(saludos|saludos cordiales|atentamente|cordialmente|un saludo|abrazo|un cordial saludo|best regards|regards|sincerely|thanks|thank you|gracias|att|un abrazo|afectuosamente)\s*,?\s*$"
    ).expect("Failed to compile CLOSING_REGEX");

    static ref DISCLAIMER_KEYWORDS: Regex = Regex::new(
        r"(?i)(este mensaje es confidencial|la información contenida|este correo electrónico|this message is confidential|disclaimer:|antes de imprimir|please consider the environment|think before you print|aviso de confidencialidad|confidentiality notice|legal notice|aviso legal|este mensaje y sus anexos|privacidad y confidencialidad|nota de confidencialidad)"
    ).expect("Failed to compile DISCLAIMER_KEYWORDS");
}

/// Clean a raw email thread to optimize token consumption for LLMs
pub fn clean_email_thread_logic(text: &str) -> String {
    let raw_lines: Vec<&str> = text.lines().map(|line| line.trim()).collect();
    let mut cleaned_lines: Vec<String> = Vec::new();
    let mut index = 0;
    let total_lines = raw_lines.len();

    while index < total_lines {
        let current_line = raw_lines[index];

        // 1. Skip empty lines for now (we will normalize spacing at the end)
        if current_line.is_empty() {
            cleaned_lines.push(String::new());
            index += 1;
            continue;
        }

        // 2. Remove email headers
        if HEADER_REGEX.is_match(current_line) {
            index += 1;
            continue;
        }

        // 3. Remove structural thread separators (horizontal lines, "-----Original Message-----")
        if SEPARATOR_REGEX.is_match(current_line) {
            index += 1;
            continue;
        }

        // 4. Remove corporate watermarks / disclaimers
        if DISCLAIMER_KEYWORDS.is_match(current_line) {
            index += 1;
            continue;
        }

        // 5. Advanced signature detection with look-ahead
        if CLOSING_REGEX.is_match(current_line) {
            // Check next lines to see if it's a signature block
            let mut is_signature = false;
            let mut lines_to_skip = 1;

            // Look ahead up to 5 non-empty lines
            let mut non_empty_lookahead_count = 0;
            let mut current_lookahead_offset = 1;

            while index + current_lookahead_offset < total_lines && non_empty_lookahead_count < 5 {
                let lookahead_line = raw_lines[index + current_lookahead_offset];
                if !lookahead_line.is_empty() {
                    non_empty_lookahead_count += 1;

                    // If we find contact info, disclaimer, or header, it's definitely a signature block
                    if PHONE_REGEX.is_match(lookahead_line)
                        || EMAIL_LINK_REGEX.is_match(lookahead_line)
                        || DISCLAIMER_KEYWORDS.is_match(lookahead_line)
                        || HEADER_REGEX.is_match(lookahead_line)
                        || SEPARATOR_REGEX.is_match(lookahead_line)
                        || CLOSING_REGEX.is_match(lookahead_line)
                    {
                        is_signature = true;
                    }
                }
                lines_to_skip += 1;
                current_lookahead_offset += 1;
            }

            // Even if no contact info was found, if the thread ends shortly or next lines are very short,
            // classify it as signature
            if !is_signature && index + 1 == total_lines {
                is_signature = true;
            }

            if is_signature {
                // Skip the closing and the lookahead signature details
                index += lines_to_skip;
                continue;
            }
        }

        // 6. If it's a line with only contact info, skip it as it's likely a dangling signature part
        if PHONE_REGEX.is_match(current_line) || (EMAIL_LINK_REGEX.is_match(current_line) && current_line.len() < 100) {
            index += 1;
            continue;
        }

        // 7. Retain the line as pure message body
        cleaned_lines.push(current_line.to_string());
        index += 1;
    }

    // Post-processing: Normalize spacing and line breaks
    let mut final_lines: Vec<String> = Vec::new();
    let mut was_empty = true; // Avoid leading empty lines

    for line in cleaned_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !was_empty {
                final_lines.push(String::new());
                was_empty = true;
            }
        } else {
            final_lines.push(trimmed.to_string());
            was_empty = false;
        }
    }

    // Remove trailing empty line if exists
    if let Some(last) = final_lines.last() {
        if last.is_empty() {
            final_lines.pop();
        }
    }

    final_lines.join("\n")
}

/// Tauri command wrapper to clean email threads
#[tauri::command]
pub fn clean_email_thread(text: String) -> Result<String, String> {
    Ok(clean_email_thread_logic(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_headers() {
        let input = "De: Juan Pérez <juan@example.com>\nPara: maria@example.com\nAsunto: Reunión importante\nEnviado: Jueves, 28 de Mayo\n\nHola María,\n¿Cómo estás?";
        let output = clean_email_thread_logic(input);
        assert_eq!(output, "Hola María,\n¿Cómo estás?");
    }

    #[test]
    fn test_remove_signatures() {
        let input = "Hola,\nTe envío los documentos.\n\nSaludos cordiales,\nJuan Pérez\nTel: +34 600 000 000\nwww.ejemplo.com";
        let output = clean_email_thread_logic(input);
        assert_eq!(output, "Hola,\nTe envío los documentos.");
    }

    #[test]
    fn test_remove_disclaimers() {
        let input = "Hola,\nEste es el contenido principal.\n\nEste mensaje es confidencial y contiene información privilegiada.";
        let output = clean_email_thread_logic(input);
        assert_eq!(output, "Hola,\nEste es el contenido principal.");
    }

    #[test]
    fn test_clean_complex_thread() {
        let mut thread = String::new();
        for i in 1..=10 {
            thread.push_str(&format!(
                "De: Usuario {}\nPara: Admin\nAsunto: Respuesta {}\nFecha: 28-05-2026\n\nEste es el mensaje de la respuesta {}.\n\nSaludos,\nUsuario {}\nTel: 123456\n\nEste mensaje y sus anexos son confidenciales.\n\n",
                i, i, i, i
            ));
            thread.push_str("________________________________\n\n");
        }

        let output = clean_email_thread_logic(&thread);
        
        // Assert we successfully extracted all 10 responses and removed headers/footers
        for i in 1..=10 {
            assert!(output.contains(&format!("Este es el mensaje de la respuesta {}.", i)));
            assert!(!output.contains(&format!("De: Usuario {}", i)));
            assert!(!output.contains(&format!("Saludos,\nUsuario {}", i)));
        }
        assert!(!output.contains("Este mensaje y sus anexos son confidenciales"));
    }
}
