pub mod chunking;
pub mod embeddings;

pub use chunking::{chunk_text, Chunk, ChunkConfig, TokenEstimator};
pub use embeddings::{generate_embeddings, get_embeddings};

use calamine::{open_workbook_auto, Data, Reader};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    Pdf,
    Docx,
    Xlsx,
    PlainText,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractedDocument {
    pub path: String,
    pub file_name: String,
    pub format: DocumentFormat,
    pub text: String,
    pub char_count: usize,
    pub byte_count: u64,
    pub extraction_millis: u128,
}

#[derive(Debug, Error)]
pub enum IngestionError {
    #[error("No se encontro el archivo: {0}")]
    FileNotFound(String),
    #[error("La ruta no apunta a un archivo: {0}")]
    NotAFile(String),
    #[error("El formato del archivo no esta soportado: {0}")]
    UnsupportedFormat(String),
    #[error("El PDF no tiene capa de texto extraible. Se requiere OCR local para PDFs escaneados.")]
    OcrRequired,
    #[error("No se pudo leer el archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("No se pudo abrir el paquete del documento: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("No se pudo extraer texto de {format}: {message}")]
    Parse { format: &'static str, message: String },
}

pub fn extract_text_from_path(path: &Path) -> Result<ExtractedDocument, IngestionError> {
    validate_file(path)?;

    let started_at = Instant::now();
    let format = detect_document_format(path)?;
    let text = match format {
        DocumentFormat::Pdf => extract_pdf(path)?,
        DocumentFormat::Docx => extract_docx(path)?,
        DocumentFormat::Xlsx => extract_xlsx(path)?,
        DocumentFormat::PlainText => extract_plain_text(path)?,
    };

    let metadata = fs::metadata(path)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    Ok(ExtractedDocument {
        path: path.to_string_lossy().to_string(),
        file_name,
        format,
        char_count: text.chars().count(),
        byte_count: metadata.len(),
        text,
        extraction_millis: started_at.elapsed().as_millis(),
    })
}

fn validate_file(path: &Path) -> Result<(), IngestionError> {
    if !path.exists() {
        return Err(IngestionError::FileNotFound(path.display().to_string()));
    }

    if !path.is_file() {
        return Err(IngestionError::NotAFile(path.display().to_string()));
    }

    Ok(())
}

fn detect_document_format(path: &Path) -> Result<DocumentFormat, IngestionError> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    match extension.as_str() {
        "pdf" => Ok(DocumentFormat::Pdf),
        "docx" => Ok(DocumentFormat::Docx),
        "xlsx" | "xlsm" => Ok(DocumentFormat::Xlsx),
        "txt" | "md" | "csv" | "json" | "log" => Ok(DocumentFormat::PlainText),
        _ => Err(IngestionError::UnsupportedFormat(extension)),
    }
}

fn extract_plain_text(path: &Path) -> Result<String, IngestionError> {
    let bytes = fs::read(path)?;
    Ok(normalize_line_endings(&String::from_utf8_lossy(&bytes)))
}

fn extract_pdf(path: &Path) -> Result<String, IngestionError> {
    let text = pdf_extract::extract_text(path).map_err(|error| IngestionError::Parse {
        format: "pdf",
        message: error.to_string(),
    })?;
    ensure_pdf_text(text)
}

fn ensure_pdf_text(text: String) -> Result<String, IngestionError> {
    let text = normalize_line_endings(&text);
    if text.trim().is_empty() {
        return Err(IngestionError::OcrRequired);
    }

    Ok(text)
}

fn extract_docx(path: &Path) -> Result<String, IngestionError> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut document_xml = String::new();

    archive
        .by_name("word/document.xml")
        .map_err(|error| IngestionError::Parse {
            format: "docx",
            message: error.to_string(),
        })?
        .read_to_string(&mut document_xml)?;

    let text = text_from_word_document_xml(&document_xml)?;
    Ok(trim_trailing_blank_lines(&text))
}

fn text_from_word_document_xml(xml: &str) -> Result<String, IngestionError> {
    let mut reader = XmlReader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut text = String::new();
    let mut inside_text_node = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"w:t" | b"t" => inside_text_node = true,
                _ => {}
            },
            Ok(Event::Text(event)) => {
                if !inside_text_node {
                    continue;
                }

                let value = event.unescape().map_err(|error| IngestionError::Parse {
                    format: "docx",
                    message: error.to_string(),
                })?;
                text.push_str(value.as_ref());
            }
            Ok(Event::Empty(event)) => match event.name().as_ref() {
                b"w:tab" | b"tab" => text.push('\t'),
                b"w:br" | b"br" => text.push('\n'),
                _ => {}
            },
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"w:t" | b"t" => inside_text_node = false,
                b"w:p" | b"p" => text.push('\n'),
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(error) => {
                return Err(IngestionError::Parse {
                    format: "docx",
                    message: error.to_string(),
                });
            }
            _ => {}
        }
    }

    Ok(normalize_line_endings(&text))
}

fn extract_xlsx(path: &Path) -> Result<String, IngestionError> {
    let mut workbook = open_workbook_auto(path).map_err(|error| IngestionError::Parse {
        format: "xlsx",
        message: error.to_string(),
    })?;

    let mut sections = Vec::new();
    for sheet_name in workbook.sheet_names().to_owned() {
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|error| IngestionError::Parse {
                format: "xlsx",
                message: error.to_string(),
            })?;

        let rows = range
            .rows()
            .filter_map(|row| {
                let cells = row
                    .iter()
                    .filter_map(cell_to_text)
                    .collect::<Vec<_>>();

                if cells.is_empty() {
                    None
                } else {
                    Some(cells.join("\t"))
                }
            })
            .collect::<Vec<_>>();

        if !rows.is_empty() {
            sections.push(format!("Sheet: {sheet_name}\n{}", rows.join("\n")));
        }
    }

    Ok(sections.join("\n\n"))
}

fn cell_to_text(cell: &Data) -> Option<String> {
    match cell {
        Data::Empty => None,
        _ => {
            let value = cell.to_string();
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        }
    }
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn trim_trailing_blank_lines(text: &str) -> String {
    text.trim_end_matches('\n').to_string()
}

#[tauri::command]
pub fn extract_document_text(path: String) -> Result<ExtractedDocument, String> {
    let path = PathBuf::from(path);
    extract_text_from_path(&path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn chunk_extracted_text(text: String, config: ChunkConfig) -> Result<Vec<Chunk>, String> {
    Ok(chunk_text(&text, &config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    #[test]
    fn plain_text_extraction_preserves_content_with_normalized_line_endings() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("source.txt");
        fs::write(&path, "Linea 1\r\nLinea 2").expect("write text");

        let extracted = extract_text_from_path(&path).expect("extract text");

        assert_eq!(extracted.text, "Linea 1\nLinea 2");
        assert_eq!(extracted.format, DocumentFormat::PlainText);
    }

    #[test]
    fn unsupported_extension_returns_clear_error() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("source.bin");
        fs::write(&path, b"binary").expect("write binary");

        let error = extract_text_from_path(&path).expect_err("unsupported");

        assert!(error.to_string().contains("no esta soportado"));
    }

    #[test]
    fn docx_extraction_reads_paragraph_text() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("source.docx");
        write_docx_fixture(&path, "Contrato", "Cliente & Empresa");

        let extracted = extract_text_from_path(&path).expect("extract docx");

        assert_eq!(extracted.text, "Contrato\nCliente & Empresa");
        assert_eq!(extracted.format, DocumentFormat::Docx);
    }

    #[test]
    fn xlsx_extraction_groups_rows_by_sheet() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("source.xlsx");
        write_xlsx_fixture(&path);

        let extracted = extract_text_from_path(&path).expect("extract xlsx");

        assert!(extracted.text.contains("Sheet: Hoja1"));
        assert!(extracted.text.contains("Cliente\tImporte"));
        assert!(extracted.text.contains("Acme\t42"));
    }

    #[test]
    fn empty_pdf_text_is_reported_as_ocr_required() {
        let error = ensure_pdf_text(String::new()).expect_err("ocr required");

        assert!(matches!(error, IngestionError::OcrRequired));
    }

    fn write_docx_fixture(path: &Path, first: &str, second: &str) {
        let file = fs::File::create(path).expect("create docx");
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        let first = xml_escape(first);
        let second = xml_escape(second);
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>{first}</w:t></w:r></w:p>
    <w:p><w:r><w:t>{second}</w:t></w:r></w:p>
  </w:body>
</w:document>"#
        );

        zip.start_file("[Content_Types].xml", options)
            .expect("content types");
        zip.write_all(br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#)
            .expect("write content types");
        zip.start_file("word/document.xml", options)
            .expect("document xml");
        zip.write_all(xml.as_bytes()).expect("write document xml");
        zip.finish().expect("finish docx");
    }

    fn xml_escape(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    fn write_xlsx_fixture(path: &Path) {
        let file = fs::File::create(path).expect("create xlsx");
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default();

        write_zip_file(
            &mut zip,
            options,
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#,
        );
        write_zip_file(
            &mut zip,
            options,
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
        );
        write_zip_file(
            &mut zip,
            options,
            "xl/workbook.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Hoja1" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#,
        );
        write_zip_file(
            &mut zip,
            options,
            "xl/_rels/workbook.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#,
        );
        write_zip_file(
            &mut zip,
            options,
            "xl/worksheets/sheet1.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    <row r="1"><c r="A1" t="inlineStr"><is><t>Cliente</t></is></c><c r="B1" t="inlineStr"><is><t>Importe</t></is></c></row>
    <row r="2"><c r="A2" t="inlineStr"><is><t>Acme</t></is></c><c r="B2"><v>42</v></c></row>
  </sheetData>
</worksheet>"#,
        );

        zip.finish().expect("finish xlsx");
    }

    fn write_zip_file(
        zip: &mut zip::ZipWriter<fs::File>,
        options: SimpleFileOptions,
        name: &str,
        contents: &str,
    ) {
        zip.start_file(name, options).expect("start zip file");
        zip.write_all(contents.as_bytes()).expect("write zip file");
    }
}
