use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::write::SimpleFileOptions;

#[derive(Debug, Deserialize)]
pub struct ExportStructuredTableXlsxRequest {
    pub columns: Vec<ExportTableColumn>,
    pub rows: Vec<HashMap<String, String>>,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportTableColumn {
    pub name: String,
    #[serde(rename = "data_type")]
    pub _data_type: String,
    #[serde(rename = "nullable")]
    pub _nullable: bool,
}

#[derive(Debug, Serialize)]
pub struct ExportStructuredTableXlsxResult {
    pub path: String,
    pub format: String,
}

#[derive(Debug, Error)]
pub enum TableXlsxExportError {
    #[error("The table must contain at least one column.")]
    EmptyColumns,
    #[error("The table must contain at least one row.")]
    EmptyRows,
    #[error("Column names cannot be empty.")]
    EmptyColumnName,
    #[error("Could not resolve the user's downloads or desktop directory.")]
    OutputDirectoryNotFound,
    #[error("Could not write XLSX archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Could not write exported XLSX file: {0}")]
    Io(#[from] std::io::Error),
}

pub async fn export_structured_table_xlsx(
    request: ExportStructuredTableXlsxRequest,
) -> Result<ExportStructuredTableXlsxResult, TableXlsxExportError> {
    let filename = request
        .filename
        .as_deref()
        .map(sanitize_filename)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "zeldrix-ai-table".to_string());
    let path = unique_output_path(&filename, "xlsx")?;
    let bytes = build_table_xlsx(&request)?;

    fs::write(&path, bytes)?;

    Ok(ExportStructuredTableXlsxResult {
        path: path.to_string_lossy().to_string(),
        format: "xlsx".to_string(),
    })
}

pub fn build_table_xlsx(
    table: &ExportStructuredTableXlsxRequest,
) -> Result<Vec<u8>, TableXlsxExportError> {
    validate_table(table)?;

    let cursor = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(cursor);
    let options = SimpleFileOptions::default();

    write_zip_file(&mut zip, options, "[Content_Types].xml", content_types_xml())?;
    write_zip_file(&mut zip, options, "_rels/.rels", root_relationships_xml())?;
    write_zip_file(&mut zip, options, "docProps/core.xml", core_properties_xml())?;
    write_zip_file(&mut zip, options, "docProps/app.xml", app_properties_xml())?;
    write_zip_file(&mut zip, options, "xl/workbook.xml", workbook_xml())?;
    write_zip_file(
        &mut zip,
        options,
        "xl/_rels/workbook.xml.rels",
        workbook_relationships_xml(),
    )?;
    write_zip_file(&mut zip, options, "xl/styles.xml", styles_xml())?;
    write_zip_file(
        &mut zip,
        options,
        "xl/worksheets/sheet1.xml",
        &worksheet_xml(table),
    )?;

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

fn validate_table(table: &ExportStructuredTableXlsxRequest) -> Result<(), TableXlsxExportError> {
    if table.columns.is_empty() {
        return Err(TableXlsxExportError::EmptyColumns);
    }

    if table.rows.is_empty() {
        return Err(TableXlsxExportError::EmptyRows);
    }

    if table.columns.iter().any(|column| column.name.trim().is_empty()) {
        return Err(TableXlsxExportError::EmptyColumnName);
    }

    Ok(())
}

fn write_zip_file(
    zip: &mut zip::ZipWriter<Cursor<Vec<u8>>>,
    options: SimpleFileOptions,
    name: &str,
    contents: &str,
) -> Result<(), TableXlsxExportError> {
    zip.start_file(name, options)?;
    zip.write_all(contents.as_bytes())?;
    Ok(())
}

fn worksheet_xml(table: &ExportStructuredTableXlsxRequest) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetViews><sheetView workbookViewId="0"/></sheetViews>
"#,
    );

    xml.push_str(&columns_xml(table.columns.len()));
    xml.push_str("  <sheetData>\n");
    xml.push_str("    <row r=\"1\">");

    for (column_index, column) in table.columns.iter().enumerate() {
        let cell_reference = cell_reference(column_index, 1);
        xml.push_str(&format!(
            r#"<c r="{cell_reference}" s="1" t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
            escape_xml_text(&column.name)
        ));
    }

    xml.push_str("</row>\n");

    for (row_index, row) in table.rows.iter().enumerate() {
        let excel_row_index = row_index + 2;
        xml.push_str(&format!("    <row r=\"{excel_row_index}\">"));

        for (column_index, column) in table.columns.iter().enumerate() {
            let cell_reference = cell_reference(column_index, excel_row_index);
            let value = row.get(&column.name).map(String::as_str).unwrap_or_default();
            xml.push_str(&format!(
                r#"<c r="{cell_reference}" t="inlineStr"><is><t xml:space="preserve">{}</t></is></c>"#,
                escape_xml_text(value)
            ));
        }

        xml.push_str("</row>\n");
    }

    xml.push_str(
        r#"  </sheetData>
  <autoFilter ref=""#,
    );
    xml.push_str(&format!(
        "A1:{}{}",
        column_name(table.columns.len() - 1),
        table.rows.len() + 1
    ));
    xml.push_str(
        r#""/>
  <pageMargins left="0.7" right="0.7" top="0.75" bottom="0.75" header="0.3" footer="0.3"/>
</worksheet>"#,
    );

    xml
}

fn columns_xml(column_count: usize) -> String {
    let mut xml = String::from("  <cols>\n");

    for column_index in 0..column_count {
        let excel_column_index = column_index + 1;
        xml.push_str(&format!(
            r#"    <col min="{excel_column_index}" max="{excel_column_index}" width="18" customWidth="1"/>"#
        ));
        xml.push('\n');
    }

    xml.push_str("  </cols>\n");
    xml
}

fn content_types_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#
}

fn root_relationships_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#
}

fn workbook_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Resultados" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#
}

fn workbook_relationships_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#
}

fn styles_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="2">
    <font><sz val="11"/><color theme="1"/><name val="Calibri"/><family val="2"/></font>
    <font><b/><sz val="11"/><color rgb="FFFFFFFF"/><name val="Calibri"/><family val="2"/></font>
  </fonts>
  <fills count="3">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="gray125"/></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FF1F2937"/><bgColor indexed="64"/></patternFill></fill>
  </fills>
  <borders count="2">
    <border><left/><right/><top/><bottom/><diagonal/></border>
    <border><left/><right/><top/><bottom style="thin"><color rgb="FFD1D5DB"/></bottom><diagonal/></border>
  </borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="2">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
    <xf numFmtId="0" fontId="1" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"/>
  </cellXfs>
  <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
  <dxfs count="0"/>
  <tableStyles count="0" defaultTableStyle="TableStyleMedium2" defaultPivotStyle="PivotStyleLight16"/>
</styleSheet>"#
}

fn core_properties_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:creator>Zeldrix</dc:creator>
  <cp:lastModifiedBy>Zeldrix</cp:lastModifiedBy>
</cp:coreProperties>"#
}

fn app_properties_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <Application>Zeldrix</Application>
</Properties>"#
}

fn escape_xml_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| is_valid_xml_char(*ch))
        .flat_map(|ch| match ch {
            '&' => "&amp;".chars().collect::<Vec<_>>(),
            '<' => "&lt;".chars().collect(),
            '>' => "&gt;".chars().collect(),
            _ => vec![ch],
        })
        .collect()
}

fn is_valid_xml_char(ch: char) -> bool {
    matches!(ch, '\u{9}' | '\u{A}' | '\u{D}' | '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}')
}

fn cell_reference(column_index: usize, row_index: usize) -> String {
    format!("{}{}", column_name(column_index), row_index)
}

fn column_name(mut column_index: usize) -> String {
    let mut name = String::new();

    loop {
        let remainder = column_index % 26;
        name.insert(0, char::from(b'A' + remainder as u8));

        if column_index < 26 {
            break;
        }

        column_index = (column_index / 26) - 1;
    }

    name
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

fn unique_output_path(filename: &str, extension: &str) -> Result<PathBuf, TableXlsxExportError> {
    let output_dir = UserDirs::new()
        .and_then(|dirs| {
            dirs.download_dir()
                .or_else(|| dirs.desktop_dir())
                .map(Path::to_path_buf)
        })
        .ok_or(TableXlsxExportError::OutputDirectoryNotFound)?;

    let mut candidate = output_dir.join(format!("{filename}.{extension}"));
    let mut counter = 2;

    while candidate.exists() {
        candidate = output_dir.join(format!("{filename}-{counter}.{extension}"));
        counter += 1;
    }

    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use zip::ZipArchive;

    #[test]
    fn build_table_xlsx_should_create_required_workbook_entries() {
        let bytes = build_table_xlsx(&sample_request()).expect("xlsx bytes");
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).expect("zip archive");

        assert!(archive.by_name("[Content_Types].xml").is_ok());
        assert!(archive.by_name("xl/workbook.xml").is_ok());
        assert!(archive.by_name("xl/styles.xml").is_ok());
        assert!(archive.by_name("xl/worksheets/sheet1.xml").is_ok());
    }

    #[test]
    fn build_table_xlsx_should_escape_headers_and_values() {
        let bytes = build_table_xlsx(&sample_request()).expect("xlsx bytes");
        let worksheet = read_zip_entry(bytes, "xl/worksheets/sheet1.xml");

        assert!(worksheet.contains("Cliente &amp; Socio"));
        assert!(worksheet.contains(r#"<t xml:space="preserve"> ACME  &lt;Peru&gt; </t>"#));
    }

    #[test]
    fn build_table_xlsx_should_reject_empty_columns() {
        let mut request = sample_request();
        request.columns.clear();

        let error = build_table_xlsx(&request).expect_err("empty columns must fail");

        assert!(matches!(error, TableXlsxExportError::EmptyColumns));
    }

    #[test]
    fn build_table_xlsx_should_reject_empty_rows() {
        let mut request = sample_request();
        request.rows.clear();

        let error = build_table_xlsx(&request).expect_err("empty rows must fail");

        assert!(matches!(error, TableXlsxExportError::EmptyRows));
    }

    #[test]
    fn column_name_should_support_columns_after_z() {
        assert_eq!(column_name(0), "A");
        assert_eq!(column_name(25), "Z");
        assert_eq!(column_name(26), "AA");
        assert_eq!(column_name(51), "AZ");
        assert_eq!(column_name(52), "BA");
    }

    fn sample_request() -> ExportStructuredTableXlsxRequest {
        ExportStructuredTableXlsxRequest {
            columns: vec![
                ExportTableColumn {
                    name: "Cliente & Socio".to_string(),
                    _data_type: "string".to_string(),
                    _nullable: false,
                },
                ExportTableColumn {
                    name: "Importe".to_string(),
                    _data_type: "currency".to_string(),
                    _nullable: false,
                },
            ],
            rows: vec![HashMap::from([
                ("Cliente & Socio".to_string(), " ACME  <Peru> ".to_string()),
                ("Importe".to_string(), "42.50".to_string()),
            ])],
            filename: None,
        }
    }

    fn read_zip_entry(bytes: Vec<u8>, entry_name: &str) -> String {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        let mut entry = archive.by_name(entry_name).expect("zip entry");
        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .expect("read zip entry");
        contents
    }
}
