use std::path::PathBuf;
// use anyhow::{Error, Result};
use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
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
        /// Recipe name
        recipe: String,
        /// Path to a text file with a list of recipes to audit
        #[arg(short = 'l', long = "recipe-list", value_name = "TEXT_FILE")]
        recipelist: Option<PathBuf>,
    },
    /// Get info about configuration or a recipe
    Info {
        /// Recipe name
        recipe: String,
        /// Don't offer to search GitHub if a recipe can't be found
        #[arg(short, long)]
        quiet: bool,
    },
    /// Run one or more install recipes. Example: autopkg install Firefox -- equivalent to: autopkg run Firefox.install
    Install {
        /// Recipe name
        recipe: String,
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
    /// List all available Processors
    #[clap(visible_alias = "processor-list")]
    ListProcessors {
        /// List only Core processors
        #[arg(short = 'o', long)]
        core: bool,
        /// List only custom processors
        #[arg(short = 'c', long)]
        custom: bool,
    },
    /// List recipes available locally
    ListRecipes {
        // TODO: Consider turning this into a table
        /// Include recipe's identifier in the list
        #[arg(short, long = "with-identifiers")]
        identifiers: bool,
        /// Include recipe's path in the list
        #[arg(short, long = "with-paths")]
        paths: bool,
    },
    /// List installed recipe repos
    #[clap(visible_alias = "repo-list")]
    ListRepos {
        // no subcommands
    },
    /// Make a recipe override
    MakeOverride {
        /// Recipe to create override for
        recipe: String,
        /// Name for override file
        #[arg(short, long, value_name = "FILENAME")]
        name: Option<String>,
        /// Force overwrite an override file
        #[arg(short, long)]
        force: bool,
        /// Make an override even if the specified recipe or one of its parents is deprecated
        #[arg(long = "ignore-deprecation")]
        ignoredeprecation: bool,
        /// The format of the recipe override to be created. Valid options include: 'plist' or 'yaml' (default)
        #[arg(long, value_name = "FORMAT", default_value_t = Format::Yaml)]
        format: Format,
    },
    /// Make a new template recipe
    NewRecipe {
        /// Identifier for the new recipe
        #[arg(
            short,
            long,
            value_name = "IDENTIFIER",
            default_value = "com.github.autopkg.CHANGEME"
        )]
        identifier: String,
        /// Parent recipe identifier for this recipe
        #[arg(short, long = "parent-identifier", value_name = "IDENTIFIER")]
        parent: Option<String>,
        /// The format of the recipe to be created. Valid options include: 'plist' or 'yaml' (default)
        #[arg(long, value_name = "FORMAT", default_value_t = Format::Yaml)]
        format: Format,
    },
    /// Get information about a specific processor
    ProcessorInfo {
        /// Name of processor
        processor: Option<String>,
    },
    /// Add one or more recipe repos from a URL, or AutoPkg org on GitHub
    /// 
    /// Download one or more new recipe repos and add it to the search path
    /// The 'recipe_repo_url' argument can be of the following forms:
    /// - repo (implies 'https://github.com/autopkg/repo')
    /// - user/repo (implies 'https://github.com/user/repo')
    /// - (http[s]://|git://|ssh://|user@server:)path/to/any/git/repo
    #[command(verbatim_doc_comment)]
    RepoAdd {
        /// A repo name in AutoPkg org, user/repo combo, or URL of an AutoPkg recipe git repo
        recipe_repo_url: String,
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum Format {
    /// Property List format
    Plist,
    /// Yaml format
    Yaml,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Plist => write!(f, "plist"),
            Format::Yaml => write!(f, "yaml"),
        }
    }
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
        Some(Commands::Audit { recipelist, recipe }) => {
            // This would be from "audit -l <recipelist>"
            if let Some(recipelist) = recipelist {
                println!("Auditing recipes from list: {}", recipelist.display());
            } else {
                // This is if -l is not specified as a flag
                println!("Auditing recipe: {}", recipe);
            }
        }
        Some(Commands::Info { quiet, recipe }) => {
            // This would be from "info --quiet <recipe>"
            if *quiet {
                println!("Quiet mode is on");
            } else {
                // This is if --quiet is not specified as a flag
                println!("Quiet mode is off");
            }
            println!("Getting info for recipe: {}", recipe);
        }
        Some(Commands::Install {
            check,
            preprocessor,
            postprocessor,
            ignore,
            key,
            recipelist,
            pkg,
            reportplist,
            quiet,
            recipe,
        }) => {
            // This would be from "install --check <recipe>"
            if *check {
                println!("Checking for new/changed downloads");
            } else {
                // This is if --check is not specified as a flag
                println!("Not checking for new/changed downloads");
            }
            if let Some(preprocessor) = preprocessor {
                // This would be from "install -r <preprocessor>"
                println!("Preprocessor: {}", preprocessor);
            } else {
                // This is if -r is not specified as a flag
                println!("No preprocessor");
            }
            if let Some(postprocessor) = postprocessor {
                // This would be from "install -o <postprocessor>"
                println!("Postprocessor: {}", postprocessor);
            } else {
                // This is if -o is not specified as a flag
                println!("No postprocessor");
            }
            if *ignore {
                // This would be from "install --ignore-parent-trust-verification-errors"
                println!("Ignoring parent trust verification errors");
            } else {
                // This is if --ignore-parent-trust-verification-errors is not specified as a flag
                println!("Not ignoring parent trust verification errors");
            }
            if let Some(recipelist) = recipelist {
                // This would be from "install <recipe> -l <recipelist>"
                println!("Running recipes from list: {}", recipelist.display());
            } else {
                // This is if -l is not specified as a flag
                println!("Running recipe: {}", recipe);
            }
            if let Some(pkg) = pkg {
                // This would be from "install <recipe> <pkg>"
                println!("Providing pkg/dmg: {}", pkg.display());
            } else {
                // This is if <pkg> is not specified
                println!("No pkg/dmg provided");
            }
            if let Some(reportplist) = reportplist {
                // This would be from "install <recipe> --report-plist <reportplist>"
                println!("Saving run report plist to: {}", reportplist.display());
            } else {
                // This is if --report
                println!("No report plist saved");
            }
            if *quiet {
                // This would be from "install <recipe> --quiet"
                println!("Quiet mode is on");
            } else {
                // This is if --quiet is not specified as a flag
                println!("Quiet mode is off");
            }
            if let Some(key) = key {
                // This would be from "install <recipe> -k <key>"
                println!("-k pair specified:");
                for (k, v) in key {
                    println!("{}: {}", k, v);
                }
            } else {
                // This is if -k is not specified as a flag
                println!("No key/value pairs provided");
            }
        }
        Some(Commands::ListProcessors { core, custom }) => {
            if *core {
                // This would be from "list-processors -o"
                println!("Listing core processors");
            } else if *custom {
                // This would be from "list-processors -c"
                println!("Listing custom processors");
            } else {
                // This is if neither -o nor -c are specified as flags
                println!("Listing all processors");
            }
        }
        Some(Commands::ListRecipes { identifiers, paths }) => {
            if *identifiers {
                // This would be from "list-recipes -i"
                println!("Listing recipes with identifiers");
            } else if *paths {
                // This would be from "list-recipes -p"
                println!("Listing recipes with paths");
            } else {
                // This is if neither -i nor -p are specified as flags
                println!("Listing recipes");
            }
        }
        Some(Commands::ListRepos {}) => {
            // This would be from "list-repos"
            println!("Listing repos");
        }
        Some(Commands::MakeOverride {
            name,
            force,
            ignoredeprecation,
            format,
            recipe,
        }) => {
            // This would be from "make-override <recipe>"
            println!("Making override for recipe: {}", recipe);
            if let Some(name) = name {
                // This would be from "make-override --name <name>"
                println!("Override name: {}", name);
            } else {
                // This is if --name is not specified as a flag
                println!("No override name");
            }
            if *force {
                // This would be from "make-override --force"
                println!("Forcing override creation");
            } else {
                // This is if --force is not specified as a flag
                println!("Not forcing override creation");
            }
            if *ignoredeprecation {
                // This would be from "make-override --ignore-deprecation"
                println!("Ignoring deprecation");
            } else {
                // This is if --ignore-deprecation is not specified as a flag
                println!("Not ignoring deprecation");
            }
            println!("Format: {}", format);
        }
        Some(Commands::NewRecipe {
            identifier,
            parent,
            format,
        }) => {
            // This would be from "new-recipe -i <identifier>"
            println!("Making new recipe with identifier: {}", identifier);
            if let Some(parent) = parent {
                // This would be from "new-recipe --parent-identifier <parent>"
                println!("Parent identifier: {}", parent);
            } else {
                // This is if --parent-identifier is not specified as a flag
                println!("No parent identifier");
            }
            println!("Format: {}", format);
        }
        Some(Commands::ProcessorInfo { processor }) => {
            if let Some(processor) = processor {
                // This would be from "processor-info <processor>"
                println!("Getting info for processor: {}", processor);
            } else {
                // This is if <processor> is not specified
                println!("Getting info for all processors");
            }
        }
        Some(Commands::RepoAdd { recipe_repo_url }) => {
            // This would be from "repo-add <recipe_repo_url>"
            println!("Adding repo: {}", recipe_repo_url);
        }
        None => {} // This is if no subcommand is used
    }

    // Continued program logic goes here...
}
