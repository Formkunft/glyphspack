use anyhow::{Result, bail};
use clap::{Arg, ArgAction, Command};
use glyphspack::{pack, unpack};
use std::path::Path;

const ARG_KEY_OUTFILE: &str = "OUT";
const ARG_KEY_FILE: &str = "IN";
const ARG_KEY_FORCE: &str = "FORCE";
const ARG_KEY_QUIET: &str = "QUIET";
const FILE_EXT_STANDALONE: &str = "glyphs";
const FILE_EXT_PACKAGE: &str = "glyphspackage";

enum Operation {
    Pack,
    Unpack,
}

fn main() -> Result<()> {
    let config = Command::new("glyphspack")
        .version("2.0.0")
        .author("Florian Pircher <florian@formkunft.com>")
        .about("Convert between .glyphs and .glyphspackage files. The conversion direction is automatically detected depending on whether <FILE> is a directory or not.")
        .after_help("See the Glyphs Handbook <https://glyphsapp.com/learn> for details on the standalone and the package format flavors.")
        .arg(
            Arg::new(ARG_KEY_OUTFILE)
                .short('o')
                .long("out")
                .help("The output file")
                .value_name("OUTFILE"),
        )
        .arg(
            Arg::new(ARG_KEY_FORCE)
                .short('f')
                .long("force")
                .help("Overwrites output file if it already exists")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(ARG_KEY_QUIET)
                .short('q')
                .long("quiet")
                .help("Suppresses log messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(ARG_KEY_FILE)
                .help("The input file")
                .value_name("FILE")
                .required(true)
                .index(1),
        )
        .get_matches();
    let force = config.get_flag(ARG_KEY_FORCE);
    let quiet = config.get_flag(ARG_KEY_QUIET);
    let out_file = config.get_one::<String>(ARG_KEY_OUTFILE);
    let in_file = config.get_one::<String>(ARG_KEY_FILE).unwrap();
    let in_path = Path::new(in_file.as_str());

    if !in_path.exists() {
        bail!("<FILE> does not exist: {}", in_path.display());
    }

    let operation = if in_path.is_dir() {
        Operation::Unpack
    } else {
        Operation::Pack
    };

    let out_path = match out_file {
        Some(file) => Path::new(file.as_str()).to_owned(),
        None => match operation {
            Operation::Pack => in_path.with_extension(FILE_EXT_PACKAGE),
            Operation::Unpack => in_path.with_extension(FILE_EXT_STANDALONE),
        },
    };

    if !force && out_path.exists() {
        bail!("<OUTFILE> already exists: {}", out_path.display());
    }

    match operation {
        Operation::Pack => {
            if !quiet {
                eprintln!("Packing {} into {}", in_path.display(), out_path.display());
            }
            pack::pack(in_path, &out_path, force)
        }
        Operation::Unpack => {
            if !quiet {
                eprintln!(
                    "Unpacking {} into {}",
                    in_path.display(),
                    out_path.display()
                );
            }
            unpack::unpack(in_path, &out_path)
        }
    }
}
