mod args;
mod excel;
mod regex;
mod structures;

pub use args::Arguments;
pub use excel::write_xlsx;
pub use regex::*;
pub use structures::PerDcomp;

use claudiofsr_lib::BytesExtension;
use csv::ReaderBuilder;
use encoding_rs::WINDOWS_1252;

use std::{
    collections::{BTreeMap, HashMap},
    convert::AsRef,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    str,
    sync::Arc,
};

pub type MyError = Box<dyn std::error::Error>;
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

    let _number_of_bytes: usize = buffer_input.read_until(NEWLINE_BYTE, &mut vec_bytes)?;

    let first_line: String = get_string_utf8(vec_bytes.trim(), 0, &paths.input)?;

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

    let mut buffer_output = BufWriter::with_capacity(1024 * 1024, file_output);

    BufReader::new(file_input)
        .split(NEWLINE_BYTE)
        .enumerate()
        .map(|(i, res)| {
            let bytes = res?;
            let line = get_string_utf8(&bytes, i + 1, &paths.input)?;

            if i == 0 {
                Ok(get_fields_without_duplication(&line, args))
            } else {
                Ok(line)
            }
        })
        .collect::<MyResult<Vec<String>>>()? // Interrompe no primeiro erro
        .into_iter()
        .try_for_each(|line| writeln!(buffer_output, "{}", line))?;

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
        let length = fields_without_duplication.len();
        println!("These {length} fields are the column names:");
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

/**
Converts a slice of bytes to a String, attempting to handle different encodings.

It first tries to decode the bytes as UTF-8. If that fails, it attempts to decode
them using WINDOWS_1252 encoding. If both fail, it returns an error.

### Arguments

* `slice_bytes` - A slice of bytes to convert to a String.
* `line_number` - The line number where these bytes were read from (for error reporting).
* `path` - The path to the file from which these bytes were read (for error reporting).

### Returns

A `Result` containing the decoded String if successful, or an error if decoding fails.

```rust
use std::error::Error;
use perdcomp_csv_to_xlsx::get_string_utf8;

fn main() -> Result<(), Box<dyn Error>> {
    let bytes: &[u8] = "café".as_bytes();
    let path_buf = std::path::PathBuf::from("test.txt");
    // Use the ? operator to propagate the error
    let result: String = get_string_utf8(bytes, 1, &path_buf)?;

    assert_eq!(result, "café");
    Ok(())
}
```
*/
pub fn get_string_utf8(slice_bytes: &[u8], line_number: usize, path: &Path) -> MyResult<String> {
    // Attempt to decode as UTF-8 first
    if let Ok(s) = std::str::from_utf8(slice_bytes) {
        return Ok(s.to_string());
    }

    // If UTF-8 decoding fails, attempt WINDOWS_1252 decoding
    let (res, _, has_errors) = WINDOWS_1252.decode(slice_bytes);

    // If WINDOWS_1252 decoding also fails, return a detailed error
    if has_errors {
        return Err(format!(
            "UTF-8 and WINDOWS_1252 decoding failed!\n\
                Failed to decode line {line_number} from file {path:?}"
        )
        .into());
    }

    Ok(res.into_owned())
}

/**
Reads a CSV file from the given path and deserializes its contents into a vector of `PerDcomp` structs.

### Arguments

* `args` - A struct containing configuration options, such as the delimiter.
* `path` - The path to the CSV file.

### Returns

A `MyResult` containing a vector of `PerDcomp` structs if successful, or a `MyError` if an error occurred.
*/
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
        .from_path(path.as_ref())?;

    reader
        .deserialize()
        .map(|result_perdcomp: Result<PerDcomp, csv::Error>| {
            let mut per_comp = result_perdcomp?;
            per_comp.get_year();
            Ok(per_comp)
        })
        .collect()
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

#[cfg(test)]
mod tests_get_string_utf8 {
    use super::*;

    #[test]
    fn test_get_string_utf8_valid_utf8() {
        let bytes = "hello".as_bytes();
        let result = get_string_utf8(bytes, 1, &PathBuf::from("test.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_get_string_utf8_windows_1252() {
        // "café" encoded in WINDOWS_1252
        let bytes: [u8; 5] = [99, 97, 102, 233, 0];
        let result = get_string_utf8(&bytes, 1, &PathBuf::from("test.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "café\0");
    }

    #[test]
    fn test_get_string_utf8_invalid_encoding() {
        // Invalid UTF-8 and likely invalid WINDOWS_1252
        let bytes: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
        let result = get_string_utf8(&bytes, 1, &PathBuf::from("test.txt"));
        assert!(result.is_ok());
        // Depending on the exact implementation of WINDOWS_1252 decoder, the result might vary.
        // The important thing is that it doesn't panic.  We check that it returns something,
        // indicating a "best effort" decoding.
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_string_utf8_empty_slice() {
        let bytes: [u8; 0] = [];
        let result = get_string_utf8(&bytes, 1, &PathBuf::from("test.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}

#[cfg(test)]
mod test_my_perdcomp {
    use super::*;
    use rust_xlsxwriter::XlsxSerialize;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, Serialize, Deserialize, XlsxSerialize)]
    #[xlsx(table = Table::new())]
    //#[xlsx(header_format = Format::new().set_font_size(12.0))]
    #[serde(rename_all = "PascalCase")]
    pub struct MyPerDcomp {
        #[serde(rename = "PER/DCOMP")] // Coluna Repetida
        #[xlsx(value_format = Format::new().set_bold().set_align(FormatAlign::Center))]
        pub per_dcomp: Option<String>,

        #[serde(rename = "CNPJ/CPF Declarante/Sucessora")]
        #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
        pub cnpj_declarante: Option<String>,

        #[serde(rename = "Tipo Crédito")]
        pub tipo_do_credito: Option<String>,

        #[serde(rename = "Período Apuração Crédito")]
        #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
        pub trimestre_de_apuracao: Option<String>,

        #[serde(rename = "Ano")]
        #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
        pub ano: Option<u32>,
    }

    impl MyPerDcomp {
        pub fn get_year(&mut self) {
            // trimestre_de_apuracao = "3º TRIMESTRE 2021"
            if let Some(trimestre) = self.trimestre_de_apuracao.as_ref()
                && let Some(captures) = REGEX_TRIMESTRE_ANO.captures(trimestre)
            {
                let trim: Option<String> = captures.get(1).map(|s| s.as_str().to_string());
                let year: Option<u32> = captures.get(2).and_then(|s| s.as_str().parse().ok());

                self.trimestre_de_apuracao = trim;
                self.ano = year;
            }
        }
    }

    /// cargo test -- --show-output read_csv
    #[test]
    fn read_csv() -> MyResult<()> {
        let data_csv = r#"
PER/DCOMP,foo,Período Apuração Crédito,"Tipo Crédito",Valor Total Crédito
112,"4","1º TRIMESTRE de 2021",78,"1,0"
321,"5","2º TRIMESTRE de 2021",89,"23.543,34"
555,"6","1º TRIMESTRE de 2020",72,"88,1"
"#;

        let mut reader = ReaderBuilder::new()
            .quoting(true)
            .double_quote(true)
            .has_headers(true)
            .trim(csv::Trim::All)
            .flexible(false)
            .delimiter(b',')
            .from_reader(data_csv.as_bytes());

        let result: MyResult<Vec<MyPerDcomp>> = reader
            .deserialize()
            .map(|result_perdcomp: Result<MyPerDcomp, csv::Error>| {
                let mut per_comp = result_perdcomp?;
                per_comp.get_year();
                Ok(per_comp)
            })
            .collect();

        let perdcomps = result?;

        dbg!(&perdcomps);

        assert_eq!(
            perdcomps.get(1).and_then(|p| p.per_dcomp.clone()),
            Some("321".to_string())
        );
        assert_eq!(perdcomps.get(2).and_then(|p| p.ano), Some(2020));
        Ok(())
    }
}
