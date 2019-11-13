use std::{
    fmt,
    io::{self, Write},
    process,
    str::FromStr,
};

use clap::crate_version;

use crate::input::Format;
use tokei::{Language, LanguageType};

pub const FALLBACK_ROW_LEN: usize = 79;
const NO_LANG_HEADER_ROW_LEN: usize = 67;
const NO_LANG_ROW_LEN: usize = 61;
const NO_LANG_ROW_LEN_NO_SPACES: usize = 54;
const IDENT_INACCURATE: &str = "(!)";

pub fn crate_version() -> String {
    if Format::supported().is_empty() {
        format!(
            "{} compiled without serialization formats.",
            crate_version!()
        )
    } else {
        format!(
            "{} compiled with serialization support: {}",
            crate_version!(),
            Format::supported().join(", ")
        )
    }
}

pub fn setup_logger(verbose_option: u64) {
    use log::LevelFilter;

    let mut builder = env_logger::Builder::new();

    let filter_level = match verbose_option {
        1 => LevelFilter::Warn,
        2 => LevelFilter::Debug,
        3 => LevelFilter::Trace,
        _ => LevelFilter::Error,
    };

    builder.filter(None, filter_level);
    builder.init();
}

pub fn parse_or_exit<T>(s: &str) -> T
where
    T: FromStr,
    T::Err: fmt::Display,
{
    T::from_str(s).unwrap_or_else(|e| {
        eprintln!("Error:\n{}", e);
        process::exit(1);
    })
}

pub fn print_header<W: Write>(sink: &mut W, row: &str, columns: usize) -> io::Result<()> {
    writeln!(sink, "{}", row)?;
    writeln!(
        sink,
        " {:<6$} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "Language",
        "Files",
        "Lines",
        "Code",
        "Comments",
        "Blanks",
        columns - NO_LANG_HEADER_ROW_LEN
    )?;

    writeln!(sink, "{}", row)
}

pub fn print_results<'a, I, W>(
    sink: &mut W,
    row: &str,
    languages: I,
    list_files: bool,
) -> io::Result<()>
where
    I: Iterator<Item = (&'a LanguageType, &'a Language)>,
    W: Write,
{
    let path_len = row.len() - NO_LANG_ROW_LEN_NO_SPACES;
    let columns = row.len();
    for (name, language) in languages.filter(isnt_empty) {
        print_language(sink, columns, language, name.name())?;

        if list_files {
            writeln!(sink, "{}", row)?;
            for stat in &language.stats {
                writeln!(sink, "{:1$}", stat, path_len)?;
            }
            writeln!(sink, "{}", row)?;
        }
    }

    Ok(())
}

pub fn isnt_empty(&(_, language): &(&LanguageType, &Language)) -> bool {
    !language.is_empty()
}

pub fn print_language<W>(
    sink: &mut W,
    columns: usize,
    language: &Language,
    name: &str,
) -> io::Result<()>
where
    W: Write,
{
    let mut lang_section_len = columns - NO_LANG_ROW_LEN;
    if language.inaccurate {
        lang_section_len -= IDENT_INACCURATE.len();
    }
    // truncate and replace the last char with a `|` if the name is too long
    if lang_section_len < name.len() {
        write!(sink, " {:.len$}", name, len = lang_section_len - 1)?;
        write!(sink, "|")?;
    } else {
        write!(sink, " {:<len$}", name, len = lang_section_len)?;
    }
    if language.inaccurate {
        write!(sink, "{}", IDENT_INACCURATE)?;
    };
    write!(sink, " ")?;
    writeln!(
        sink,
        "{:>6} {:>12} {:>12} {:>12} {:>12}",
        language.stats.len(),
        language.lines,
        language.code,
        language.comments,
        language.blanks
    )
}

pub fn print_inaccuracy_warning<W>(sink: &mut W) -> io::Result<()>
where
    W: Write,
{
    writeln!(
        sink,
        "Note: results can be inaccurate for languages marked with '{}'",
        IDENT_INACCURATE
    )
}
