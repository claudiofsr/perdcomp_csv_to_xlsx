use regex::Regex;
use std::sync::LazyLock;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

// Trimestre de apuracao = " 3º TRIMESTRE 2021 " | "3º TRIMESTRE de 2021"
pub static REGEX_TRIMESTRE_ANO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(?ix)
        (.*Trimestre.*?) # "3º TRIMESTRE"
        \s*              # whitespace
        (?:de|\/)?       # "de" ou "/"
        \s*              # whitespace
        (\d{2,4})        # year: 2021
    "#,
    )
    .unwrap()
});

// Regex para Datas (removido ^ e $ para permitir busca dentro de strings maiores)
pub static REGEX_DDMMYYYY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d{1,2})[-/](\d{1,2})[-/](\d{4})").unwrap());

// Regex para capturar o primeiro ano de 4 dígitos que encontrar (fallback)
pub static REGEX_ANO_GENERICO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{4})\b").unwrap());
