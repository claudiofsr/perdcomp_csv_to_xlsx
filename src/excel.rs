use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet, XlsxSerialize};
use serde::{Deserialize, Serialize};
use std::path::Path;

// use rayon::prelude::*;

use crate::MyResult;

const FONT_SIZE: f64 = 10.0;
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

/// Write XLSX File according to some struct T
///
/// The lines (or rows) are given by &[T]
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
pub fn write_xlsx<'de, T, P>(
    lines: &[T],
    sheet_name: &str,
    output_file: P,
    hide_cols: &[u16],
) -> MyResult<()>
where
    P: AsRef<Path> + std::fmt::Debug,
    T: Serialize + Deserialize<'de> + XlsxSerialize, // + Sync + Send
{
    if lines.is_empty() {
        return Ok(());
    }

    eprintln!("Write XLSX File: {output_file:?}");

    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::Center) // horizontally
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(FONT_SIZE);

    // Split a vector into smaller vectors of size N
    for (index, data) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let mut new_name = sheet_name.to_string();

        if index >= 1 {
            new_name = format!("{} {}", sheet_name, index + 1);
        }

        // Get worksheet with sheet name.
        let mut worksheet: Worksheet = get_worksheet(data, &new_name, &fmt_header)?;

        // Oculta colunas vazias APENAS se a lista não for vazia
        for &col_idx in hide_cols {
            worksheet.set_column_hidden(col_idx)?;
        }

        workbook.push_worksheet(worksheet);
    }

    // Save the workbook to disk.
    workbook.save(output_file.as_ref()).inspect_err(|e| {
        // Add a custom error message
        eprintln!("fn write_xlsx()");
        eprintln!("File {output_file:?}");
        eprintln!("Failed to write XLSX file: {e}")
    })?;

    Ok(())
}

/// Get Worksheet according to some struct T
///
/// <https://github.com/jmcnamara/rust_xlsxwriter/blob/main/examples/app_serialize.rs>
///
/// <https://github.com/jmcnamara/rust_xlsxwriter/blob/main/tests/integration/serde06.rs>
fn get_worksheet<'de, T>(lines: &[T], sheet_name: &str, fmt_header: &Format) -> MyResult<Worksheet>
where
    T: Serialize + Deserialize<'de> + XlsxSerialize, // + Sync + Send
{
    let mut worksheet = Worksheet::new();

    worksheet
        .set_name(sheet_name)?
        .set_row_height(0, 64)?
        .set_row_format(0, fmt_header)?
        .set_freeze_panes(1, 0)?;

    // Set the serialization location and headers.
    worksheet.set_serialize_headers::<T>(0, 0)?;

    worksheet.serialize(&lines)?;

    //worksheet.autofit_to_max_width(150);
    worksheet.autofit();

    Ok(worksheet)
}
