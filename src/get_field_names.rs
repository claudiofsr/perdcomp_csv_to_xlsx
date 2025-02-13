use serde::Deserialize;

/**
    Fonts:

    <https://serde.rs/impl-deserialize.html>
    <https://serde.rs/impl-deserializer.html>

    <https://stackoverflow.com/questions/72737381/get-structure-header-names-with-serde-serialize-preserving-order>

    <https://caolan.github.io/tamawiki/src/tungstenite/handshake/headers.rs.html#17-19>
*/
struct FieldTracingDeserializer<'a> {
    fields: &'a mut Vec<&'static str>,
}

impl<'de> serde::Deserializer<'de> for FieldTracingDeserializer<'_> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        panic!("Only works for structs");
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }

    // https://docs.rs/serde/latest/serde/trait.Deserializer.html#tymethod.deserialize_struct
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // Would be cleaner to return fields through a custom error struct.
        // But also more work.
        self.fields.extend_from_slice(fields);
        //self.deserialize_map(visitor);
        Err(serde::de::Error::custom("success"))
    }
}

/**
Get fields names from structure

<https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
*/
pub trait GetFieldNames {
    /**
    Gets the serialization names for structs
    ```
        use perdcomp_csv_to_xlsx::GetFieldNames;
        use serde::Deserialize;

        #[derive(Deserialize)]
        pub struct Colunas {
            #[serde(rename = "Arquivo Teste")]
            pub arquivo: String,
            #[serde(rename = "CNPJ dos Estabelecimentos")]
            pub cnpj: Option<String>,
            #[serde(rename = "Ano do Período de Apuração")]
            pub year: Option<usize>,
            #[serde(rename = "Mês")]
            pub month: u32,
            #[serde(rename = "Mês")] // duplicate name
            pub month2: u32,
            #[serde(rename = "Dia")]
            pub day: u32,
            pub last: Vec<String>,
        }

        impl GetFieldNames for Colunas {}

        let headers = Colunas::get_field_names();

        assert_eq!(
            headers,
            vec![
                "Arquivo Teste",
                "CNPJ dos Estabelecimentos",
                "Ano do Período de Apuração",
                "Mês",
                "Mês",
                "Dia",
                "last",
            ]
        );
    ```
    */
    fn get_field_names<'de>() -> Vec<&'static str>
    where
        Self: Deserialize<'de>,
    {
        let mut headers: Vec<&'static str> = Vec::new();

        let field_tracing = FieldTracingDeserializer {
            fields: &mut headers,
        };

        Self::deserialize(field_tracing).ok();

        headers
    }
}

#[cfg(test)]
mod functions {
    use super::*;
    use crate::MyResult;

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    #[allow(dead_code, unreachable_patterns)]
    #[test]
    /// cargo test -- --show-output get_headers_from_struct
    fn get_headers_from_struct() -> MyResult<()> {
        #[derive(Deserialize)]
        pub struct Colunas {
            #[serde(rename = "Arquivo Teste")]
            pub arquivo: String,
            #[serde(rename = "CNPJ dos Estabelecimentos")]
            pub cnpj: Option<String>,
            #[serde(rename = "Ano do Período de Apuração")]
            pub year: Option<usize>,
            #[serde(rename = "Mês")]
            pub month: u32,
            #[serde(rename = "Mês")] // duplicate name
            pub month2: u32,
            #[serde(rename = "Dia")]
            pub day: u32,
            pub last: Vec<String>,
        }

        impl GetFieldNames for Colunas {}

        let headers = Colunas::get_field_names();

        println!("headers: {headers:#?}");

        Ok(())
    }

    /*

    #[test]
    /// cargo test -- --show-output get_headers_from_enum
    fn get_headers_from_enum() -> MyResult<()> {

        #[derive(Debug, Deserialize)]
        enum Data {
            #[serde(rename = "Inteiros")]
            Integer(u64),
            #[serde(rename = "Pares")]
            Pair(String, String),
        }

        impl GetFieldNames for Data {}

        let headers = Data::get_headers_v2();

        println!("headers: {headers:#?}");

        Ok(())
    }

    */
}
