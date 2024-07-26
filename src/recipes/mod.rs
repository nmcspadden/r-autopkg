use anyhow::Result;
use plist::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::BufReader;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{fs, io, iter};
use tracing::{debug, error, info, span, trace, warn, Level};
use tracing_subscriber::field::debug;
use walkdir::{DirEntry, WalkDir};

use crate::{constants, recipes, Preferences};

/// Recipes are AutoPkg's primary object
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Recipe {
    /// Human-readable description of the recipe
    pub description: String,
    /// Unique identifier for recipe
    pub identifier: String,
    /// Minimum version of AutoPkg necessary to use recipe
    pub minimum_version: String,
    /// Parent recipes are optional, such as in download recipes
    pub parent_recipe: Option<String>,
    /// Input variables that can be overridden
    pub input: HashMap<String, PlistDataType>,
    /// A list of Processors to execute in serial
    pub process: Vec<Processor>,
    /// Trust info (only present in Overrides!)
    pub parent_recipe_trust_info: Option<ParentRecipeTrust>,
}

impl Recipe {
    /// Instantiate a new Recipe object with only the strings. Input and Process variables must be created later to be valid.
    pub fn new(
        description: String,
        identifier: String,
        minimum_version: String,
        parent_recipe: Option<String>,
    ) -> Recipe {
        // Set up a Processor struct instantiation that contains only one processor and no arguments, for simplicity
        let initial_processor: Processor = Processor {
            processor: "EndOfCheckPhase".to_string(),
            arguments: None,
        };
        // The input dictionary contains only "NAME" with a value of "test_recipe"
        Recipe {
            description,
            identifier,
            minimum_version,
            parent_recipe,
            input: HashMap::from([(
                "NAME".to_string(),
                PlistDataType::Str("test_recipe".to_string()),
            )]),
            process: vec![initial_processor],
            parent_recipe_trust_info: None,
        }
    }
    pub fn has_parent(&self) -> bool {
        if self.parent_recipe.as_deref().unwrap_or("").is_empty() {
            false
        } else {
            debug!("Found parent: {:?}", &self.parent_recipe);
            true
        }
    }

    /// Validate that the recipe matches some basic rules
    pub fn is_valid_recipe(&self) -> bool {
        // Descriptions, Identifier, and Minimum Version must be defined
        let static_strings_are_not_empty = !(self.description.is_empty()
            || self.identifier.is_empty()
            || self.minimum_version.is_empty());
        // The Input must contain a key "NAME"
        let input_contains_name: bool = self.input.contains_key("NAME");
        static_strings_are_not_empty && input_contains_name
    }
}

#[derive(Debug, Deserialize)]
pub struct ParentRecipeTrust {
    /// Non-core processors by identifier/path
    non_core_processors: HashMap<String, TrustBlock>,
    /// All parents by identifier
    parent_recipes: HashMap<String, TrustBlock>,
}

#[derive(Debug, Deserialize)]
pub struct TrustBlock {
    git_hash: String,
    path: String,
    sha256_hash: String,
}

#[derive(Debug)]
pub struct UnreadableFileError;

/// Plists (and yaml) can contain only limited possible values
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PlistDataType {
    ArrayOfDicts(Vec<HashMap<String, String>>),
    ArrayOfStrs(Vec<String>),
    Bool(bool),
    DictOfDicts(HashMap<String, PlistDataType>),
    DictOfStrs(HashMap<String, String>),
    Str(String),
}

/// Processors all contain a processor name, and potentially arguments, which is a dictionary of PlistDataTypes
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Processor {
    processor: String,
    arguments: Option<HashMap<String, PlistDataType>>,
}

/// Set a shorthand for RecipeMap for a sorted btreemap
type RecipeMap = BTreeMap<String, BTreeMap<String, String>>;

/// Read in the path with a plist parser
///
/// The plist parser isn't very good, and will blow up if you look at it funny
/// This is probably going to fail way more often than we'd expect
pub fn read_plist_recipe(plist_path: &Path) -> Result<Recipe, plist::Error> {
    trace!("Attempting to parse plist at {}", plist_path.display());
    let plist_recipe: Recipe = plist::from_file(plist_path)?;
    Ok(plist_recipe)
}

/// Read in the path with a yaml parser
pub fn read_yaml_recipe(yaml_path: &Path) -> Result<Recipe, serde_yaml::Error> {
    trace!("Attempting to parse yaml at {}", yaml_path.display());
    let f = fs::File::open(yaml_path).expect("Could not open file!");
    let recipe: Recipe = serde_yaml::from_reader(f)?;
    Ok(recipe)
}

/// Attempt to read in the recipe at path, but if it fails, return
/// a custom error struct UnreadableFileError
pub fn read_recipe(path: &Path) -> Result<Recipe, UnreadableFileError> {
    trace!("Loading {} - trying plist first", path.display());
    read_plist_recipe(path)
        .or_else(|err| read_yaml_recipe(path))
        .map_err(|_| UnreadableFileError)
}

fn find_parent(recipe: &Recipe, prefs: &Preferences) -> Option<Recipe> {
    if let Some(parent) = &recipe.parent_recipe {
        debug!("Found parent: {parent}");
        let _ = read_recipe(&get_recipe_path_by_identifier(&recipe.identifier, prefs));
    };
    None
}

/// Take a Recipe and return its parent identifier
fn get_parent_identifier(recipe: &Recipe) -> Option<String> {
    recipe.parent_recipe.to_owned()
}

/// Take an identifier and return its parent identifier
fn get_parent_identifier_from_id(id: &str, prefs: &Preferences) -> Option<String> {
    let recipe_path = get_recipe_path_by_identifier(id, prefs);
    let recipe = read_recipe(&recipe_path).unwrap();
    recipe.parent_recipe
}

/// Get the identifier of a parent recipe from disk.
///
/// This takes a recipe path and reads the file in, and will panic if it
/// cannot read the file.
fn get_recipe_parent_identifier_from_path(
    recipe_path: &str,
    prefs: &Preferences,
) -> Option<String> {
    let recipe_id = "com.github.autopkg.install.AutoPkg-Release";

    let recipe_path = get_recipe_path_by_identifier(recipe_id, prefs);
    info!("Path: {}", recipe_path.display());

    let recipe = match read_recipe(&recipe_path) {
        Ok(recipe) => recipe,
        Err(e) => panic!("Unable to read recipe!"),
    };
    recipe.parent_recipe
}

pub fn load_recipe(id: &str, prefs: &Preferences) -> Recipe {
    trace!("Loading identifier at {id}");
    // This should take a path and load up its parents
    // 1. If it has a parent, follow that to the parent
    // 2. Load the parent recipes into a Vec of Recipes
    // 3. Merge the inputs together, first in, first out
    // meaning, the childmost recipe should have the final say on values of keys;
    // any keys defined in parents will just persist through. Last one always wins.
    // 4. Combine the Processes together, from first (top parent) to last (child)
    // 5. Return the combined Recipe
    let id_path = get_recipe_path_by_identifier(id, prefs);
    let recipe: Recipe = read_recipe(&id_path).unwrap();
    // What if, instead of reading each recipe as I load its parents, I instead create
    // a list of identifiers for all the parents, and then iterate through the vec
    // and load each recipe?
    // That seems much easier
    debug!("Pushing starting recipe onto pile");
    let parent_id: String = match get_parent_identifier(&recipe) {
        Some(parent_id) => parent_id,
        None => return recipe, // if there's no parent, just return this recipe
    };
    let mut identifier_chain: Vec<String> = vec![id.to_owned()];
    while let Some(parent_id) = get_parent_identifier(&recipe) {
        debug!("Pushing parent {parent_id} onto pile");
        identifier_chain.push(parent_id);
        let id_path = get_recipe_path_by_identifier(id, prefs);
        let recipe = read_recipe(&id_path).unwrap();
    }
    debug!("Ids in the vec: {:?}", identifier_chain);
    // TODO: This is fake for now to satisfy the build:
    recipe
}

/// This takes a DirEntry reference from a Walkdir walker
/// and returns true if the filename ends with ".recipe"
///
/// TODO: Make this generic
fn is_recipe_file(entry: &DirEntry) -> bool {
    trace!("file_is_recipe: {:?}", &entry.file_name());
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".recipe"))
        .unwrap_or(false)
}

/// This takes a DirEntry reference from a Walkdir walker
/// and returns true if the name starts with ".git"
fn is_git_folder(entry: &DirEntry) -> bool {
    trace!("is_git_folder: {:?}", &entry.file_name());
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".git"))
        .unwrap_or(false)
}

/// Get all .recipe files recursively from a folder
///
/// Note that this only goes two additional folder depth, so it's intended to
/// search the Recipes dir, which could contain recipe files directly, or
/// folders therein. The RecipeRepos dir will contain repo folders, which
/// themselves can contain subfolders for recipes.
///
/// This will return a vector of paths to each of the recipes it finds.
fn get_all_recipes_recursively_from_folder<P>(path: &P) -> Vec<PathBuf>
where
    P: AsRef<Path>, // This function takes any type that implements AsRef<Path>
                    // which could be a PathBuf or Path
{
    let mut recipe_files = Vec::new();
    let walker = WalkDir::new(path)
        .follow_links(true)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_entry(|f| !is_git_folder(f)); // don't bother looking inside .git folder
    for entry in walker.filter_map(|f| f.ok()) {
        if is_recipe_file(&entry) {
            trace!("{}", entry.path().display());
            recipe_files.push(entry.path().to_path_buf())
        }
    }
    // Alternative way of doing this:
    // .filter_map(|f| {
    //     match f {
    //         Ok(entry) if is_recipe_file(&entry) => {
    //             debug!("{}", entry.path().display());
    //             Some(entry.path().to_path_buf())
    //         },
    //         _ => None,
    //     }
    // })
    recipe_files
}

/// Read a plist file and return a specific String value
///
/// This will fail if the key being asked for isn't a String.
/// This function probably needs to be totally rewritten to support Results.
fn get_key_from_recipe_file(recipe: &Path, key: &str) -> String {
    let recipe_data = Value::from_file(recipe).expect("failed to read plist");
    get_string_key_from_recipe_value(&recipe_data, key)
}

/// Return the String value of a specific key name from a recipe represented
/// as a Plist::Value.
///
/// This probably needs to be made more generic.
fn get_string_key_from_recipe_value(recipe_data: &Value, key: &str) -> String {
    let identifier = recipe_data
        .as_dictionary()
        .and_then(|dict| dict.get(key))
        .and_then(|identifier| identifier.as_string())
        .unwrap();
    trace!("{}: {}", key, identifier);
    identifier.to_string()
}

/// Given a specific recipe file path, return the "short name" of the recipe
///
/// For example:
/// `RecipeRepos/nmcspadden-recipes/Something/Something.download.recipe` ->
/// `Something.download`
fn calculate_short_name(entry: &Path) -> String {
    let no_ext = entry.file_stem().unwrap();
    trace!("Stem: {:?}", no_ext);
    no_ext.to_owned().into_string().unwrap()
}

fn build_maps_from_folder(
    expanded_path: &Path,
    identifier_map: &mut BTreeMap<String, String>,
    shortname_map: &mut BTreeMap<String, String>,
) {
    info!("Considering looking through {}", expanded_path.display());

    let recipes_in_folder = get_all_recipes_recursively_from_folder(&expanded_path);
    info!("Calculating identifiers and shortnames");
    for recipe in recipes_in_folder {
        trace!("Recipe: {}", recipe.display());
        let identifier = get_key_from_recipe_file(&recipe, "Identifier");
        let shortname = calculate_short_name(&recipe);

        // We must convert the recipe PathBuf here into a String, and so we
        // shadow the 'recipe' variable name since we don't need its original value anymore
        let recipe = recipe.into_os_string().into_string().unwrap();
        // We have to clone it explicitly because otherwise we have an ownership collision
        identifier_map.insert(identifier, recipe.clone());
        shortname_map.insert(shortname, recipe);
    }
}

/// Get a list of all directories one level under a topdir
fn list_dirs_within_folder(topdir: &Path) -> Result<Vec<PathBuf>> {
    // Shout out to https://stackoverflow.com/a/67940478 for this
    Ok(read_dir(topdir)?
        .filter_map(|f| {
            f.ok().and_then(|d| {
                let p = d.path();
                if p.is_dir() {
                    Some(p)
                } else {
                    None
                }
            })
        })
        .collect())
}

/// Build a recipe map of all known recipes.
///
/// The recipe map is a dictionary that contains top-level keys:
/// {
///   "identifiers": {
///     identifier: absolute file path
///   },
///   "overrides": {
///     override_name: absolute file path
///   },
///   "shortnames": {
///     short_name: absolute file path
///   },
/// }
/// TODO: Add support for Overrides
pub fn build_recipe_map(prefs: &Preferences) -> Result<RecipeMap, Box<dyn std::error::Error>> {
    // We're using BTreeMaps here because they are always sorted by keys
    // This means the JSON representation of these will be sorted, and
    // deterministic
    let mut recipe_map: RecipeMap = BTreeMap::new();
    let mut identifier_map: BTreeMap<String, String> = BTreeMap::new();
    let mut shortname_map: BTreeMap<String, String> = BTreeMap::new();

    // Look for recipes in the recipe repo parent folder first
    // TODO: Iterate through the search dirs along with the recipe repo parent folder to look for recipes
    let dirs = prefs.recipe_search_dirs.iter();
    let repos = iter::once(&prefs.recipe_repo_dir);
    let paths_to_search = dirs.chain(repos);

    for folder in paths_to_search {
        build_maps_from_folder(folder, &mut identifier_map, &mut shortname_map);
    }

    recipe_map.insert("identifiers".to_string(), identifier_map);
    recipe_map.insert("shortnames".to_string(), shortname_map);

    // Emit to disk
    info!("Writing recipe map to disk at {:?}", prefs.recipe_map_path);
    std::fs::write(
        &prefs.recipe_map_path,
        serde_json::to_string_pretty(&recipe_map).unwrap(),
    )?;

    Ok(recipe_map)
}

/// Read the recipe map from JSON file
pub fn read_recipe_map(prefs: &Preferences) -> Result<RecipeMap> {
    // Reading the file into a string first is significantly faster than
    // reading directly from a reader: https://github.com/serde-rs/json/issues/160
    let json_data = fs::read_to_string(&prefs.recipe_map_path)?;
    // The BTreeMap here is strongly typed, so it force converts all the JSON data to
    // the expected String types
    let recipe_map: RecipeMap = serde_json::from_str(&json_data)?;
    Ok(recipe_map)
}

pub fn find_recipe_in_map(map: &RecipeMap, recipe: &str) -> Option<String> {
    debug!("find_recipe_in_map: Recipe {recipe}");
    map["identifiers"]
        .get(recipe)
        // .and(map["overrides"].get(recipe))
        .or_else(|| map["shortnames"].get(recipe))
        .cloned()
}

/// Generate an override for a recipe
// pub fn generate_recipe_override(recipe: &Recipe) -> Recipe {
pub fn generate_recipe_override(recipe: &Recipe) {
    debug!("Generating override!");
    // To generate an override, we now have to actually load the full recipe
    // We need all the identifiers paths in the recipe chain,
    // along with any non-core processors in order to correctly
    // generate a hash
}

/// Find a recipe path by searching map for an identifier.
///
/// Panics if recipe is not found in map. This should probably be rewritten
/// to return a Result instead. This nested match is ugly.
pub fn get_recipe_path_by_identifier(identifier: &str, prefs: &Preferences) -> PathBuf {
    let recipe_map = match read_recipe_map(prefs) {
        Ok(recipe_map) => recipe_map,
        Err(e) => panic!("Unable to read recipe map: {}", e),
    };
    match recipe_map["identifiers"].get(identifier) {
        Some(path) => PathBuf::from(path),
        None => panic!("Identifier {identifier} not found in recipe map!"),
    }
}

// I need to figure out how to get this to return a result correctly
// pub fn get_recipe_path_by_identifier2(identifier: &str) -> Result<String> {
//     // let recipe_map = match read_recipe_map() {
//     //     Ok(recipe_map) => recipe_map,
//     //     Err(e) => panic!("Unable to read recipe map: {}", e),
//     // };
//     // match recipe_map["identifiers"].get(identifier) {
//     //     Some(path) => path.to_string(),
//     //     None => panic!("Identifier {identifier} not found in recipe map!"),
//     // }
//     let recipe_map = read_recipe_map()?;
//     recipe_map["identifiers"]
//         .get(identifier)
//         .ok_or("Identifier not found in recipe map")
// }

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    /// Create a default recipe structure with test data only
    ///
    /// This is only used for tests
    fn create_default_test_recipe() -> Recipe {
        Recipe::new(
            "test".to_string(),
            "com.github.autopkg.test".to_string(),
            "3.0".to_string(),
            Some("com.github.autopkg.test.parent".to_string()),
        )
    }

    #[test]
    fn test_recipe_has_parent() {
        // The default recipe has a prepopulated parent recipe
        let mut test_recipe: Recipe = create_default_test_recipe();
        assert!(test_recipe.has_parent());
        // An empty string does not qualify as a parent
        test_recipe.parent_recipe = Some("".to_string());
        assert!(!test_recipe.has_parent());
        // A None value does not qualify as a parent
        test_recipe.parent_recipe = None;
        assert!(!test_recipe.has_parent());
    }

    #[test]
    fn test_recipe_is_valid() {
        // The default recipe should always be valid
        let mut test_recipe: Recipe = create_default_test_recipe();
        assert!(test_recipe.is_valid_recipe());
        // To be valid, static strings must all be defined,
        // so an empty string in any of them should be invalid
        test_recipe.description = "".to_string();
        assert!(!test_recipe.is_valid_recipe());
        // set Description back, empty identifier
        test_recipe.description = "stuff".to_string();
        test_recipe.identifier = "".to_string();
        assert!(!test_recipe.is_valid_recipe());
        // set Identifier back, empty minimum version
        test_recipe.identifier = "stuff".to_string();
        test_recipe.minimum_version = "".to_string();
        assert!(!test_recipe.is_valid_recipe());
        // Input must contain "NAME" key, if we delete it, invalid recipe
        test_recipe.input.remove("NAME");
        assert!(!test_recipe.is_valid_recipe());
    }

    #[test]
    fn test_get_string_key_from_recipe_value() {
        // Create a copy of GoogleChrome.download.recipe as a plist string
        let plist_string = "
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
        <plist version=\"1.0\">
        <dict>
            <key>Description</key>
            <string>Downloads latest Google Chrome enterprise package.</string>
            <key>Identifier</key>
            <string>com.github.autopkg.download.googlechromepkg</string>
            <key>Input</key>
            <dict>
                <key>NAME</key>
                <string>GoogleChrome</string>
                <key>DOWNLOAD_URL</key>
                <string>https://dl.google.com/chrome/mac/stable/accept_tos%3Dhttps%253A%252F%252Fwww.google.com%252Fintl%252Fen_ph%252Fchrome%252Fterms%252F%26_and_accept_tos%3Dhttps%253A%252F%252Fpolicies.google.com%252Fterms/googlechrome.pkg</string>
            </dict>
            <key>MinimumVersion</key>
            <string>1.4.1</string>
            <key>Process</key>
            <array>
                <dict>
                    <key>Processor</key>
                    <string>URLDownloader</string>
                    <key>Arguments</key>
                    <dict>
                        <key>url</key>
                        <string>%DOWNLOAD_URL%</string>
                        <key>filename</key>
                        <string>GoogleChrome.pkg</string>
                    </dict>
                </dict>
                <dict>
                    <key>Processor</key>
                    <string>EndOfCheckPhase</string>
                </dict>
                <dict>
                    <key>Arguments</key>
                    <dict>
                        <key>expected_authority_names</key>
                        <array>
                            <string>Developer ID Installer: Google LLC (EQHXZ8M8AV)</string>
                            <string>Developer ID Certification Authority</string>
                            <string>Apple Root CA</string>
                        </array>
                        <key>input_path</key>
                        <string>%pathname%</string>
                    </dict>
                    <key>Processor</key>
                    <string>CodeSignatureVerifier</string>
                </dict>
            </array>
        </dict>
        </plist>";
        // This sets up a seekable reader from a string
        let seekable_plist = Cursor::new(plist_string);
        // Read in the value from our Cursor, which acts like reading from a file
        let recipe_data = Value::from_reader(seekable_plist).unwrap();
        // We should be able to extract the specific strings we want
        assert_eq!(
            get_string_key_from_recipe_value(&recipe_data, "Identifier"),
            "com.github.autopkg.download.googlechromepkg"
        );
        assert_eq!(
            get_string_key_from_recipe_value(&recipe_data, "MinimumVersion"),
            "1.4.1"
        );
    }

    #[test]
    #[should_panic]
    fn test_get_string_key_from_recipe_value_should_fail() {
        // Do the same thing as the above test, except reading a non-string should fail
        let plist_string = "
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
        <plist version=\"1.0\">
        <dict>
            <key>Description</key>
            <string>Downloads latest Google Chrome enterprise package.</string>
            <key>Identifier</key>
            <string>com.github.autopkg.download.googlechromepkg</string>
            <key>Input</key>
            <dict>
                <key>NAME</key>
                <string>GoogleChrome</string>
                <key>DOWNLOAD_URL</key>
                <string>https://dl.google.com/chrome/mac/stable/accept_tos%3Dhttps%253A%252F%252Fwww.google.com%252Fintl%252Fen_ph%252Fchrome%252Fterms%252F%26_and_accept_tos%3Dhttps%253A%252F%252Fpolicies.google.com%252Fterms/googlechrome.pkg</string>
            </dict>
            <key>MinimumVersion</key>
            <string>1.4.1</string>
            <key>Process</key>
            <array>
                <dict>
                    <key>Processor</key>
                    <string>URLDownloader</string>
                    <key>Arguments</key>
                    <dict>
                        <key>url</key>
                        <string>%DOWNLOAD_URL%</string>
                        <key>filename</key>
                        <string>GoogleChrome.pkg</string>
                    </dict>
                </dict>
                <dict>
                    <key>Processor</key>
                    <string>EndOfCheckPhase</string>
                </dict>
                <dict>
                    <key>Arguments</key>
                    <dict>
                        <key>expected_authority_names</key>
                        <array>
                            <string>Developer ID Installer: Google LLC (EQHXZ8M8AV)</string>
                            <string>Developer ID Certification Authority</string>
                            <string>Apple Root CA</string>
                        </array>
                        <key>input_path</key>
                        <string>%pathname%</string>
                    </dict>
                    <key>Processor</key>
                    <string>CodeSignatureVerifier</string>
                </dict>
            </array>
        </dict>
        </plist>";
        let seekable_plist = Cursor::new(plist_string);
        let recipe_data = Value::from_reader(seekable_plist).unwrap();
        // This only can parse strings, so pulling a non-string from the plist should
        // panic
        get_string_key_from_recipe_value(&recipe_data, "Process");
    }

    #[test]
    fn test_calculate_short_name() {
        assert_eq!(
            "MyRecipe.download",
            calculate_short_name(Path::new("/Path/test/MyRecipe.download.recipe"))
        )
    }
}
