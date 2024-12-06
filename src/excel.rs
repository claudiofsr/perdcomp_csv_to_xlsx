use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use itertools::{self, Itertools};
use rust_xlsxwriter::{
    Table,
    Format,
    FormatAlign,
    Workbook,
    Worksheet,
};
use serde::{Serialize, Deserialize};
use serde_aux::prelude::serde_introspect;
use struct_iterable::Iterable;
use std::{
    collections::HashMap,
    path::Path,
};

// use rayon::prelude::*;

use crate::{
    MyResult,
    REGEX_CENTER, REGEX_VALUE, REGEX_DATE, REGEX_ALIQ,
};

const FONT_SIZE: f64 = 10.0;
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

/// Add some methods to Info struct
///
/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
pub trait InfoExtension {

    /**
    Gets the serialization names for structs and enums.

    use serde_aux::prelude::serde_introspect;

    <https://docs.rs/serde-aux/latest/src/serde_aux/serde_introspection.rs.html>
    */
    fn get_headers<'de>() -> &'static [&'static str]
    where
        Self: Deserialize<'de>
    {
        serde_introspect::<Self>()
    }
}

/// Write XLSX File according to some struct T
///
/// The lines (or rows) are given by &[T]
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
pub fn write_xlsx<'de, T, P>(lines: &[T], sheet_name: &str, output_file: P) -> MyResult<()>
where
    P: AsRef<Path> + std::marker::Copy + std::fmt::Debug,
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    if lines.is_empty() {
        return Ok(());
    }

    eprintln!("Write XLSX File: {:?}", output_file);

    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Split a vector into smaller vectors of size N
    for (index, data) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let mut new_name = sheet_name.to_string();

        if index >= 1 {
            new_name = format!("{} {}", sheet_name, index + 1);
        }

        // Get worksheet with sheet name.
        let worksheet: Worksheet = get_worksheet(data, &new_name)?;

        workbook.push_worksheet(worksheet);
    }

    // Save the workbook to disk.
    workbook.save(output_file)
        .inspect_err(|e| {
            // Add a custom error message
            eprintln!("fn write_xlsx()");
            eprintln!("File {:?}", output_file);
            eprintln!("Failed to write XLSX file: {e}")
        })?;

    Ok(())
}

/// Get Worksheet according to some struct T
fn get_worksheet<'de, T>(lines: &[T], sheet_name: &str) -> MyResult<Worksheet>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    let column_names: &[&str] = T::get_headers(); // <-- InfoExtension
    let column_number: u16 = column_names.len().try_into()?;
    let row_number: u32 = lines.len().try_into()?;

    // println!("column_names: {column_names:#?}");

    // Add some formats to use with the serialization data.
    let fmt: HashMap<&str, Format> = create_formats();

    let mut worksheet = Worksheet::new();

    worksheet
        .set_name(sheet_name)?
        .set_row_height(0, 64)?
        .set_row_format(0, fmt.get("header").unwrap())?
        .set_freeze_panes(1, 0)?;

    // Set up the start location and headers of the data to be serialized.
    worksheet.deserialize_headers::<T>(0, 0)?;

    format_columns_by_names(&mut worksheet, &fmt, column_names)?;

    // Create and configure a new table.
    // Why LibreOffice Calc not recognize the table styles?
    let table = Table::new()
        .set_autofilter(true)
        .set_total_row(false);

    // Add the table to the worksheet.
    worksheet.add_table(0, 0, row_number, column_number - 1, &table)?;

    for line in lines {
        // Serialize the data.
        worksheet.serialize(line)?;
    }

    /*
    lines
        .iter()
        .try_for_each(|line| -> MyResult<()> {
            // Serialize the data.
            worksheet.serialize(line)?;
            Ok(())
        })?;
    */

    // worksheet.autofit();
    auto_fit(&mut worksheet, lines, column_names)?;

    Ok(worksheet)
}

/// Add some formats to use with the serialization data.
fn create_formats() -> HashMap<&'static str, Format> {

    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::Center) // horizontally
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(FONT_SIZE);

    let fmt_center = Format::new()
        .set_align(FormatAlign::Center);

    let fmt_value = Format::new()
        .set_num_format("#,##0.00"); // 2 digits after the decimal point

    let fmt_aliq = Format::new()
        .set_num_format("#,##0.0000"); // 4 digits after the decimal point

    let fmt_date: Format = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_num_format("dd/mm/yyyy");

    // https://github.com/jmcnamara/rust_xlsxwriter/issues/81
    //let fmt_default = Format::new().set_text_wrap();

    HashMap::from([
        ("header", fmt_header),
        ("center", fmt_center),
        ("value",  fmt_value),
        ("aliq",   fmt_aliq),
        ("date",   fmt_date),
    //("defaut", fmt_defaut),
    ])
}

/// Format columns by names using regex
fn format_columns_by_names(
    worksheet: &mut Worksheet,
    fmt: &HashMap<&str, Format>,
    column_names: &[&str],
) -> MyResult<()> {

    for (index, col_name) in column_names.iter().enumerate() {

        let column_number: u16 = index.try_into()?;

        if REGEX_CENTER.is_match(col_name) {
            worksheet.set_column_format(column_number, fmt.get("center").unwrap())?;
            continue;
        }

        if REGEX_VALUE.is_match(col_name) {
            worksheet.set_column_format(column_number, fmt.get("value").unwrap())?;
            continue;
        }

        if REGEX_ALIQ.is_match(col_name) {
            worksheet.set_column_format(column_number, fmt.get("aliq").unwrap())?;
            continue;
        }

        if REGEX_DATE.is_match(col_name) {
            worksheet.set_column_format(column_number, fmt.get("date").unwrap())?;
            continue;
        }

        //worksheet.set_column_format(column_number, fmt.get("defaut").unwrap())?;
    }

    Ok(())
}

/// Iterate over all data and find the max data width for each column.
fn auto_fit<'de, T>(
    worksheet: &mut Worksheet,
    lines: &[T],
    column_names: &[&str],
) -> MyResult<()>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    // HashMap<col index, col width>:
    let mut max_length: HashMap<usize, usize> = HashMap::new();

    let width_min = 8;
    let width_max = 80;
    let adjustment = 1.2;

    column_names
        .iter()
        .enumerate()
        .for_each(|(col_index, col_name)| {
            // Init values: add column name lengths
            let col_len = col_name.chars_count().div_ceil(4);
            let col_width = width_min.max(col_len);
            max_length.insert(col_index, col_width);
        });

    lines
        .iter()
        //.into_par_iter() // rayon parallel iterator
        .for_each(|line| {
            get_length_of_column_values(line, &mut max_length)
        });

    for (index, len) in max_length {
        let width = width_max.min(len);
        //println!("{index:>2} {}: {width}", column_names[index]);
        worksheet.set_column_width(index as u16, (width as f64) * adjustment)?;
    }

    Ok(())
}

/// Match through different types.
///
/// Font: <https://github.com/therustmonk/match_cast/blob/master/src/lib.rs>
macro_rules! match_cast {
    ($any:ident { $( $bind:ident as $patt:ty => $body:block , )+ }) => {{
        let downcast = || {
            $(
            if let Some($bind) = $any.downcast_ref::<$patt>() {
                return $body;
            }
            )+
            None
        };
        downcast()
    }};
}

/// Struct Iterable is a Rust library that provides a proc macro to make a struct iterable.
///
/// use struct_iterable::Iterable
///
/// <https://crates.io/crates/struct_iterable>
fn get_length_of_column_values<'de, T>(line: &T, max_length: &mut HashMap<usize, usize>)
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable
{
    line
        .iter()
        .enumerate()
        .for_each( |(index, (_field_name, field_value))| {

            // Get the length of field_value: &dyn Any.
            // <https://doc.rust-lang.org/beta/core/any/index.html>

            let field_value_len: usize = match_cast!( field_value {
                val as Option<u8> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<u16> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<u32> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<usize> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<f64> => {
                    val.as_ref().map(|f| f.to_string().chars_count())
                },
                val as Option<NaiveDate> => {
                    val.as_ref().map(|date| date.to_string().chars_count())
                },
                val as Option<String> => {
                    val.as_deref().map(|s| s.chars_count())
                },
                val as String => {
                    Some(val.chars_count())
                },
                val as Vec<String> => {
                    //Some(val.iter().map(|s| s.chars_count()).sum())
                    // use itertools;
                    Some(val.iter().join(", ").chars_count())
                },
            }).unwrap_or(format!("{field_value:?}").chars_count());

            let length: usize = *max_length.get(&index).unwrap_or(&0);

            if field_value_len > length {
                max_length.insert(index, field_value_len);
            }
        });
}
