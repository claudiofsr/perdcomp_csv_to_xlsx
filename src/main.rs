use std::{
    cmp::Reverse,
    time::Instant,
};

// Functions defined in lib.rs
use perdcomp_csv_to_xlsx::*;
use tempfile::NamedTempFile;

/*
    clear && cargo test -- --nocapture
    clear && cargo run -- --help
    clear && cargo run -- -tkvp /tmp/teste.csv
    cargo b -r && cargo install --path=.
    perdcomp_csv_to_xlsx -tkvp ~/Documents/perdcomp.csv
*/

fn main() -> MyResult<()> {

    let time = Instant::now();
    let arguments = Arguments::build()?;

    // See https://docs.rs/tempfile
    // Create a file inside of `std::env::temp_dir()`.
    let temporary = NamedTempFile::new()?;

    let paths = Paths {
        input: arguments.path.clone().into(), // CSV file
        output: temporary.path().into(),      // Temp file
    };

    //let first_line = get_first_line(&paths)?;
    //println!("first_line: {first_line:?}");

    // Convert a csv file (WINDOWS_1252) to UTF8
    format_input_csv_file(&arguments, &paths)?;

    let mut perdcomps: Vec<PerDcomp> = read_csv(&arguments, &temporary)?;

    // Sort Vec<PerDcomp> by key
    perdcomps.sort_by_key(|perdcomp| (
        perdcomp.get_year(),
        perdcomp.trimestre_de_apuracao.clone(),
        Reverse(perdcomp.tipo_do_credito.clone()),
        perdcomp.data_da_transmissao,
    ));

    if arguments.verbose {
        println!("Display up to the first 50 lines:\n");
        perdcomps
            .iter()
            .take(50)
            .enumerate()
            .for_each(|(index, perdcomp)| {
                println!("line {:02}: {perdcomp:?}\n", index + 1);
            })
    }

    // Prevent the temporary file from being deleted.
    // And then, rename the temporary file to “temporary.csv”.
    if arguments.keep {
        let path_buf = temporary
            .into_temp_path()
            .keep()?;
        rename_file(&path_buf, "temporary.csv")?;
    }

    //println!("perdcomps: {perdcomps:#?}");
    write_xlsx(&perdcomps, "PERDComp", "perdcomp.xlsx")?;

    if arguments.time {
        eprintln!("\nTotal Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}
