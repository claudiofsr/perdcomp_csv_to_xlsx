// Functions defined in lib.rs
use perdcomp_csv_to_xlsx::*;

use execution_time::ExecutionTime;
use std::cmp::Reverse;
use tempfile::NamedTempFile;

/*
    clear && cargo test -- --nocapture
    clear && cargo run -- --help
    clear && cargo run -- -tkvp /tmp/teste.csv
    cargo b -r && cargo install --path=.
    perdcomp_csv_to_xlsx -tkvp ~/Documents/perdcomp.csv
*/

/*
SCC
* Filtros
NI (Declarante ou Sucessora): CNPJ Base
Tipo Crédito: Pis e COFINS
* Filtros Complementares
Credito PA: inicio e fim
* Processamento
Status PER/DCOMP: Ativo
Demonstra Crédito: Sim
*/

fn main() -> MyResult<()> {
    let timer = ExecutionTime::start();
    let arguments = Arguments::build()?;

    // See https://docs.rs/tempfile
    // Create a file inside of `std::env::temp_dir()`.
    let temporary = NamedTempFile::new()?;

    let paths = Paths {
        input: arguments.path.clone().into(), // CSV file
        output: temporary.path().into(),      // Temp file
    };

    if arguments.verbose {
        dbg!(&paths);
    }

    //let first_line = get_first_line(&paths)?;
    //println!("first_line: {first_line:?}");

    // Convert a csv file (WINDOWS_1252) to UTF8
    format_input_csv_file(&arguments, &paths)?;

    let mut perdcomps: Vec<PerDcomp> = read_csv(&arguments, &temporary)?;

    // Sort Vec<PerDcomp> by key
    perdcomps.sort_by_key(|perdcomp| {
        (
            perdcomp.ano,
            perdcomp.trimestre_de_apuracao.clone(),
            Reverse(perdcomp.tipo_do_credito.clone()),
            perdcomp.data_da_transmissao,
        )
    });

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
        let path_buf = temporary.into_temp_path().keep()?;
        rename_file(&path_buf, "temporary.csv")?;
    }

    // 1. Detecta colunas vazias apenas se o flag estiver ativo
    let columns_to_hide = if arguments.remove_empty {
        PerDcomp::get_empty_column_indices(&perdcomps)
    } else {
        Vec::new()
    };

    //println!("perdcomps: {perdcomps:#?}");

    // 2. Passa a lista para a função de escrita
    write_xlsx(&perdcomps, "PERDComp", "perdcomp.xlsx", &columns_to_hide)?;

    if arguments.time {
        timer.print_elapsed_time();
    }

    Ok(())
}
