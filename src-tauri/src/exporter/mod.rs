use directories::UserDirs;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const PAGE_WIDTH: f32 = 612.0;
const PAGE_HEIGHT: f32 = 792.0;
const MARGIN_LEFT: f32 = 72.0;
const MARGIN_TOP: f32 = 72.0;
const MARGIN_BOTTOM: f32 = 72.0;
const BODY_FONT_SIZE: f32 = 11.0;
const BODY_LINE_HEIGHT: f32 = 16.0;

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub html: String,
    pub format: ExportFormat,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Docx,
}

#[derive(Debug, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub format: String,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("El documento esta vacio.")]
    EmptyDocument,
    #[error("La exportacion DOCX no esta implementada intencionalmente en esta version.")]
    UnsupportedDocx,
    #[error("No se pudo resolver el escritorio del usuario.")]
    DesktopNotFound,
    #[error("No se pudo escribir el archivo exportado: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq)]
enum DocumentBlock {
    Heading { level: u8, text: String },
    Paragraph(String),
    Bullet(String),
    Ordered { index: usize, text: String },
    Quote(String),
    Code(String),
    Image(String),
}

pub async fn export_document(request: ExportRequest) -> Result<ExportResult, ExportError> {
    if request.html.trim().is_empty() {
        return Err(ExportError::EmptyDocument);
    }

    match request.format {
        ExportFormat::Pdf => {
            let blocks = html_to_blocks(&request.html);
            if blocks.is_empty() {
                return Err(ExportError::EmptyDocument);
            }

            let filename = request
                .filename
                .as_deref()
                .map(sanitize_filename)
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "documento-zeldrix".to_string());
            let path = unique_desktop_path(&filename, "pdf")?;
            let bytes = render_pdf(&blocks);
            fs::write(&path, bytes)?;

            Ok(ExportResult {
                path: path.to_string_lossy().to_string(),
                format: "pdf".to_string(),
            })
        }
        ExportFormat::Docx => Err(ExportError::UnsupportedDocx),
    }
}

fn html_to_blocks(html: &str) -> Vec<DocumentBlock> {
    let fragment = Html::parse_fragment(html);
    let selector = Selector::parse("h1,h2,h3,p,li,blockquote,pre,img").expect("valid selector");
    let mut blocks = Vec::new();
    let mut ordered_index = 1;

    for element in fragment.select(&selector) {
        let tag = element.value().name();
        if tag != "li" {
            ordered_index = 1;
        }

        match tag {
            "h1" | "h2" | "h3" => {
                let text = normalized_text(element.text());
                if !text.is_empty() {
                    let level = tag.trim_start_matches('h').parse::<u8>().unwrap_or(1);
                    blocks.push(DocumentBlock::Heading { level, text });
                }
            }
            "p" => {
                let text = normalized_text(element.text());
                if !text.is_empty() {
                    blocks.push(DocumentBlock::Paragraph(text));
                }
            }
            "li" => {
                let text = normalized_text(element.text());
                if !text.is_empty() {
                    let is_ordered = element
                        .ancestors()
                        .filter_map(scraper::ElementRef::wrap)
                        .any(|ancestor| ancestor.value().name() == "ol");

                    if is_ordered {
                        blocks.push(DocumentBlock::Ordered {
                            index: ordered_index,
                            text,
                        });
                        ordered_index += 1;
                    } else {
                        blocks.push(DocumentBlock::Bullet(text));
                    }
                }
            }
            "blockquote" => {
                let text = normalized_text(element.text());
                if !text.is_empty() {
                    blocks.push(DocumentBlock::Quote(text));
                }
            }
            "pre" => {
                let text = element.text().collect::<Vec<_>>().join("\n");
                let text = text.trim().to_string();
                if !text.is_empty() {
                    blocks.push(DocumentBlock::Code(text));
                }
            }
            "img" => {
                let alt = element.value().attr("alt").unwrap_or("Imagen corporativa");
                blocks.push(DocumentBlock::Image(alt.to_string()));
            }
            _ => {}
        }
    }

    blocks
}

fn normalized_text<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else if ch.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches(['-', '_'])
        .to_lowercase()
}

fn unique_desktop_path(filename: &str, extension: &str) -> Result<PathBuf, ExportError> {
    let desktop = UserDirs::new()
        .and_then(|dirs| dirs.desktop_dir().map(Path::to_path_buf))
        .ok_or(ExportError::DesktopNotFound)?;

    let mut candidate = desktop.join(format!("{filename}.{extension}"));
    let mut counter = 2;

    while candidate.exists() {
        candidate = desktop.join(format!("{filename}-{counter}.{extension}"));
        counter += 1;
    }

    Ok(candidate)
}

fn render_pdf(blocks: &[DocumentBlock]) -> Vec<u8> {
    let pages = layout_pages(blocks);
    write_pdf(&pages)
}

#[derive(Debug, Clone)]
struct PdfLine {
    text: String,
    x: f32,
    y: f32,
    font_size: f32,
}

#[derive(Debug, Clone, Copy)]
struct BlockStyle {
    font_size: f32,
    line_height: f32,
    before: f32,
    after: f32,
}

fn layout_pages(blocks: &[DocumentBlock]) -> Vec<Vec<PdfLine>> {
    let mut pages = vec![Vec::new()];
    let mut y = PAGE_HEIGHT - MARGIN_TOP;

    for block in blocks {
        let (prefix, text, style) = match block {
            DocumentBlock::Heading { level, text } => {
                let size = match level {
                    1 => 22.0,
                    2 => 17.0,
                    _ => 14.0,
                };
                (
                    "",
                    text.as_str(),
                    BlockStyle {
                        font_size: size,
                        line_height: size + 7.0,
                        before: 10.0,
                        after: 8.0,
                    },
                )
            }
            DocumentBlock::Paragraph(text) => ("", text.as_str(), body_style(4.0, 7.0)),
            DocumentBlock::Bullet(text) => ("- ", text.as_str(), body_style(3.0, 4.0)),
            DocumentBlock::Ordered { index, text } => {
                let prefix = format!("{index}. ");
                add_wrapped_block(&mut pages, &mut y, &prefix, text, body_style(3.0, 4.0));
                continue;
            }
            DocumentBlock::Quote(text) => ("> ", text.as_str(), body_style(5.0, 7.0)),
            DocumentBlock::Code(text) => (
                "",
                text.as_str(),
                BlockStyle {
                    font_size: 10.0,
                    line_height: 14.0,
                    before: 5.0,
                    after: 7.0,
                },
            ),
            DocumentBlock::Image(alt) => ("[Imagen] ", alt.as_str(), body_style(5.0, 7.0)),
        };

        add_wrapped_block(&mut pages, &mut y, prefix, text, style);
    }

    pages
}

fn body_style(before: f32, after: f32) -> BlockStyle {
    BlockStyle {
        font_size: BODY_FONT_SIZE,
        line_height: BODY_LINE_HEIGHT,
        before,
        after,
    }
}

fn add_wrapped_block(
    pages: &mut Vec<Vec<PdfLine>>,
    y: &mut f32,
    prefix: &str,
    text: &str,
    style: BlockStyle,
) {
    *y -= style.before;
    let max_width = PAGE_WIDTH - MARGIN_LEFT * 2.0;
    let indent = if prefix.is_empty() { 0.0 } else { 18.0 };
    let lines = wrap_text(text, style.font_size, max_width - indent);

    for (line_index, line) in lines.iter().enumerate() {
        if *y < MARGIN_BOTTOM {
            pages.push(Vec::new());
            *y = PAGE_HEIGHT - MARGIN_TOP;
        }

        let text = if line_index == 0 {
            format!("{prefix}{line}")
        } else {
            line.clone()
        };
        let x = MARGIN_LEFT + if line_index == 0 { 0.0 } else { indent };
        pages.last_mut().expect("at least one page").push(PdfLine {
            text,
            x,
            y: *y,
            font_size: style.font_size,
        });
        *y -= style.line_height;
    }

    *y -= style.after;
}

fn wrap_text(text: &str, font_size: f32, max_width: f32) -> Vec<String> {
    let chars_per_line = (max_width / (font_size * 0.52)).max(12.0) as usize;
    let mut lines = Vec::new();

    for raw_line in text.lines() {
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let extra = if current.is_empty() { 0 } else { 1 };
            if current.len() + word.len() + extra > chars_per_line && !current.is_empty() {
                lines.push(current);
                current = String::new();
            }

            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn write_pdf(pages: &[Vec<PdfLine>]) -> Vec<u8> {
    let page_count = pages.len();
    let font_id = 3 + page_count * 2;
    let mut objects = Vec::new();

    let kids = (0..page_count)
        .map(|index| format!("{} 0 R", 3 + index * 2))
        .collect::<Vec<_>>()
        .join(" ");

    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    objects.push(format!(
        "<< /Type /Pages /Kids [{kids}] /Count {page_count} >>"
    ));

    for (index, page_lines) in pages.iter().enumerate() {
        let page_id = 3 + index * 2;
        let content_id = page_id + 1;
        let stream = page_content_stream(page_lines);
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.0} {PAGE_HEIGHT:.0}] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            stream.len(),
            stream
        ));
    }

    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());

    let mut pdf = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());

    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }

    let xref_offset = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );

    pdf
}

fn page_content_stream(lines: &[PdfLine]) -> String {
    let mut stream = String::new();

    for line in lines {
        stream.push_str(&format!(
            "BT /F1 {:.1} Tf {:.1} {:.1} Td ({}) Tj ET\n",
            line.font_size,
            line.x,
            line.y,
            escape_pdf_text(&line.text)
        ));
    }

    stream
}

fn escape_pdf_text(text: &str) -> String {
    text.chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '(' => "\\(".chars().collect::<Vec<_>>(),
            ')' => "\\)".chars().collect::<Vec<_>>(),
            '\n' | '\r' => " ".chars().collect::<Vec<_>>(),
            ch if ch.is_ascii() => vec![ch],
            ch => ascii_fallback(ch).chars().collect::<Vec<_>>(),
        })
        .collect()
}

fn ascii_fallback(ch: char) -> String {
    match ch {
        'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => "a",
        'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => "e",
        'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => "i",
        'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => "o",
        'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => "u",
        'ñ' | 'Ñ' => "n",
        '¿' | '¡' => "",
        '“' | '”' => "\"",
        '‘' | '’' => "'",
        '–' | '—' => "-",
        _ => "?",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_should_remove_unsafe_characters() {
        assert_eq!(
            sanitize_filename("Contrato Cliente / Mayo 2026"),
            "contrato-cliente-_-mayo-2026"
        );
    }

    #[test]
    fn html_to_blocks_should_preserve_document_order() {
        let blocks = html_to_blocks(
            "<h1>Contrato</h1><p>Hola <strong>mundo</strong></p><ul><li>Uno</li></ul>",
        );

        assert_eq!(
            blocks,
            vec![
                DocumentBlock::Heading {
                    level: 1,
                    text: "Contrato".to_string(),
                },
                DocumentBlock::Paragraph("Hola mundo".to_string()),
                DocumentBlock::Bullet("Uno".to_string()),
            ]
        );
    }

    #[test]
    fn render_pdf_should_create_valid_pdf_header_and_trailer() {
        let bytes = render_pdf(&[DocumentBlock::Paragraph("Documento de prueba".to_string())]);
        let pdf = String::from_utf8_lossy(&bytes);

        assert!(pdf.starts_with("%PDF-1.4"));
        assert!(pdf.contains("%%EOF"));
    }

    #[test]
    fn render_pdf_should_paginate_long_documents() {
        let blocks = (0..120)
            .map(|index| DocumentBlock::Paragraph(format!("Parrafo corporativo numero {index}")))
            .collect::<Vec<_>>();

        let bytes = render_pdf(&blocks);
        let pdf = String::from_utf8_lossy(&bytes);

        assert!(pdf.contains("/Count 3") || pdf.contains("/Count 4") || pdf.contains("/Count 5"));
    }

    #[tokio::test]
    async fn export_document_should_reject_docx_with_clear_unsupported_error() {
        let result = export_document(ExportRequest {
            html: "<p>Documento corporativo</p>".to_string(),
            format: ExportFormat::Docx,
            filename: Some("documento-zeldrix".to_string()),
        })
        .await;

        assert!(matches!(result, Err(ExportError::UnsupportedDocx)));
    }
}
