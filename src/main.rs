use std::path::PathBuf;

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
        /// Display the help message
        #[arg(short, long)]
        quiet: bool,
    },
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
        None => {} // This is if no subcommand is used
    }

    // Continued program logic goes here...
}
