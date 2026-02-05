use crate::MyResult;
use clap::{ArgAction, Parser};
use std::path::PathBuf;

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
fn get_styles() -> clap::builder::Styles {
    let cyan = anstyle::Color::Ansi(anstyle::AnsiColor::Cyan);
    let green = anstyle::Color::Ansi(anstyle::AnsiColor::Green);
    let yellow = anstyle::Color::Ansi(anstyle::AnsiColor::Yellow);

    clap::builder::Styles::styled()
        .placeholder(anstyle::Style::new().fg_color(Some(yellow)))
        .usage(anstyle::Style::new().fg_color(Some(cyan)).bold())
        .header(
            anstyle::Style::new()
                .fg_color(Some(cyan))
                .bold()
                .underline(),
        )
        .literal(anstyle::Style::new().fg_color(Some(green)))
}

/*
https://www.perrygeo.com/getting-started-with-application-configuration-in-rust.html
https://stackoverflow.com/questions/55133351/is-there-a-way-to-get-clap-to-use-default-values-from-a-file
https://rust-cli.github.io/book/in-depth/config-files.html
https://docs.rs/confy/latest/confy/index.html

How to Set Environment Variables in Linux:
export DELIMITER_CSV=';'

How to Print Environment Variables in Linux:
printenv DELIMITER_CSV
or
echo $DELIMITER_CSV

Removing shell variable and values:
unset DELIMITER_CSV
*/

// https://docs.rs/clap/latest/clap/struct.Command.html#method.help_template
const APPLET_TEMPLATE: &str = "\
{before-help}
{about-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}";

#[derive(Parser, Debug)]
#[command(
    // Read from `Cargo.toml`
    author, version, about,
    long_about = None,
    next_line_help = true,
    help_template = APPLET_TEMPLATE,
    styles=get_styles(),
)]
pub struct Arguments {
    /// Set the field delimiter to use when parsing CSV.
    ///
    /// The default is b','.
    #[arg(
        short('d'),
        long,
        env("DELIMITER_CSV"),
        required = false,
        default_value_t = ','
    )]
    pub delimiter: char,

    /// Prevent the temporary file from being deleted.
    ///
    /// And then, rename the temporary file to “temporary.csv”.
    ///
    /// <https://docs.rs/tempfile/latest/tempfile/struct.TempPath.html#method.keep>
    ///
    /// <https://docs.rs/clap/latest/clap/enum.ArgAction.html>
    #[arg(short('k'), long("keep"), default_value_t = false, action=ArgAction::SetTrue)]
    pub keep: bool,

    /// Set the csv file path.
    #[arg(short('p'), long("path"), required = true)]
    pub path: PathBuf,

    /// Remove columns that are empty in all rows.
    #[arg(short('r'), long("remove-empty"), default_value_t = false, action=ArgAction::SetTrue)]
    pub remove_empty: bool,

    /// Show total execution time.
    #[arg(short('t'), long("time"), default_value_t = false)]
    pub time: bool,

    /// Show intermediate runtime messages.
    ///
    /// Display up to the first 50 lines.
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> MyResult<Arguments> {
        let args: Arguments = Arguments::parse();
        Ok(args)
    }
}
