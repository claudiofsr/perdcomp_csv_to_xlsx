//! This module handles exporting structured data vectors to Excel files.
//! It acts as a pure, decoupled generic exporter, utilizing global workbook
//! formats to ensure all cells scale cleanly.
//!
//! Visual metadata and styles are handled by the serialized types, while
//! this module coordinates workbook construction, parallel worksheet populating,
//! column hiding, and diagnostic logging.

use rayon::prelude::*;
use rust_xlsxwriter::{Format, FormatAlign, Workbook, Worksheet, XlsxError, XlsxSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

use crate::structures::FONT_SIZE;

/// The maximum number of rows allowed in a single worksheet.
/// Excel's strict physical limit is 1,048,576 rows. We split at 1,000,000
/// to maintain a clean margin.
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

/// Writes any slice of serializable items implementing XlsxSerialize into an Excel file.
///
/// This function acts as a high-performance concurrent orchestrator. Worksheets are generated,
/// serialized, and auto-fitted in parallel across multiple CPU cores using `rayon`.
/// Once populated, sheets are pushed sequentially to a final `Workbook` and saved to disk.
///
/// # Errors
///
/// Returns an [`XlsxError`] if the format setup, parallel serialization, or file system
/// write operation encounters an issue.
pub fn write_xlsx<'de, T, P>(
    lines: &[T],
    sheet_name: &str,
    output_file: P,
    hide_cols: &[u16],
    verbose: bool,
) -> Result<(), XlsxError>
where
    P: AsRef<Path>,
    T: Serialize + Deserialize<'de> + XlsxSerialize + Send + Sync,
{
    // 1. Exit early if the dataset is empty to prevent creating a corrupted, zero-byte file.
    if lines.is_empty() {
        eprintln!("Warning: Input data is empty. Skipping XLSX generation.");
        return Ok(());
    }

    let output_path = output_file.as_ref();

    // Log the file creation process.
    eprintln!("Write XLSX File (Parallel Mode): {output_path:?}");

    // 2. Concurrently calculate optimal column widths using Rayon.
    // This parses structures on worker threads to avoid stalling the main writer process.
    let col_widths = calculate_max_column_widths(lines, verbose);

    // 3. Initialize a new empty Excel workbook.
    let mut workbook = Workbook::new();

    // 4. Define fallback formatting rules for standard worksheet cells.
    // This vertically centers content and applies standard configuration sizes.
    let default_format = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_font_size(FONT_SIZE);

    // 5. Establish global workbook formatting defaults.
    // Set standard rows to 24pt height and columns to a fallback width of 80pt.
    workbook.set_default_format(&default_format, 24, 80)?;

    // 6. Partition datasets into parallel chunks and generate worksheets concurrently.
    // This avoids thread-blocking bottlenecks during major document assembly tasks.
    let worksheets_result: Result<Vec<Worksheet>, XlsxError> = lines
        .par_chunks(MAX_NUMBER_OF_ROWS)
        .enumerate()
        .map(|(index, data_chunk)| {
            let dynamic_sheet_name = format_sheet_name(sheet_name, index + 1);
            if index > 0 {
                eprintln!(
                    "Notice: Dataset size ({}) exceeds limit ({}).
                    Preparing additional sheet in parallel: {dynamic_sheet_name}",
                    lines.len(),
                    MAX_NUMBER_OF_ROWS
                );
            }
            create_and_populate_worksheet(&dynamic_sheet_name, hide_cols, &col_widths, data_chunk)
        })
        .collect();

    // 7. Sequentially push completed worksheets onto the main thread's workbook registry.
    for worksheet in worksheets_result? {
        workbook.push_worksheet(worksheet);
    }

    // 8. Commit structural changes to disk, logging any filesystem issues encountered.
    workbook.save(output_path).inspect_err(|err| {
        eprintln!("fn write_xlsx()");
        eprintln!("File {output_path:?}");
        eprintln!("Failed to write XLSX file: {err}");
    })?;

    // Log the successful file generation path.
    eprintln!("Success: XLSX file generated at {output_path:?}");
    Ok(())
}

/// Names the worksheet depending on the chunk split index.
///
/// Suffixes are omitted if only one chunk exists.
fn format_sheet_name(base_name: &str, index: usize) -> String {
    if index > 1 {
        format!("{} {}", base_name, index)
    } else {
        base_name.to_string()
    }
}

/// Creates, fully configures, populates, and adjusts a single independent worksheet instance.
///
/// This function does not reference any active workbook lifetimes, making it fully Send
/// and suitable for generation on concurrent threads.
fn create_and_populate_worksheet<'de, T>(
    sheet_name: &str,
    hide_cols: &[u16],
    col_widths: &[u16],
    data: &[T],
) -> Result<Worksheet, XlsxError>
where
    T: Serialize + Deserialize<'de> + XlsxSerialize,
{
    // Log the worksheet assembly process, matching the desired columns and rows count.
    eprintln!(
        "Info: Populating worksheet '{}' with {} columns and {} rows.",
        sheet_name,
        col_widths.len(),
        data.len()
    );

    let mut worksheet = Worksheet::new();
    worksheet.set_name(sheet_name)?;

    // 1. Serialize headers.
    // They will inherit the struct level `#[xlsx(header_format = ...)]` automatically.
    worksheet.set_serialize_headers::<T>(0, 0)?;

    // 2. Serialize the data rows.
    // Fields mapped with `#[xlsx(value_format = ...)]` apply explicit layouts,
    // while unformatted fields fall back to the workbook's Calibri 14 default.
    worksheet.serialize(&data)?;

    // 3. Configure the exact height of the header row (Row 0) to 42.0 pixels/points.
    worksheet.set_row_height(0, 62.0)?;
    worksheet.set_freeze_panes(1, 0)?;

    // 4. Set dynamically calculated column widths.
    for (col_idx, &width) in col_widths.iter().enumerate() {
        worksheet.set_column_width(col_idx as u16, width as f64)?;
    }

    // 5. Hide target empty or requested columns.
    if !hide_cols.is_empty() {
        eprintln!(
            "Info: Hiding {} columns in worksheet '{}'...",
            hide_cols.len(),
            sheet_name
        );
        for &col_idx in hide_cols {
            worksheet.set_column_hidden(col_idx)?;
        }
    }

    // Log the successful population statement.
    eprintln!(
        "Info: Worksheet '{}' populated and formatted successfully.",
        sheet_name
    );
    Ok(worksheet)
}

/// Computes the maximum character length of all cells for each column in parallel.
///
/// By converting records to intermediate `serde_json::Value` structures, this function
/// scans the grid across multiple threads using Rayon. It determines baseline widths
/// from header keys, checks row contents, adds padding for visual readability,
/// and applies safety boundaries to prevent excessively narrow or wide columns.
///
/// # Performance
///
/// Rayon divides the dataset into balanced workloads. Parallel reduction merges
/// the local maximum widths with zero allocation overhead during reduction steps.
pub fn calculate_max_column_widths<'de, T>(data: &[T], verbose: bool) -> Vec<u16>
where
    T: Serialize + Deserialize<'de> + Send + Sync,
{
    // 1. Safe boundary exit for empty slices.
    if data.is_empty() {
        return Vec::new();
    }

    // 2. Convert the first record to an intermediate JSON format to evaluate structure properties.
    let first_val = serde_json::to_value(&data[0]).unwrap_or(Value::Null);

    // 3. Print the first data row properties if verbose diagnostics mode is active.
    if verbose && let Ok(pretty_json) = serde_json::to_string_pretty(&first_val) {
        eprintln!(
            "First data row representation (explicit structure check):\n{}\n",
            pretty_json
        );
    }

    let mut headers = Vec::new();

    // 4. Capture character lengths of fields to initialize header baseline widths.
    if let Value::Object(ref map) = first_val {
        for key in map.keys() {
            headers.push(key.chars().count() as u16);
        }
    } else {
        return Vec::new();
    }

    let num_cols = headers.len();

    // 5. Parallel search over split chunks.
    // Rayon divides task sequences and processes values concurrently to build a local maximum width map.
    let max_lengths = data
        .par_chunks(1024)
        .map(|chunk| {
            let mut local_maxes = vec![0u16; num_cols];
            for item in chunk {
                if let Ok(Value::Object(map)) = serde_json::to_value(item) {
                    for (i, value) in map.values().enumerate() {
                        if i < num_cols {
                            let len = get_value_length(value);
                            if len > local_maxes[i] {
                                local_maxes[i] = len;
                            }
                        }
                    }
                }
            }
            local_maxes
        })
        .reduce(
            || vec![0u16; num_cols],
            |mut acc, local| {
                for i in 0..num_cols {
                    acc[i] = acc[i].max(local[i]);
                }
                acc
            },
        );

    // 6. Map elements to computed column widths, adding safety padding and bounding limits.
    max_lengths
        .into_iter()
        .enumerate()
        .map(|(i, max_val_len)| {
            // Divide the raw header length by 4 to let titles wrap comfortably.
            let header_len = headers[i] / 4;
            let raw_max = header_len.max(max_val_len);

            // Add safe padding to handle font metrics and clamp results within standard visual boundaries.
            (raw_max + 2).clamp(8, 100)
        })
        .collect()
}

/// Approximates the string length of a serialized JSON field.
///
/// For string types (including dates serialized in ISO 8601 "YYYY-MM-DD" format),
/// it returns the direct character count. The 10-character length of "YYYY-MM-DD"
/// matches the 10-character length of "DD/MM/YYYY" rendered in Excel.
fn get_value_length(value: &Value) -> u16 {
    match value {
        Value::Null => 0,
        Value::String(s) => s.chars().count() as u16,
        Value::Number(n) => estimate_formatted_number_length(n),
        Value::Bool(b) => {
            if *b {
                4
            } else {
                5
            }
        }
        _ => 0,
    }
}

/// Estimates the final rendered length of a number formatted in Excel
/// with standard thousands separators and decimals (e.g., `#,##0.00`).
fn estimate_formatted_number_length(n: &serde_json::Number) -> u16 {
    if let Some(val_f64) = n.as_f64() {
        let abs_val = val_f64.abs();
        if !abs_val.is_finite() {
            return 0;
        }

        // 1. Compute digits count of the integer portion mathematically using base-10 log.
        let int_part = abs_val.trunc() as i64;
        let digits = if int_part == 0 {
            1
        } else {
            (int_part as f64).log10().floor() as u16 + 1
        };

        // 2. Count the number of financial thousands separators (periods).
        let separators = (digits - 1) / 3;

        // 3. Standard decimal suffix addition (e.g., ",00" represents 3 extra characters).
        let decimal_and_separator = 3;

        // 4. Check if a negative sign character (-) needs to be accounted for.
        let sign = if val_f64 < 0.0 { 1 } else { 0 };

        digits + separators + decimal_and_separator + sign
    } else {
        n.to_string().chars().count() as u16
    }
}
