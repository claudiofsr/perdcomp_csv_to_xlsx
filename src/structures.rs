use crate::REGEX_TRIMESTRE_ANO;

use chrono::NaiveDate;
use rust_xlsxwriter::XlsxSerialize;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, XlsxSerialize)]
#[xlsx(table = Table::new())]
//#[xlsx(header_format = Format::new().set_bold())]
#[serde(rename_all = "PascalCase")]
pub struct PerDcomp {
    #[serde(rename = "PER/DCOMP")] // Coluna Repetida
    #[xlsx(value_format = Format::new().set_bold().set_align(FormatAlign::Center))]
    pub per_dcomp: Option<String>,

    #[serde(rename = "CNPJ/CPF Declarante/Sucessora")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub cnpj_declarante: Option<String>,

    #[serde(rename = "Tipo Crédito")]
    pub tipo_do_credito: Option<String>,

    #[serde(rename = "Valor Total Crédito", deserialize_with = "string_as_f64")]
    #[xlsx(value_format = Format::new().set_num_format("#,##0.00"))]
    pub valor_total_do_credito: f64,

    #[serde(
        rename = "Valor Crédito Data Transmissão",
        deserialize_with = "string_as_f64"
    )]
    #[xlsx(value_format = Format::new().set_num_format("#,##0.00"))]
    pub valor_do_credito_na_data_de_transmissao: f64,

    #[serde(
        rename = "Valor Total Débitos/Valor Pedido Rest/Ress.",
        deserialize_with = "string_as_f64"
    )]
    #[xlsx(value_format = Format::new().set_num_format("#,##0.00"))]
    #[xlsx(
        //column_width = 10.0,
        //column_width_pixels = 20,
        value_format = Format::new()
            .set_bold()
            .set_num_format("#,##0.00")
            //.set_font_color(Color::Blue)
            .set_align(FormatAlign::Right)
    )]
    pub valor_do_per: f64,

    #[serde(default)]
    #[serde(rename = "Data Transmissão", with = "option_date")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub data_da_transmissao: Option<NaiveDate>,

    #[serde(rename = "Demonstra Crédito")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub demonstra_credito: Option<String>,

    #[serde(rename = "Pendente Atuação")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub pendente_atuacao: Option<String>,

    #[serde(rename = "Tipo Documento")]
    pub tipo_do_documento: Option<String>,

    #[serde(rename = "Nome Empresarial/Nome")]
    pub nome_empresarial: Option<String>,

    #[serde(rename = "UA Declarante/Sucessora")]
    pub ua_declarante: Option<String>,

    #[serde(rename = "Detentor Crédito")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub cnpj_detentor_do_credito: Option<String>,

    #[serde(rename = "Período Apuração Crédito")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub trimestre_de_apuracao: Option<String>,

    #[serde(rename = "Ano")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub ano: Option<u32>,

    #[serde(rename = "Período Apuração Pagamento")]
    pub pa_pagamento: Option<String>,

    #[serde(default)]
    #[serde(rename = "Data 1ª DCOMP Ativa", with = "option_date")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
    pub data_dcomp_ativa: Option<NaiveDate>,

    #[serde(rename = "PER/DCOMP Ativo com Demonstrativo de Crédito")]
    #[xlsx(value_format = Format::new().set_align(FormatAlign::Center))]
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
