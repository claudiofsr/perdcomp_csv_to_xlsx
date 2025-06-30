use regex::Regex;
use std::sync::LazyLock;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

// Trimestre de apuracao = " 3º TRIMESTRE 2021 " | "3º TRIMESTRE de 2021"
pub static REGEX_TRIMESTRE_ANO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(?ix)
        ^                # start
        \s*              # whitespace
        (.*Trimestre.*?) # "3º TRIMESTRE"
        \s*              # whitespace
        (?:de|\/)?       # "de" ou "/"
        \s*              # whitespace
        (\d{2,4})        # year: 2021
        \s*              # whitespace
        $                # final
    "#,
    )
    .unwrap()
});

pub static REGEX_DDMMYYYY: LazyLock<Regex> = LazyLock::new(||
    // 25/05/2023 12:39:04
    // 25/05/23 12:39:04
    // 25/05/20 12:39
    // 10/06/2024
    // 03-12-2021 09:13:20

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
    "#).unwrap());
