mod args;
mod excel;
mod get_field_names;
mod regex;
mod structures;

pub use args::Arguments;
pub use excel::write_xlsx;
pub use get_field_names::GetFieldNames;
pub use structures::PerDcomp;
pub use regex::*;

use csv::ReaderBuilder;
use claudiofsr_lib::BytesExtension;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use std::{
    str,
    process,
    io::Read,
    convert::AsRef,
    path::{Path, PathBuf},
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{
        BufRead,
        BufReader,
        BufWriter,
        Write,
    },
    sync::Arc,
};

pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;

pub const NEWLINE_BYTE: u8 = b'\n';

#[derive(Debug)]
pub struct Paths {
    pub input: Arc<Path>,
    pub output: Arc<Path>,
}

/// Get the first line of the file
///
/// <https://doc.rust-lang.org/std/io/trait.BufRead.html#method.read_until>
pub fn get_first_line(paths: &Paths) -> MyResult<String> {
    let file_input: File = OpenOptions::new()
        .read(true)
        .write(false)
        .open(&paths.input)?;

    let mut buffer_input = BufReader::new(file_input);
    let mut vec_bytes: Vec<u8> = Vec::new();

    let _number_of_bytes: usize = buffer_input
        .read_until(NEWLINE_BYTE, &mut vec_bytes)?;

    let first_line: String = get_string_utf8(vec_bytes.trim(), 0, &paths.input);

    Ok(first_line)
}

/// Open a csv input file (paths.input).
///
/// Convert all lines of csv file to to_utf8.
///
/// Replace duplicate field names from first row with indexed names.
///
/// Write the result to a temporary file (paths.output).
pub fn format_input_csv_file(args: &Arguments, paths: &Paths) -> MyResult<()> {

    let file_input: File = OpenOptions::new()
        .read(true)
        .write(false)
        .open(&paths.input)?;

    let file_output: File = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true) // replace the file
        .open(&paths.output)?;

    let buffer_input = BufReader::new(file_input);
    let mut buffer_output = BufWriter::new(file_output);

    buffer_input
        .split(NEWLINE_BYTE)
        .map_while(|result_vec_bytes| {
            match result_vec_bytes {
                Ok(vec_bytes) => Some(vec_bytes),
                Err(why) => {
                    eprintln!("fn format_input_csv_file()");
                    eprintln!("Error: {why}");
                    process::exit(1)
                }
            }
        })
        .enumerate()
        .map(|(line_number, vec_bytes)| {
            (line_number, get_string_utf8(&vec_bytes, line_number + 1, &paths.input))
        })
        .try_for_each(|(line_number, line)| -> MyResult<()> {
            if line_number == 0 {
                // The first row (line_number == 0) has column names.
                let old = "UA Detentor Crédito";
                let new = "UA Detentor do Crédito";
                let line = line.replace(old, new);
                let header = get_fields_without_duplication(&line, args);
                writeln!(buffer_output, "{header}")?;
            } else {
                writeln!(buffer_output, "{line}")?;
            };

            Ok(())
        })?;

    buffer_output.flush()?;

    Ok(())
}

/// Get fields without duplication.
///
/// Add indexes on duplicate fields (column names).
pub fn get_fields_without_duplication(line: &str, args: &Arguments) -> String {

    let cols: Vec<String> = parse_line(line, args.delimiter, args.verbose);
    let frequency: BTreeMap<&str, u32> = get_frequency(args, &cols);
    let mut count = HashMap::new();
    let mut fields_without_duplication: Vec<String> = Vec::new();

    for col in &cols {
        let new_col_name = if frequency[col.as_str()] > 1 {
            *count.entry(col).or_insert(0) += 1;
            if count[col] > 1 {
                format!("{col} [{}]", count[col])
            } else {
                col.to_string()
            }
        } else {
            col.to_string()
        };

        let column_with_quotes = if new_col_name.contains(&args.delimiter.to_string()) {
            format!("{new_col_name:#?}")
        } else {
            new_col_name
        };

        fields_without_duplication.push(column_with_quotes);
    }

    let fields = fields_without_duplication.join(&args.delimiter.to_string());

    if args.verbose {
        println!("These fields are the column names:");
        println!("fields_without_duplication: {fields_without_duplication:#?}\n");
    }

    fields
}

/**
 Parse line with delimiter and double-quotes
    ```
    use perdcomp_csv_to_xlsx::parse_line;

    let line: &str = r#"a, b, "foo \nbar",def , the other"#;
    let delimiter: char = ',';
    let verbose = true;

    let cols: Vec<String> = parse_line(&line, delimiter, verbose);

    println!("line: {line}");
    assert_eq!(
        cols,
        vec!["a", "b", "\"foo \\nbar\"", "def", "the other"]
    );

    ```
*/
pub fn parse_line(line: &str, delimiter: char, verbose: bool) -> Vec<String> {

    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(false) // one line
        .flexible(false)
        .trim(csv::Trim::All)
        .delimiter(delimiter as u8)
        .from_reader(line.as_bytes());

    let colunas: Vec<String> = reader
        .records()
        .map_while(Result::ok)
        .flat_map(|record| {
            record
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .collect();

    if verbose {
        println!("colunas: {colunas:#?}\n");
    }

    colunas
}

/// Get word frequency
pub fn get_frequency<'a>(args: &Arguments, cols: &'a [String]) -> BTreeMap<&'a str, u32> {

    // Ordenado pelo nome, em caso de mesma frequência.
    let mut frequency: BTreeMap<&str, u32> = BTreeMap::new();

    for col in cols {
        *frequency.entry(col).or_insert(0) += 1;
    }

    if args.verbose {
        println!("frequency: {frequency:#?}\n");
    }

    frequency
}

/// Converts a slice of bytes to a String.
///
/// Consider the case of files with differently encoded lines!
///
/// That is, one line in UTF-8 and another line in WINDOWS_1252.
pub fn get_string_utf8<T>(
    slice_bytes: &[u8],
    line_number: usize,
    path: T,
) -> String
where
    T: std::fmt::Debug
{
    // from_utf8() checks to ensure that the bytes are valid UTF-8
    let line_utf8: String = match str::from_utf8(slice_bytes) {
        Ok(str) => str.to_string(),
        Err(error1) => {
            let mut data = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(slice_bytes);
            let mut buffer = String::new();
            let _number_of_bytes = match data.read_to_string(&mut buffer) {
                Ok(num) => num,
                Err(error2) => {
                    eprintln!("Problem reading data from file in buffer!");
                    eprintln!("File: {path:?}");
                    eprintln!("Line nº {line_number}");
                    eprintln!("Used encoding type: WINDOWS_1252.");
                    eprintln!("Try another encoding type!");
                    panic!(
                        "Failed to convert data from WINDOWS_1252 to UTF-8!:
                        {error1}\n{error2}\n",
                    );
                },
            };
            //println!("read number_of_bytes: {_number_of_bytes}; buffer: {buffer}");
            buffer
        }
    };
    // line_utf8.trim_end_matches('\r').to_string()
    line_utf8
}

pub fn read_csv<P>(args: &Arguments, path: P) -> MyResult<Vec<PerDcomp>>
where
    P: AsRef<Path>,
{
    let mut reader = ReaderBuilder::new()
        .quoting(true)
        .double_quote(true)
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(false)
        .delimiter(args.delimiter as u8)
        .from_path(path)?; // utf8

    let perdcomps: Vec<PerDcomp> = reader
        .deserialize()
        //.map_while(Result::ok)
        .map_while(|result_perdcomp| {
            match result_perdcomp {
                Ok(perdcomp) => Some(perdcomp),
                Err(why) => {
                    eprintln!("fn read_csv()");
                    eprintln!("Error: Failed to deserialize CSV file");
                    eprintln!("Error: {why}");
                    process::exit(1)
                }
            }
        })
        .collect();

    Ok(perdcomps)
}

/// Rename a file to a new name,
/// replacing the original file if new_path already exists.
pub fn rename_file(old_path: &PathBuf, new_name: &str) -> MyResult<()> {
    let mut new_path = old_path.clone();
    new_path.set_file_name(new_name);
    eprintln!("Keep temporary CSV file: {new_path:?}");
    fs::rename(old_path, new_path)?;
    Ok(())
}
