#[macro_use]
extern crate clap;
extern crate mt940;
extern crate serde_json;

use clap::{App, Arg, AppSettings};
use mt940::parse_mt940;
use mt940::sanitizers::to_swift_charset;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn is_file(p: String) -> Result<(), String> {
    if Path::new(&p).is_file() {
        Ok(())
    } else {
        Err(format!(
            "Path '{}' doesn't exist or is not a regular file.",
            &p
        ))
    }
}

fn has_parent_dir(p: String) -> Result<(), String> {
    let parent_dir = if let Some(p) = Path::new(&p).parent() {
        p
    } else {
        return Err("Path doesn't have a parent dir.".into());
    };
    if parent_dir.is_dir() {
        Ok(())
    } else {
        Err(format!(
            "Path '{}' doesn't exist or is not a regular file.",
            &p
        ))
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Convert mt940 statement to json. \n\n\
                It will try to sanitize input by default. For instance, it will \
                try to fit all found characters into the allowable SWIFT charset \
                before attempting conversion. You can turn off this behavior by \
                enabling strict mode.")
        .global_setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("strict")
                .short("s")
                .help("Enable strict parsing\nWhen this is on, no automatic sanitizing will take place."),
        )
        .arg(
            Arg::with_name("statement")
                .value_name("STATEMENT")
                .takes_value(true)
                .required(true)
                .validator(is_file)
                .help("Input mt940 statement"),
        )
        .arg(
            Arg::with_name("output")
                .value_name("OUTPUT")
                .takes_value(true)
                .validator(has_parent_dir)
                .help("Output file in JSON format"),
        )
        .get_matches();

    let is_strict = value_t!(matches, "strict", bool).unwrap_or(false);
    let statement_input = value_t_or_exit!(matches, "statement", String);

    let mut input = fs::read_to_string(statement_input)?;

    // Do some sanitizing if not running in strict mode.
    if !is_strict {
        input = to_swift_charset(&input);
    }

    let parsed = parse_mt940(&input).unwrap_or_else(|e| panic!("{}", e));

    let json = serde_json::to_string_pretty(&parsed)?;

    if matches.is_present("output") {
        // Write to a file.
        let output = value_t_or_exit!(matches, "output", String);
        fs::write(output, json)?;
    } else {
        // Write to stdout instead.
        io::stdout().write_all(json.as_bytes())?;
    };

    Ok(())
}
