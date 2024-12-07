use once_cell::sync::Lazy;
use regex::Regex;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

/// Example:
///
/// <https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html>
pub static REGEX_CANCELAMENTO: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Cancelamento
    ").unwrap()
);

pub static REGEX_CENTER: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        # non-capturing group: (?:regex)
        ^(:?
            Ano|
            CNPJ|CPF|CST|
            Chave|NCM|
            Registro|Identifica|
            Cancelado|
            Estado|
            Per.*Dcomp|
            Trimestre|
            N.*Processo|
            UA\s*(Declarante|Detentor)
        )|
        Código|
        PGD
    ").unwrap()
);

pub static REGEX_VALUE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Total|Valor
    ").unwrap()
);

pub static REGEX_ALIQ: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Alíquota
    ").unwrap()
);

pub static REGEX_DATE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        ^(:?Data|Dia)
    ").unwrap()
);

pub static REGEX_FIELDS: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?x)
        \b # word boundary
        (:?
            CTe|protCTe|
            NFe|protNFe|
            evento|retEvento|
            eventoCTe|retEventoCTe|
            evtMovOpFin
        )
        \b # word boundary
    ").unwrap()
);

pub static REGEX_ERROR_MISSING_FIELD: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        missing\s*field
    ").unwrap()
);

pub static REGEX_ERROR_DUPLICATE_FIELD: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        duplicate\s*field
    ").unwrap()
);

// Trimestre de apuracao = " 3º TRIMESTRE 2021 " | "3º TRIMESTRE de 2021"
pub static REGEX_TRIMESTRE_ANO: Lazy<Regex> = Lazy::new(||
    Regex::new(r#"^(?x)
        ^       # start
        \s*     # whitespace
        (.*?)   # "3º TRIMESTRE"
        [de\s]* # " de "
        (\d{4}) # year: 2021
        \s*     # whitespace
        $       # final
    "#).unwrap()
);

pub static REGEX_DDMMYYYY: Lazy<Regex> = Lazy::new(||
    // 25/05/2023 12:39:04
    // 25/05/23 12:39:04
    // 25/05/20 12:39

    // Regex::new(r"^\s*(\d{1,2})\s*/\s*(\d{1,2})\s*/\s*(\d{2,4}).*").unwrap()

    // https://docs.pola.rs/docs/rust/dev/src/polars_time/chunkedarray/string/infer.rs.html

    Regex::new(r#"^(?x)
        ^            # start
        \s*          # whitespace
        ['"]?        # optional quotes
        \s*          # whitespace
        (\d{1,2})    # day
        \s*          # whitespace
        [-/]         # separator
        \s*          # whitespace
        (\d{1,2})    # month
        \s*          # whitespace
        [-/]         # separator
        \s*          # whitespace
        (\d{2,4})    # year

        (?:
            [T\s]                 # separator
            (?:\d{2})             # hour
            :?                    # separator
            (?:\d{2})             # minute
            (?:
                :?                # separator
                (?:\d{2})         # second
                (?:
                    \.(?:\d{1,9}) # subsecond
                )?
            )?
        )?

        \s*          # whitespace
        ['"]?        # optional quotes
        \s*          # whitespace
        $            # final
    "#).unwrap()
);
