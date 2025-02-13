use serde::{de::Error, Deserialize, Deserializer, Serialize};

//use serde_json;
//use serde_with::{serde_as, OneOrMany, Map, DisplayFromStr};

use chrono::{Datelike, NaiveDate};

use crate::{excel::InfoExtension, GetFieldNames, REGEX_DDMMYYYY, REGEX_TRIMESTRE_ANO};
use rust_xlsxwriter::serialize_chrono_naive_to_excel;

const FORMAT_DDMMYYYY: &str = "%-d/%-m/%Y"; // %Y: year, zero-padded to 4 digits.
const FORMAT_DDMMYY: &str = "%-d/%-m/%y"; // %y: year, zero-padded to 2 digits.

//#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PerDcomp {
    #[serde(rename = "PER/DCOMP")] // Coluna Repetida
    pub per_dcomp: Option<String>,

    #[serde(rename = "CNPJ/CPF Declarante/Sucessora")]
    pub cnpj_declarante: Option<String>,

    #[serde(rename = "Tipo Crédito")]
    pub tipo_do_credito: Option<String>,

    #[serde(rename = "Valor Total Crédito", deserialize_with = "string_as_f64")]
    pub valor_total_do_credito: f64,

    #[serde(
        rename = "Valor Crédito Data Transmissão",
        deserialize_with = "string_as_f64"
    )]
    pub valor_do_credito_na_data_de_transmissao: f64,

    #[serde(
        rename = "Valor Total Débitos/Valor Pedido Rest/Ress.",
        deserialize_with = "string_as_f64"
    )]
    pub valor_do_per: f64,

    //#[serde(rename = "Data da Transmissão", with = "my_date_format")]
    #[serde(
        rename = "Data Transmissão",
        deserialize_with = "string_as_date",
        serialize_with = "serialize_chrono_naive_to_excel"
    )]
    pub data_da_transmissao: NaiveDate,

    #[serde(rename = "Demonstra Crédito")]
    pub demonstra_credito: Option<String>,

    #[serde(rename = "Pendente Atuação")]
    pub pendente_atuacao: Option<String>,

    #[serde(rename = "Tipo Documento")]
    pub tipo_do_documento: Option<String>,

    #[serde(rename = "Nome Empresarial/Nome")]
    pub nome_empresarial: Option<String>,

    #[serde(rename = "UA Declarante/Sucessora")]
    pub ua_declarante: Option<String>,

    #[serde(rename = "Detentor Crédito")]
    pub cnpj_detentor_do_credito: Option<String>,

    #[serde(rename = "Período Apuração Crédito")]
    pub trimestre_de_apuracao: Option<String>,

    #[serde(rename = "Ano")]
    pub ano: Option<u32>,

    #[serde(rename = "Período Apuração Pagamento")]
    pub pa_pagamento: Option<String>,

    #[serde(default)]
    #[serde(rename = "Data 1ª DCOMP Ativa", with = "option_date")]
    pub data_dcomp_ativa: Option<NaiveDate>,

    #[serde(rename = "PER/DCOMP Ativo com Demonstrativo de Crédito")]
    pub per_ativo_com_credito: Option<String>,

    #[serde(rename = "Processo Atribuído PER/DCOMP")]
    pub num_processo_atribuido_ao_perdcomp: Option<String>,

    #[serde(rename = "Processo Administrativo Anterior")]
    pub num_processo_administrativo_anterior: Option<String>,

    #[serde(rename = "Processo Judicial")]
    pub processo_judicial: Option<String>,

    #[serde(rename = "Origem Discussão Judicial")]
    pub origem_judicial: Option<String>,

    #[serde(rename = "Situação")]
    pub situacao: Option<String>,

    #[serde(rename = "Motivo")]
    pub motivo: Option<String>,
}

impl PerDcomp {
    /**
    Get year (last 4 chars)
    ```
        use perdcomp_csv_to_xlsx::PerDcomp;

        let mut per_comp = PerDcomp::default();

        let trim_a = "3º TRIMESTRE de 2021".to_string();
        per_comp.trimestre_de_apuracao = Some(trim_a);
        per_comp.get_year();

        assert_eq!(per_comp.trimestre_de_apuracao, Some("3º TRIMESTRE".to_string()));
        assert_eq!(per_comp.ano, Some(2021));

        let trim_b = "4º Trimestre 2024".to_string();
        per_comp.trimestre_de_apuracao = Some(trim_b);
        per_comp.get_year();

        assert_eq!(per_comp.trimestre_de_apuracao, Some("4º Trimestre".to_string()));
        assert_eq!(per_comp.ano, Some(2024));

        let trim_c = "4º trimestre/2023".to_string();
        per_comp.trimestre_de_apuracao = Some(trim_c);
        per_comp.get_year();

        assert_eq!(per_comp.trimestre_de_apuracao, Some("4º trimestre".to_string()));
        assert_eq!(per_comp.ano, Some(2023));
    ```
    */
    pub fn get_year(&mut self) {
        // trimestre_de_apuracao = "3º TRIMESTRE 2021"
        if let Some(trimestre) = self.trimestre_de_apuracao.as_ref() {
            if let Some(captures) = REGEX_TRIMESTRE_ANO.captures(trimestre) {
                let trim: Option<String> = captures.get(1).map(|s| s.as_str().to_string());
                let year: Option<u32> = captures.get(2).and_then(|s| s.as_str().parse().ok());

                self.trimestre_de_apuracao = trim;
                self.ano = year;
            }
        }
    }
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for PerDcomp {}
impl GetFieldNames for PerDcomp {}

/// Deserializes a string into an `f64`.
///
/// This function attempts to parse a string into an `f64`, handling common
/// European number formatting where `.` is used as a thousands separator and
/// `,` is used as a decimal separator.  It removes all `.` characters and
/// replaces the last `,` with a `.` before parsing.
///
/// # Arguments
///
/// * `deserializer`: A Serde deserializer.
///
/// # Returns
///
/// A `Result` containing the parsed `f64` if successful, or a `serde::de::Error`
/// if deserialization or parsing fails.
pub fn string_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).and_then(|string| {
        // Trim whitespace and perform the conversion:
        // 1.234.567,89 => 1234567.89
        let s = string.trim().replace('.', "").replace(',', ".");
        s.parse::<f64>().map_err(|e| {
            // Capture the parse error 'e'
            // Include the original error and the string in the error message.
            Error::custom(format!(
                "\nfn string_as_f64()\nFailed to parse f64 from string \"{}\": {}",
                string, e
            ))
        })
    })
}

/// Deserializes a string into a `NaiveDate`.
///
/// It attempts to parse the string using the following formats:
/// - dd/mm/yyyy
/// - dd/mm/yy
///
/// The function first uses a regular expression to validate the basic format
/// and then attempts to parse the string using `NaiveDate::parse_from_str`.
///
/// Returns a `NaiveDate` if parsing is successful, or a `serde::de::Error` if not.
pub fn string_as_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the string.
    let string = String::deserialize(deserializer)?;

    // Validate the date format using a regular expression.
    let captures = REGEX_DDMMYYYY.captures(&string).ok_or_else(|| {
        Error::custom(format!(
            "\nfn string_as_date()\nInvalid date format: '{}'. Expected dd/mm/yy or dd/mm/yyyy.",
            string
        ))
    })?;

    // Extract day, month, and year from the regex captures.
    let d = captures.get(1).map_or("", |m| m.as_str());
    let m = captures.get(2).map_or("", |m| m.as_str());
    let y = captures.get(3).map_or("", |m| m.as_str());

    // Format the date string for parsing.
    let formatted_date = format!("{}/{}/{}", d, m, y);

    // Attempt to parse the date using both formats.
    let result_date1 = NaiveDate::parse_from_str(&formatted_date, FORMAT_DDMMYYYY);
    let result_date2 = NaiveDate::parse_from_str(&formatted_date, FORMAT_DDMMYY);

    // Match the parsing results and return the valid date, prioritizing 4-digit years.
    match (result_date1, result_date2) {
        (Ok(date1), _) if date1.year() >= 1000 => Ok(date1), // Retain only the year consisting of 4 digits
        (_, Ok(date2)) if date2.year() >= 1000 => Ok(date2), // Retain only the year consisting of 4 digits
        _ => Err(Error::custom(format!(
            "\nfn string_as_date()\nInvalid date: '{}' (formatted: '{}').  Expected format dd/mm/yy or dd/mm/yyyy.",
            string, formatted_date
        ))),
    }
}

// Font: https://serde.rs/custom-date-format.html
#[allow(dead_code)]
mod my_date_format {
    use chrono::{Datelike, NaiveDate};
    use serde::{self, de::Error, Deserialize, Deserializer, Serializer};

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        //const FORMAT: &str = "%-d/%-m/%Y";
        //let string = format!("{}", date.format(FORMAT));

        let year: i32 = date.year();
        let month: u32 = date.month();
        let day: u32 = date.day();

        let string = format!("{day:02}/{month:02}/{year:04}");
        serializer.serialize_str(&string)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";

        let string = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&string, FORMAT).map_err({
            //eprintln!("NaiveDate Error: {string}");
            Error::custom
        })?;
        Ok(dt)
    }
}

// Font: https://stackoverflow.com/questions/44301748/how-can-i-deserialize-an-optional-field-with-custom-functions-using-serde
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=d4e3ff1407b518c7848a4ef31b4cf05c
// https://github.com/serde-rs/serde/issues/1425
mod option_date {
    use chrono::NaiveDate;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    // Define the expected date format.  Using a constant improves readability and maintainability.
    const FORMAT: &str = "%-d/%-m/%Y";
    //const FORMAT: &str = "%-d/%-m/%YT%H:%M:%S%z";

    /// Serializes an `Option<NaiveDate>` to a string representation.
    ///
    /// If the date is `Some`, it's formatted according to `FORMAT` and serialized as a string.
    /// If the date is `None`, it's serialized as a `None` value (e.g., `null` in JSON).
    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_str(&d.format("%d/%m/%Y").to_string()),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an `Option<NaiveDate>` from a string.
    ///
    /// It attempts to parse the string according to `FORMAT`.  It handles variations in input
    /// by replacing hyphens with slashes and splitting on whitespace or 'T' to isolate the date part.
    ///
    /// Returns `Some(NaiveDate)` if parsing is successful, `None` otherwise.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input into an Option<String>.  This handles the case where the input is null/None.
        let optional_string: Option<String> = Option::deserialize(deserializer)?;

        match optional_string {
            Some(string) => {
                // Preprocess the string to handle different separators.
                // string: "17-2-2014 16:32:52.34" or "17/02/2014T16:32:52.34" or "17/02/2014"
                let normalized_string = string.replace('-', "/");

                // Split the string to isolate the date part.
                let parts: Vec<&str> = normalized_string
                    .trim()
                    .split(|c: char| c.is_ascii_whitespace() || c == 'T')
                    .collect();

                // Get the first part, which should be the date.
                let date_str = parts.first().map_or("", |&s| s);

                // Attempt to parse the date string.
                NaiveDate::parse_from_str(date_str, FORMAT)
                    .map(Some) // Convert the successful parse to Some(NaiveDate)
                    .map_err(|error| {
                        // Include the original error message
                        let msg = format!(
                            "\nmod option_date\ndate: {string:?}\nFailed to parse date: {error}\n"
                        );
                        Error::custom(msg)
                    })
            }
            None => Ok(None), // If the input was None, return None.
        }
    }
}

#[cfg(test)]
mod functions {
    use super::*;
    use crate::MyResult;
    use serde_json::json;

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    #[test]
    /// cargo test -- --show-output get_headers_from_per_dcomp
    fn get_headers_from_per_dcomp() -> MyResult<()> {
        let headers = PerDcomp::get_field_names();
        println!("headers: {headers:#?}");
        assert_eq!(headers[0], "PER/DCOMP");
        //assert_eq!(headers[59], "PER/DCOMP [2]");
        Ok(())
    }

    #[test]
    /// cargo test -- --show-output deserialize_date_and_vale
    fn deserialize_date_and_vale() -> MyResult<()> {
        // Some struct
        #[derive(Deserialize, Debug, PartialEq)]
        struct TestStruct {
            #[serde(deserialize_with = "string_as_date")]
            date: NaiveDate,
            #[serde(deserialize_with = "string_as_f64")]
            value: f64,
        }

        let dates = [
            " 25-05-2023 09:13:20 ",
            "25/05/2023 12:39:04",
            "25/05/23 12:39:04",
            "25/05/23 12:39",
            "25/05/2023",
        ];

        let expected = TestStruct {
            date: NaiveDate::from_ymd_opt(2023, 5, 25).unwrap(),
            value: 1.234,
        };

        println!("expected: {expected:#?}");

        for date in dates {
            let json = json!(
                {"date": date, "value": "1,2340"}
            );
            let result: TestStruct = serde_json::from_value(json).unwrap();
            assert_eq!(result, expected);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_string_as_f64 {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(deserialize_with = "string_as_f64")]
        value: f64,
    }

    #[test]
    fn test_valid_string_with_european_format() {
        let json = r#"{"value": "1.234.567,89"}"#;
        let expected = TestStruct { value: 1234567.89 };
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_valid_string_with_no_separators() {
        let json = r#"{"value": "1234567"}"#;
        let expected = TestStruct { value: 1234567.0 };
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_string() {
        let json = r#"{"value": ""}"#;
        let result = serde_json::from_str::<TestStruct>(json);
        assert!(result.is_err()); // Expect an error because an empty string cannot be parsed to f64.
    }

    #[test]
    fn test_invalid_string() {
        let json = r#"{"value": "abc"}"#;
        let result = serde_json::from_str::<TestStruct>(json);
        assert!(result.is_err()); // Expect an error because "abc" cannot be parsed to f64.
    }

    #[test]
    fn test_string_with_leading_and_trailing_whitespace() {
        let json = r#"{"value": "  1.234,56  "}"#;
        let expected = TestStruct { value: 1234.56 };
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod tests_string_as_date {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "string_as_date")]
        date: NaiveDate,
    }

    #[test]
    fn test_valid_date_ddmmyyyy() {
        let json = r#"{"date": "20/01/2024"}"#;
        let expected_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let expected_struct = TestStruct {
            date: expected_date,
        };
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert_eq!(actual_struct.unwrap(), expected_struct);
    }

    #[test]
    fn test_valid_date_ddmmyy() {
        let json = r#"{"date": "20-07-24"}"#;
        let expected_date = NaiveDate::from_ymd_opt(2024, 7, 20).unwrap();
        let expected_struct = TestStruct {
            date: expected_date,
        };
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert_eq!(actual_struct.unwrap(), expected_struct);
    }

    #[test]
    fn test_invalid_date_format() {
        let json = r#"{"date": "20-13-2024"}"#;
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(actual_struct.is_err());
    }

    #[test]
    fn test_invalid_date_value() {
        let json = r#"{"date": "32/01/2024"}"#;
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(actual_struct.is_err());
    }

    #[test]
    fn test_incomplete_date() {
        let json = r#"{"date": "20/01/"}"#;
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(actual_struct.is_err());
    }

    #[test]
    fn test_empty_date() {
        let json = r#"{"date": ""}"#;
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(actual_struct.is_err());
    }

    #[test]
    fn test_invalid_characters() {
        let json = r#"{"date": "20/AB/2024"}"#;
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(actual_struct.is_err());
    }

    #[test]
    fn test_short_year_out_of_range() {
        let json = r#"{"date": "20/01/00"}"#; // Year 2000
        let expected_date = NaiveDate::from_ymd_opt(2000, 1, 20).unwrap();
        let expected_struct = TestStruct {
            date: expected_date,
        };
        let actual_struct: Result<TestStruct, _> = serde_json::from_str(json);
        assert_eq!(actual_struct.unwrap(), expected_struct);
    }
}

#[cfg(test)]
mod option_date_tests {
    use super::option_date;
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(with = "option_date")]
        date: Option<NaiveDate>,
    }

    #[test]
    fn test_serialize_some_date() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let test_struct = TestStruct { date: Some(date) };
        let expected_json = r#"{"date":"02/01/2024"}"#;
        let actual_json = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(actual_json, expected_json);
    }

    #[test]
    fn test_serialize_none_date() {
        let test_struct = TestStruct { date: None };
        let expected_json = r#"{"date":null}"#;
        let actual_json = serde_json::to_string(&test_struct).unwrap();
        assert_eq!(actual_json, expected_json);
    }

    #[test]
    fn test_deserialize_some_date() {
        let json = r#"{"date":"20/01/2024"}"#;
        let expected_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let expected_struct = TestStruct {
            date: Some(expected_date),
        };
        let actual_struct: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(actual_struct, expected_struct);
    }

    #[test]
    fn test_deserialize_some_date_with_hyphens() {
        let json = r#"{"date":"20-1-2024"}"#;
        let expected_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let expected_struct = TestStruct {
            date: Some(expected_date),
        };
        let actual_struct: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(actual_struct, expected_struct);
    }

    #[test]
    fn test_deserialize_none_date() {
        let json = r#"{"date":null}"#;
        let expected_struct = TestStruct { date: None };
        let actual_struct: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(actual_struct, expected_struct);
    }

    #[test]
    fn test_deserialize_some_date_with_time() {
        let json = r#"{"date":"20/1/2024 12:30:00"}"#;
        let expected_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let expected_struct = TestStruct {
            date: Some(expected_date),
        };
        let actual_struct: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(actual_struct, expected_struct);
    }

    #[test]
    fn test_deserialize_some_date_with_time_t_separator() {
        let json = r#"{"date":"20/1/2024T12:30:00"}"#;
        let expected_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
        let expected_struct = TestStruct {
            date: Some(expected_date),
        };
        let actual_struct: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(actual_struct, expected_struct);
    }

    #[test]
    fn test_deserialize_invalid_date() {
        let json = r#"{"date":"invalid date"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
