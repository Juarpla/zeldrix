use directories::UserDirs;
use scraper::{ElementRef, Html, Selector};
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
const LINK_BLUE: PdfColor = PdfColor(37.0 / 255.0, 99.0 / 255.0, 235.0 / 255.0);
const TEXT_BLACK: PdfColor = PdfColor(26.0 / 255.0, 26.0 / 255.0, 26.0 / 255.0);
const TEXT_MUTED: PdfColor = PdfColor(107.0 / 255.0, 107.0 / 255.0, 107.0 / 255.0);

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
    Heading { level: u8, content: Vec<TextRun> },
    Paragraph(Vec<TextRun>),
    Bullet(Vec<TextRun>),
    Ordered { index: usize, content: Vec<TextRun> },
    Quote(Vec<TextRun>),
    Code(Vec<TextRun>),
    Image { alt: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextRun {
    text: String,
    style: RunStyle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RunStyle {
    bold: bool,
    italic: bool,
    code: bool,
    link: bool,
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
                let content = inline_runs(element, RunStyle::default().bold());
                if !content.is_empty() {
                    let level = tag.trim_start_matches('h').parse::<u8>().unwrap_or(1);
                    blocks.push(DocumentBlock::Heading { level, content });
                }
            }
            "p" => {
                let content = inline_runs(element, RunStyle::default());
                if !content.is_empty() {
                    blocks.push(DocumentBlock::Paragraph(content));
                }
            }
            "li" => {
                let content = inline_runs(element, RunStyle::default());
                if !content.is_empty() {
                    let is_ordered = element
                        .ancestors()
                        .filter_map(scraper::ElementRef::wrap)
                        .any(|ancestor| ancestor.value().name() == "ol");

                    if is_ordered {
                        blocks.push(DocumentBlock::Ordered {
                            index: ordered_index,
                            content,
                        });
                        ordered_index += 1;
                    } else {
                        blocks.push(DocumentBlock::Bullet(content));
                    }
                }
            }
            "blockquote" => {
                let content = inline_runs(element, RunStyle::default().italic());
                if !content.is_empty() {
                    blocks.push(DocumentBlock::Quote(content));
                }
            }
            "pre" => {
                let text = element.text().collect::<Vec<_>>().join("\n");
                let text = text.trim().to_string();
                if !text.is_empty() {
                    blocks.push(DocumentBlock::Code(vec![TextRun {
                        text,
                        style: RunStyle::default().code(),
                    }]));
                }
            }
            "img" => {
                let alt = element.value().attr("alt").unwrap_or("Imagen corporativa");
                blocks.push(DocumentBlock::Image {
                    alt: alt.to_string(),
                });
            }
            _ => {}
        }
    }

    blocks
}

impl RunStyle {
    fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    fn code(mut self) -> Self {
        self.code = true;
        self
    }

    fn link(mut self) -> Self {
        self.link = true;
        self
    }
}

fn inline_runs(element: ElementRef<'_>, base_style: RunStyle) -> Vec<TextRun> {
    let mut runs = Vec::new();
    collect_inline_runs(element, base_style, &mut runs);
    normalize_runs(runs)
}

fn collect_inline_runs(element: ElementRef<'_>, inherited: RunStyle, runs: &mut Vec<TextRun>) {
    for child in element.children() {
        if let Some(text) = child.value().as_text() {
            runs.push(TextRun {
                text: text.to_string(),
                style: inherited,
            });
            continue;
        }

        if let Some(child_element) = ElementRef::wrap(child) {
            let tag = child_element.value().name();
            let style = match tag {
                "strong" | "b" => inherited.bold(),
                "em" | "i" => inherited.italic(),
                "code" => inherited.code(),
                "a" => inherited.link(),
                _ => inherited,
            };
            collect_inline_runs(child_element, style, runs);
        }
    }
}

fn normalize_runs(runs: Vec<TextRun>) -> Vec<TextRun> {
    let mut normalized: Vec<TextRun> = Vec::new();

    for run in runs {
        for word in run.text.split_whitespace() {
            if !normalized.is_empty() {
                push_or_merge_run(
                    &mut normalized,
                    TextRun {
                        text: " ".to_string(),
                        style: run.style,
                    },
                );
            }

            push_or_merge_run(
                &mut normalized,
                TextRun {
                    text: word.to_string(),
                    style: run.style,
                },
            );
        }
    }

    normalized
}

fn push_or_merge_run(runs: &mut Vec<TextRun>, run: TextRun) {
    if run.text.is_empty() {
        return;
    }

    if let Some(last) = runs.last_mut() {
        if last.style == run.style {
            last.text.push_str(&run.text);
            return;
        }
    }

    runs.push(run);
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
    y: f32,
    spans: Vec<PdfSpan>,
}

#[derive(Debug, Clone)]
struct PdfSpan {
    text: String,
    x: f32,
    font_size: f32,
    font: PdfFont,
    color: PdfColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PdfFont {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PdfColor(f32, f32, f32);

#[derive(Debug, Clone)]
struct LineRun {
    text: String,
    style: RunStyle,
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
        let (prefix, content, style, color) = match block {
            DocumentBlock::Heading { level, content } => {
                let size = match level {
                    1 => 22.0,
                    2 => 17.0,
                    _ => 14.0,
                };
                (
                    "",
                    content.clone(),
                    BlockStyle {
                        font_size: size,
                        line_height: size + 7.0,
                        before: 10.0,
                        after: 8.0,
                    },
                    TEXT_BLACK,
                )
            }
            DocumentBlock::Paragraph(content) => {
                ("", content.clone(), body_style(4.0, 7.0), TEXT_BLACK)
            }
            DocumentBlock::Bullet(content) => {
                ("- ", content.clone(), body_style(3.0, 4.0), TEXT_BLACK)
            }
            DocumentBlock::Ordered { index, content } => {
                let prefix = format!("{index}. ");
                add_wrapped_block(
                    &mut pages,
                    &mut y,
                    &prefix,
                    content,
                    body_style(3.0, 4.0),
                    TEXT_BLACK,
                );
                continue;
            }
            DocumentBlock::Quote(content) => {
                ("> ", content.clone(), body_style(5.0, 7.0), TEXT_MUTED)
            }
            DocumentBlock::Code(content) => (
                "",
                content.clone(),
                BlockStyle {
                    font_size: 10.0,
                    line_height: 14.0,
                    before: 5.0,
                    after: 7.0,
                },
                TEXT_BLACK,
            ),
            DocumentBlock::Image { alt } => (
                "[Imagen] ",
                vec![TextRun {
                    text: alt.clone(),
                    style: RunStyle::default().italic(),
                }],
                body_style(5.0, 7.0),
                TEXT_MUTED,
            ),
        };

        add_wrapped_block(&mut pages, &mut y, prefix, &content, style, color);
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
    content: &[TextRun],
    style: BlockStyle,
    color: PdfColor,
) {
    *y -= style.before;
    let max_width = PAGE_WIDTH - MARGIN_LEFT * 2.0;
    let indent = if prefix.is_empty() { 0.0 } else { 18.0 };
    let lines = wrap_runs(content, style.font_size, max_width - indent);

    for (line_index, line_runs) in lines.iter().enumerate() {
        if *y < MARGIN_BOTTOM {
            pages.push(Vec::new());
            *y = PAGE_HEIGHT - MARGIN_TOP;
        }

        let mut x = MARGIN_LEFT + if line_index == 0 { 0.0 } else { indent };
        let mut spans = Vec::new();
        if line_index == 0 && !prefix.is_empty() {
            let font = PdfFont::Regular;
            spans.push(PdfSpan {
                text: prefix.to_string(),
                x,
                font_size: style.font_size,
                font,
                color,
            });
            x += text_width(prefix, style.font_size, font);
        }

        for run in line_runs {
            let font = font_for_style(run.style);
            let run_color = if run.style.link { LINK_BLUE } else { color };
            spans.push(PdfSpan {
                text: run.text.clone(),
                x,
                font_size: style.font_size,
                font,
                color: run_color,
            });
            x += text_width(&run.text, style.font_size, font);
        }

        pages
            .last_mut()
            .expect("at least one page")
            .push(PdfLine { y: *y, spans });
        *y -= style.line_height;
    }

    *y -= style.after;
}

fn wrap_runs(runs: &[TextRun], font_size: f32, max_width: f32) -> Vec<Vec<LineRun>> {
    let mut lines: Vec<Vec<LineRun>> = vec![Vec::new()];
    let mut current_width = 0.0;

    for run in runs {
        for word in run.text.split_whitespace() {
            let font = font_for_style(run.style);
            let needs_space = !lines.last().is_some_and(Vec::is_empty);
            let candidate = if needs_space {
                format!(" {word}")
            } else {
                word.to_string()
            };
            let candidate_width = text_width(&candidate, font_size, font);

            if current_width + candidate_width > max_width && current_width > 0.0 {
                lines.push(Vec::new());
                current_width = 0.0;
            }

            let text = if lines.last().is_some_and(Vec::is_empty) {
                word.to_string()
            } else {
                candidate
            };
            current_width += text_width(&text, font_size, font);
            push_or_merge_line_run(
                lines.last_mut().expect("at least one line"),
                LineRun {
                    text,
                    style: run.style,
                },
            );
        }
    }

    if lines.last().is_some_and(Vec::is_empty) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(Vec::new());
    }

    lines
}

fn push_or_merge_line_run(line: &mut Vec<LineRun>, run: LineRun) {
    if let Some(last) = line.last_mut() {
        if last.style == run.style {
            last.text.push_str(&run.text);
            return;
        }
    }

    line.push(run);
}

fn font_for_style(style: RunStyle) -> PdfFont {
    if style.code {
        PdfFont::Mono
    } else {
        match (style.bold, style.italic) {
            (true, true) => PdfFont::BoldItalic,
            (true, false) => PdfFont::Bold,
            (false, true) => PdfFont::Italic,
            (false, false) => PdfFont::Regular,
        }
    }
}

fn text_width(text: &str, font_size: f32, font: PdfFont) -> f32 {
    let factor = match font {
        PdfFont::Mono => 0.6,
        PdfFont::Bold | PdfFont::BoldItalic => 0.56,
        PdfFont::Regular | PdfFont::Italic => 0.52,
    };

    text.chars().count() as f32 * font_size * factor
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
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.0} {PAGE_HEIGHT:.0}] /Resources << /Font << /F1 {font_id} 0 R /F2 {} 0 R /F3 {} 0 R /F4 {} 0 R /F5 {} 0 R >> >> /Contents {content_id} 0 R >>",
            font_id + 1,
            font_id + 2,
            font_id + 3,
            font_id + 4
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            stream.len(),
            stream
        ));
    }

    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_string());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Oblique >>".to_string());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-BoldOblique >>".to_string());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>".to_string());

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
        for span in &line.spans {
            stream.push_str(&format!(
                "{:.3} {:.3} {:.3} rg BT /{} {:.1} Tf {:.1} {:.1} Td ({}) Tj ET\n",
                span.color.0,
                span.color.1,
                span.color.2,
                font_name(span.font),
                span.font_size,
                span.x,
                line.y,
                escape_pdf_text(&span.text)
            ));
        }
    }

    stream
}

fn font_name(font: PdfFont) -> &'static str {
    match font {
        PdfFont::Regular => "F1",
        PdfFont::Bold => "F2",
        PdfFont::Italic => "F3",
        PdfFont::BoldItalic => "F4",
        PdfFont::Mono => "F5",
    }
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
                    content: vec![TextRun {
                        text: "Contrato".to_string(),
                        style: RunStyle::default().bold(),
                    }],
                },
                DocumentBlock::Paragraph(vec![
                    TextRun {
                        text: "Hola".to_string(),
                        style: RunStyle::default(),
                    },
                    TextRun {
                        text: " mundo".to_string(),
                        style: RunStyle::default().bold(),
                    },
                ]),
                DocumentBlock::Bullet(vec![TextRun {
                    text: "Uno".to_string(),
                    style: RunStyle::default(),
                }]),
            ]
        );
    }

    #[test]
    fn html_to_blocks_should_preserve_inline_styles_and_links() {
        let blocks = html_to_blocks(
            r#"<p>Texto <strong>fuerte</strong> <em>italico</em> <code>ABC</code> <a href="https://example.com">link</a></p>"#,
        );

        assert_eq!(
            blocks,
            vec![DocumentBlock::Paragraph(vec![
                TextRun {
                    text: "Texto".to_string(),
                    style: RunStyle::default(),
                },
                TextRun {
                    text: " fuerte".to_string(),
                    style: RunStyle::default().bold(),
                },
                TextRun {
                    text: " italico".to_string(),
                    style: RunStyle::default().italic(),
                },
                TextRun {
                    text: " ABC".to_string(),
                    style: RunStyle::default().code(),
                },
                TextRun {
                    text: " link".to_string(),
                    style: RunStyle::default().link(),
                },
            ])]
        );
    }

    #[test]
    fn render_pdf_should_create_valid_pdf_header_and_trailer() {
        let bytes = render_pdf(&[DocumentBlock::Paragraph(vec![TextRun {
            text: "Documento de prueba".to_string(),
            style: RunStyle::default(),
        }])]);
        let pdf = String::from_utf8_lossy(&bytes);

        assert!(pdf.starts_with("%PDF-1.4"));
        assert!(pdf.contains("%%EOF"));
    }

    #[test]
    fn render_pdf_should_include_font_resources_for_inline_styles() {
        let blocks = html_to_blocks(
            r#"<p>Hola <strong>bold</strong> <em>italica</em> <code>COD</code> <a href="https://example.com">link</a></p>"#,
        );
        let bytes = render_pdf(&blocks);
        let pdf = String::from_utf8_lossy(&bytes);

        assert!(pdf.contains("/BaseFont /Helvetica-Bold"));
        assert!(pdf.contains("/BaseFont /Helvetica-Oblique"));
        assert!(pdf.contains("/BaseFont /Courier"));
        assert!(pdf.contains("0.145 0.388 0.922 rg"));
    }

    #[test]
    fn render_pdf_should_paginate_long_documents() {
        let blocks = (0..120)
            .map(|index| {
                DocumentBlock::Paragraph(vec![TextRun {
                    text: format!("Parrafo corporativo numero {index}"),
                    style: RunStyle::default(),
                }])
            })
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
