use std::path::PathBuf;
// use anyhow::{Error, Result};
use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct APcli {
    /// Sets a custom preferences file
    #[arg(short, long, value_name = "FILE")]
    prefs: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Audit one or more recipes
    Audit {
        /// Path to a text file with a list of recipes to audit
        #[arg(short = 'l', long = "recipe-list", value_name = "TEXT_FILE")]
        recipelist: Option<PathBuf>,
    },
    /// Get info about configuration or a recipe
    Info {
        /// Don't offer to search GitHub if a recipe can't be found
        #[arg(short, long)]
        quiet: bool,
    },
    /// Run one or more install recipes. Example: autopkg install Firefox -- equivalent to: autopkg run Firefox.install
    Install {
        /// Name of a processor to run before each recipe. Can be repeated to run multiple preprocessors
        #[arg(short = 'r', long, value_name = "PREPROCESSOR")]
        preprocessor: Option<String>,
        /// Name of a processor to run after each recipe. Can be repeated to run multiple postprocessors
        #[arg(short = 'o', long, value_name = "POSTPROCESSOR")]
        postprocessor: Option<String>,
        /// Only check for new/changed downloads
        #[arg(short, long)]
        check: bool,
        /// Run recipes even if they fail parent trust verification
        #[arg(short, long = "ignore-parent-trust-verification-errors")]
        ignore: bool,
        /// Provide key/value pairs for recipe input. Caution: values specified here will be applied to all recipes
        #[arg(short, long, value_name = "KEY=VALUE", value_parser = parse_key_value::<String, String>)]
        key: Option<Vec<(String, String)>>,
        /// Path to a text file with a list of recipes to run
        #[arg(short = 'l', long = "recipe-list", value_name = "TEXT_FILE")]
        recipelist: Option<PathBuf>,
        /// Path to a pkg or dmg to provide to a recipe. Downloading will be skipped
        #[arg(short, long, value_name = "PKG_OR_DMG")]
        pkg: Option<PathBuf>,
        /// File path to save run report plist
        #[arg(long = "report-plist", value_name = "OUTPUT_PATH")]
        reportplist: Option<PathBuf>,
        /// Don't offer to search GitHub if a recipe can't be found
        #[arg(short, long)]
        quiet: bool,
    },
}

/// Parse a single key-value pair
// Taken directly from https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
fn parse_key_value<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

fn main() {
    let cli: APcli = APcli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(config_path) = cli.prefs.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Audit { recipelist }) => {
            if recipelist.is_some() {
                // This would be from "audit -d <something>" or "audit --list"
                println!("Printing testing lists...");
            } else {
                // This is if --list is not specified as a flag
                println!("Not printing testing lists...");
            }
        }
        Some(Commands::Info { quiet }) => {
            if *quiet {
                // This would be from "info --quiet"
                println!("Quiet mode is on");
            } else {
                // This is if --quiet is not specified as a flag
                println!("Quiet mode is off");
            }
        }
        Some(Commands::Install {
            check,
            preprocessor: _,
            postprocessor: _,
            ignore: _,
            key,
            recipelist: _,
            pkg: _,
            reportplist: _,
            quiet: _,
        }) => {
            if *check {
                // This would be from "install --check"
                println!("Check mode is on");
            } else {
                // This is if --check is not specified as a flag
                println!("Check mode is off");
            }
            if let Some(key) = key {
                // This would be from "install -k key=value"
                for (k, v) in key {
                    println!("Key: {}, Value: {}", k, v);
                }
            }
        }
        None => {} // This is if no subcommand is used
    }

    // Continued program logic goes here...
}
