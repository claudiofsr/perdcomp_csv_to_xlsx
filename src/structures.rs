use itertools::Itertools;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    de::{
        Error,
        Visitor,
        MapAccess,
        SeqAccess,
        DeserializeSeed,
        value::SeqAccessDeserializer,
    },
};

//use serde_json;
//use serde_with::{serde_as, OneOrMany, Map, DisplayFromStr};
use std::{
    fmt,
    collections::{
        //HashMap,
        BTreeMap,
    },
    marker::PhantomData,
    process,
};

use chrono::{
    Datelike,
    NaiveDate,
};

use struct_iterable::Iterable;
use rust_xlsxwriter::serialize_chrono_naive_to_excel;
use crate::{
    excel::InfoExtension,
    GetFieldNames,
    REGEX_DDMMYYYY, 
    REGEX_TRIMESTRE_ANO,
};

const FORMAT_DDMMYYYY: &str = "%-d/%-m/%Y"; // %Y: year, zero-padded to 4 digits.
const FORMAT_DDMMYY:   &str = "%-d/%-m/%y"; // %y: year, zero-padded to 2 digits.

// Examples:
// https://gist.github.com/ripx80/33f80618bf13e3f4964b0d75c62bfd28
// https://brokenco.de/2020/08/03/serde-deserialize-with-string.html
// https://github.com/serde-rs/json/issues/329
// https://serde.rs/custom-date-format.html

//#[serde_as]
#[derive(Debug, Default, Serialize, Deserialize, Iterable)]
pub struct PerDcomp {

    #[serde(rename = "PER/DCOMP")] // Coluna Repetida
    pub per_dcomp: String,

    #[serde(rename = "CNPJ Declarante")]
    pub cnpj_declarante: String,
    #[serde(rename = "Valor Total do Crédito", deserialize_with = "string_as_f64")]
    pub valor_total_do_credito: f64,
    #[serde(rename = "Valor do Crédito na Data de Transmissão", deserialize_with = "string_as_f64")]
    pub valor_do_credito_na_data_de_transmissao: f64,
    #[serde(rename = "Valor Pedido de Ressarcimento", deserialize_with = "string_as_f64")]
    pub valor_do_per: f64,

    //#[serde(rename = "Data da Transmissão", with = "my_date_format")]
    #[serde(rename = "Data da Transmissão", deserialize_with = "string_as_date", serialize_with = "serialize_chrono_naive_to_excel")]
    pub data_da_transmissao: NaiveDate,

    #[serde(rename = "Nome Empresarial")]
    pub nome_empresarial: String,
    #[serde(rename = "UA Declarante")]
    pub ua_declarante: String,
    #[serde(rename = "CNPJ Detentor Crédito")]
    pub cnpj_detentor_do_credito: String,
    #[serde(rename = "UA Detentor do Crédito")]
    pub ua_detentor_do_credito: String,
    #[serde(rename = "Tipo de Declaração")]
    pub tipo_de_declaracao: String,
    #[serde(rename = "Número Processo Atribuído PER/DCOMP")]
    pub num_processo_atribuido_ao_perdcomp: String,
    #[serde(rename = "Número Processo Administrativo Anterior")]
    pub num_processo_administrativo_anterior: String,
    #[serde(rename = "Número Processo Habilitação")]
    pub num_processo_habilitado: String,
    #[serde(rename = "Tipo Documento")]
    pub tipo_do_documento: String,
    #[serde(rename = "Tipo Crédito")]
    pub tipo_do_credito: String,
    #[serde(rename = "Trimestre Apuração")]
    pub trimestre_de_apuracao: Option<String>,
    #[serde(rename = "Ano")]
    pub ano: Option<u32>,
    #[serde(rename = "Situação")]
    pub situacao: String,
    #[serde(rename = "Motivo")]
    pub motivo: String,

    //#[serde(rename = "Data da Situação/Motivo Atual", with = "my_date_format")]
    #[serde(rename = "Data da Situação/Motivo Atual", deserialize_with = "string_as_date", serialize_with = "serialize_chrono_naive_to_excel")]
    pub data_da_situacao: NaiveDate,

    #[serde(rename = "Perfil do Contribuinte")]
    pub perfil_do_contribuinte: String,
    #[serde(rename = "Nº do PER/DCOMP Retificado/Cancelado")]
    pub mum_perdcomp_retificado_ou_cancelado: String,
    #[serde(rename = "Versão do PGD")]
    pub versao_do_pgd: String,
    #[serde(rename = "CNPJ Sucessora")]
    pub cnpj_sucessora: String,
    #[serde(rename = "UA Sucessora")]
    pub ua_sucessora: String,
    #[serde(rename = "Processo Mesmo Crédito Identificado Servidor")]
    pub processo_mesmo_credito: String,
    #[serde(default)]
    #[serde(rename = "Data de Distribuição", with = "option_date")]
    pub data_da_distribuicao: Option<NaiveDate>,

    //#[serde(rename = "PER/DCOMP [2]", skip)]
    #[serde(rename = "PER/DCOMP [2]")]
    pub per_dcomp_apenas_numeros: String,

    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    #[serde(rename = "CPF Responsável Pela Análise")]
    pub cpf_responsavel_pela_analise: String,
    #[serde(default)]
    #[serde(rename = "Dt. Transm. DCOMP Ativa Mais Antiga", with = "option_date")]
    pub data_dcomp_ativa: Option<NaiveDate>,
    #[serde(rename = "Motivos de Interesse Fiscal")]
    pub motivo_de_interesse_fiscal: String,
    #[serde(rename = "Código do Crédito Apurado")]
    pub codigo_do_credito_apurado: String,
    #[serde(rename = "Base Legal")]
    pub base_legal: String,
    #[serde(rename = "Número do RPF")]
    pub num_rpf: String,
    #[serde(rename = "Processos Judiciais / Administrativos (Judicial: Recibo EFD, Registro,  Nº do Processo, Seção Judiciária, Vara, Descrição, Data da Decisão) ou ( Administrativo: Recibo EFD, Registro,  Nº do Processo, Data da Decisão)")]
    pub processos_judiciais: String,

    // Capture additional fields (https://serde.rs/attr-flatten.html)
    // A field of map type can be flattened to hold additional data
    // that is not captured by any other fields of the struct.
    //#[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    //pub extra: Option<BTreeMap<String, String>>,

    //#[serde(rename = "Extra", flatten, with = "additional_fields")]
    //pub extra: String,

    //#[serde(skip_serializing_if = "Option::is_none", deserialize_with = "double_option")]
    //extra: Option<Option<String>>,

    /*
    #[serde(
    default,                                    // <- important for deserialization
    skip_serializing_if = "Option::is_none",    // <- important for serialization
    with = "::serde_with::rust::double_option",
    )]
    pub per_dcomp: Option<Option<String>>,
    */

    // https://stackoverflow.com/questions/64405600/how-can-i-deserialize-json-that-has-duplicate-keys-without-losing-any-of-the-val
    //#[serde(flatten, deserialize_with = "deserialize_duplicate_field_name")] // Coluna Repetida
    //pub per_dcomp: String,

    //#[serde(rename = "PER/DCOMP", flatten, deserialize_with = "dup1")]
    //#[serde(rename = "PER/DCOMP", flatten, deserialize_with = "dup2")]
    //#[serde(rename = "PER/DCOMP", flatten, deserialize_with = "dup3")]
    //pub per_dcomp: String,

    //#[serde(flatten)]
    //pub per_dcomp: BTreeMap<String, String>,

    //#[serde(with = "::serde_with::rust::maps_first_key_wins")]
    //#[serde(rename = "PER/DCOMP", flatten)]
    //pub per_dcomp: BTreeMap<String, String>,

    //#[serde(rename = "PER/DCOMP")]
    //pub per_dcomp: FlattenedVecVisitor::<Vec<String>>,

    //#[serde_as(as = "OneOrMany<_>")]
    //#[serde(default, rename = "PER/DCOMP", flatten)]
    //pub per_dcomp: Vec<String>,

    //#[serde(rename = "PER/DCOMP", deserialize_with = "string_or_seq_string")]
    //pub per_dcomp: Vec<String>,

    //#[serde(rename = "PER/DCOMP", flatten, deserialize_with = "vectorize")]
    //pub per_dcomp: String,

    /*
    // test
    #[serde(rename = "PER/DCOMP [3]")]
    pub per_dcomp_3: String,

    #[serde(rename = "PER/DCOMP [4]")]
    pub per_dcomp_4: String,

    #[serde(rename = "PER/DCOMP [5]")]
    pub per_dcomp_5: String,

    #[serde(rename = "PER/DCOMP [6]")]
    pub per_dcomp_6: String,

    #[serde(rename = "PER/DCOMP [7]")]
    pub per_dcomp_7: String,

    #[serde(rename = "PER/DCOMP [8]")]
    pub per_dcomp_8: String,

    #[serde(rename = "PER/DCOMP [9]")]
    pub per_dcomp_9: String,
    */
}

impl PerDcomp {
    /**
    Get year (last 4 chars)
    ```
        use perdcomp_csv_to_xlsx::PerDcomp;

        let mut per_comp = PerDcomp::default();
        let trim = "3º TRIMESTRE de 2021".to_string();
        per_comp.trimestre_de_apuracao = Some(trim);
        per_comp.get_year();

        assert_eq!(per_comp.trimestre_de_apuracao, Some("3º TRIMESTRE".to_string()));
        assert_eq!(per_comp.ano, Some(2021));
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

// https://serde.rs/impl-deserialize.html
#[allow(dead_code)]
pub fn dup1<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct MyVisitor;

    impl<'d> Visitor<'d> for MyVisitor {
        // The return type of the `deserialize` method.
        type Value = BTreeMap<String, String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'d>,
        {
            let mut hashmap = BTreeMap::new();
            let mut count = 1;

            while let Some((key, value)) = access.next_entry()?
                .filter(|(k, _v): &(String, String)| k == "PER/DCOMP")
                {
                    let key = [key.to_string(), count.to_string()].concat();
                    println!("key: {key} ; value: {value:?}");
                    hashmap.insert(key, value);
                    count += 1;
                }

            println!("hashmap: {hashmap:?}\n");
            Ok(hashmap)
        }
    }

    let result_btree = deserializer.deserialize_map(MyVisitor);

    result_btree.map(|btree| {
        btree.into_values().join(", ")
    })
}

// https://docs.rs/crate/serde/latest/source/src/de/mod.rs
#[allow(dead_code)]
pub fn dup2<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct MyVisitor;

    impl<'de> Visitor<'de> for MyVisitor {
        // The return type of the `deserialize` method.
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "an array of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::new();

            // Decrease the number of reallocations if there are many elements
            if let Some(size_hint) = seq.size_hint() {
                values.reserve(size_hint);
            }

            // Visit each element in the inner array and push it onto
            // the existing vector.
            while let Some(elem) = seq.next_element()? {
                values.push(elem);
            }
            Ok(values)
        }
    }

    let map = deserializer.deserialize_seq(MyVisitor);

    map.map(|h| {
        h.join(", ")
    })
}


//*
/// // A DeserializeSeed implementation that uses stateful deserialization to
/// // append array elements onto the end of an existing vector. The preexisting
/// // state ("seed") in this case is the Vec<T>. The `deserialize` method of
/// // `ExtendVec` will be traversing the inner arrays of the JSON input and
/// // appending each integer into the existing Vec.

#[derive(Debug)]
pub struct ExtendVec<'a, T: 'a>(&'a mut Vec<T>);

impl<'de, T> DeserializeSeed<'de> for ExtendVec<'_, T>
where
    T: Deserialize<'de>,
{
    // The return type of the `deserialize` method. This implementation
    // appends onto an existing vector but does not create any new data
    // structure, so the return type is ().
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Visitor implementation that will walk an inner array of the JSON
        // input.
        struct ExtendVecVisitor<'a, T: 'a>(&'a mut Vec<T>);

        impl<'de, T> Visitor<'de> for ExtendVecVisitor<'_, T>
        where
            T: Deserialize<'de>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of integers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Decrease the number of reallocations if there are many elements
                if let Some(size_hint) = seq.size_hint() {
                    self.0.reserve(size_hint);
                }

                // Visit each element in the inner array and push it onto
                // the existing vector.
                while let Some(elem) = seq.next_element()? {
                    self.0.push(elem);
                }
                Ok(())
            }
        }

        deserializer.deserialize_seq(ExtendVecVisitor(self.0))
    }
}

// Visitor implementation that will walk the outer array of the JSON input.
#[derive(Debug, Deserialize, Serialize)]
pub struct FlattenedVecVisitor<T>(PhantomData<T>);

// https://docs.rs/crate/serde/latest/source/src/de/mod.rs
#[allow(dead_code)]
impl<'de, T> Visitor<'de> for FlattenedVecVisitor<T>
where
    T: Deserialize<'de>,
{
    // This Visitor constructs a single Vec<T> to hold the flattened
    // contents of the inner arrays.
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of arrays")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Create a single Vec to hold the flattened contents.
        let mut vec = Vec::new();

        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendVec(&mut vec))? {
            // Nothing to do; inner array has been appended into `vec`.
        }

        // Return the finished vec.
        Ok(vec)
    }
}
//*/


#[allow(dead_code)]
pub fn dup3<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let visitor = FlattenedVecVisitor(PhantomData);
    let flattened: Vec<String> = deserializer.deserialize_seq(visitor)?;
    Ok(flattened.join(", "))
}



// https://stackoverflow.com/questions/44331037/how-can-i-distinguish-between-a-deserialized-field-that-is-missing-and-one-that
// https://github.com/serde-rs/serde/issues/1042
// https://docs.rs/serde_with/latest/serde_with/rust/double_option/index.html
#[allow(dead_code)]
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where T: Deserialize<'de>,
          D: Deserializer<'de>
{
    Deserialize::deserialize(deserializer).map(Some)
}

// https://stackoverflow.com/questions/51276896/how-do-i-use-serde-to-serialize-a-hashmap-with-structs-as-keys-to-json
#[allow(dead_code)]
pub fn vectorize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let container: BTreeMap<String, String> = serde::Deserialize::deserialize(deserializer)?;
    Ok(container.into_values().join(", "))
}

// https://stackoverflow.com/questions/41151080/deserialize-a-json-string-or-array-of-strings-into-a-vec
#[allow(dead_code)]
fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: Error
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
            where S: SeqAccess<'de>
        {
            Deserialize::deserialize(SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

// duplicate field name
// Font: https://github.com/serde-rs/serde/issues/1725
// https://users.rust-lang.org/t/how-can-i-handle-duplicate-fields-when-specifying-multiple-aliases-using-serde/46426
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=e03f47980362868e2684e6dd2de8ef3d
// https://bgrande.de/blog/custom-deserialization-of-multiple-type-field-from-json-in-rust/

#[allow(dead_code)]
fn deserialize_duplicate_field_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let duplicate: BTreeMap<String, String> = Deserialize::deserialize(deserializer)?;
    let perdcomp = match duplicate.get("PER/DCOMP") {
        Some(s) => s,
        None => "",
    };
    eprintln!("duplicate: {duplicate:?} ; perdcomp: {perdcomp:?}");
    Ok(perdcomp.to_string())
}



pub fn string_as_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|string| {
            // 1.234.567,89 => 1234567.89
            let s = string
                .trim()
                .replace('.', "")
                .replace(',', ".");
            s.parse::<f64>()
                .map_err({
                    //eprintln!("f64 Error: {string} -> {s}");
                    Error::custom
                })
        })
}

#[allow(dead_code)]
pub fn string_as_f64_v2<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let string_a: String = Deserialize::deserialize(deserializer)?;

    // 1.234.567,89 => 1234567.89
    let string_b = string_a
        .trim()
        .replace('.', "")
        .replace(',', ".");

    let result_float = string_b.parse::<f64>();

    let float: f64 = result_float
        .map_err({
            //eprintln!("f64 Error: {string_a} -> {string_b}");
            Error::custom
        })?;

    Ok(float)
}

pub fn string_as_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    //let mut string = " 03-12-2021 09:13:20 ".to_string();
    //remove_whitespace(&mut string);
    //let string_without_whitespace = string.replace(' ', "");

    let date = if let Some(captures) = REGEX_DDMMYYYY.captures(&string) {
        let d: &str = captures.get(1).map_or("", |m| m.as_str());
        let m: &str = captures.get(2).map_or("", |m| m.as_str());
        let y: &str = captures.get(3).map_or("", |m| m.as_str());
        format!("{d}/{m}/{y}")
    } else {
        eprintln!("fn string_as_date()");
        eprintln!("Regex Error: Invalid date format!");
        eprintln!("date: '{string}'");
        eprintln!("Expected format: dd/mm/yy or dd/mm/yyyy");
        process::exit(1)
    };

    let result_date1 = NaiveDate::parse_from_str(&date, FORMAT_DDMMYYYY);
    let result_date2 = NaiveDate::parse_from_str(&date, FORMAT_DDMMYY);

    let naive_date: NaiveDate = match (result_date1, result_date2) {
        (Ok(date1), _) if date1.year() >= 1000 => date1, // retain only the year consisting of 4 digits
        (_, Ok(date2)) if date2.year() >= 1000 => date2, // retain only the year consisting of 4 digits
        _ => {
            eprintln!("fn string_as_date()");
            eprintln!("Error: Invalid date format!");
            eprintln!("date: '{string}' -> '{date}'");
            eprintln!("Expected format: dd/mm/yy or dd/mm/yyyy");
            process::exit(1)
            //Err(Error::custom)
        },
    };

    //println!("string: {string:?}");
    //println!("result_date1: {result_date1:?}");
    //println!("result_date2: {result_date2:?}");
    //println!("naive_date: {naive_date:?}\n");

    Ok(naive_date)
}

// Font: https://serde.rs/custom-date-format.html
#[allow(dead_code)]
mod my_date_format {
    use chrono::{
        Datelike,
        NaiveDate,
    };
    use serde::{
        self,
        de::Error,
        Serializer,
        Deserialize,
        Deserializer,
    };

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
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
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";

        let string = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&string, FORMAT)
            .map_err({
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
    use serde::{
        self,
        de::Error,
        Serializer,
        Deserialize,
        Deserializer,
    };

    const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";

    pub fn serialize<S>(date: &Option<NaiveDate>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *date {
            return s.serialize_str(&d.format("%d/%m/%Y").to_string());
        }
        s.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(
                NaiveDate::parse_from_str(&s, FORMAT)
                .map_err({
                    //eprintln!("Option<NaiveDate> Error: {s:?}");
                    Error::custom
                })?
            ));
        }

        Ok(None)
    }
}

#[allow(dead_code)]
mod additional_fields {
    use std::collections::BTreeMap;
    use serde::{
        self,
        Serializer,
        Deserializer, Serialize,
    };

    pub fn serialize<S>(string: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        string.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: BTreeMap<String, String> = serde::Deserialize::deserialize(deserializer)?;
        //let key_value: String = serde_json::to_string(&map).unwrap_or_default();
        let key_value: String = map
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<String>>()
            .join(", ");
    
        Ok(key_value)
    }
}

#[cfg(test)]
mod functions {
    use super::*;
    use crate::MyResult;

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    #[test]
    /// cargo test -- --show-output get_headers_from_per_dcomp
    fn get_headers_from_per_dcomp() -> MyResult<()> {

        let headers = PerDcomp::get_field_names();
        println!("headers: {headers:#?}");        
        assert_eq!(headers[0], "PER/DCOMP");
        assert_eq!(headers[28], "PER/DCOMP [2]");
        Ok(())
    }
}
