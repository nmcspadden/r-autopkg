use anyhow::Result;
use std::fmt;
use std::path::Path;
use std::{collections::HashMap, fs, path::PathBuf};
use tracing::debug;

use serde::{Deserialize, Serialize};

pub mod constants;

/// The Preferences object used to handle all AutoPkg preferences
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Preferences {
    // The serde(default) macros here are to allow a JSON prefs file to only
    // express a subset of the total preferences. Any value not specified here
    // will instead be assigned its default. Because of how the serde macro
    // works, the default must be the name of a function that gets called.
    // https://serde.rs/field-attrs.html#default--path
    // The only preference option that must be specified is the
    // recipe_search_dirs. It must be stored somewhere mutable.
    /// List of directories to search when building a recipe map
    pub recipe_search_dirs: Vec<PathBuf>,
    /// Parent folder for AutoPkg's cached downloads
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
    /// Parent folder for all overrides
    #[serde(default = "default_recipe_override_dir")]
    pub recipe_override_dir: PathBuf,
    /// Parent folder that new Recipe Repos will be added to
    #[serde(default = "default_recipe_repo_dir")]
    pub recipe_repo_dir: PathBuf,
    /// Path to a text file containing a GitHub API/access token
    #[serde(default = "default_github_token_path")]
    pub github_token_path: PathBuf,
    /// Path to recipe map JSON file
    #[serde(default = "default_recipe_map_path")]
    pub recipe_map_path: PathBuf,
    /// Optional path to a Munki repo
    pub munki_repo: Option<PathBuf>,
    /// Whether code signature verification should be disabled.
    #[serde(default = "default_disable_code_signature_verification")]
    pub disable_code_signature_verification: bool,
    /// Path to preferences file
    #[serde(default = "default_prefs_path", skip)] // don't write this back to the prefs file
    pub prefs_path: PathBuf,
    /// Any extra keys can be added in and used within recipes or Processors.
    /// These are not used by any native/built-in AutoPkg functions
    pub extras: Option<HashMap<String, String>>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Preferences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Current AutoPkg preferences: ")?;
        // We're just going to hardcode each of the field names in
        // SCREAMING_SNAKE_CASE
        writeln!(f, "RECIPE_SEARCH_DIRS: ")?;
        for line in &self.recipe_search_dirs {
            writeln!(f, "    {}", line.display())?;
        }
        writeln!(f)?;
        writeln!(f, "CACHE_DIR: ")?;
        writeln!(f, "    {}", self.cache_dir.display())?;
        writeln!(f)?;
        writeln!(f, "RECIPE_OVERRIDE_DIR: ")?;
        writeln!(f, "    {}", self.recipe_override_dir.display())?;
        writeln!(f)?;
        writeln!(f, "RECIPE_REPO_DIR: ")?;
        writeln!(f, "    {}", self.recipe_repo_dir.display())?;
        writeln!(f)?;
        writeln!(f, "GITHUB_TOKEN_PATH: ")?;
        writeln!(f, "    {}", self.github_token_path.display())?;
        writeln!(f)?;
        writeln!(f, "RECIPE_MAP_PATH: ")?;
        writeln!(f, "    {}", self.recipe_map_path.display())?;
        if self.munki_repo.is_some() {
            writeln!(f)?;
            writeln!(f, "MUNKI_REPO: ")?;
            writeln!(f, "    {}", self.munki_repo.as_ref().unwrap().display())?;
        }
        writeln!(f)?;
        writeln!(f, "DISABLE_CODE_SIGNATURE_VERIFICATION: ")?;
        writeln!(f, "    {}", self.disable_code_signature_verification)?;
        if self.extras.is_some() {
            writeln!(f)?;
            writeln!(f, "EXTRA KEYS: ")?;
            for (key, value) in self.extras.as_ref().unwrap().iter() {
                writeln!(f, "    {:>20}: {:<10}", key, value)?;
            }
        }
        Ok(())
    }
}

impl Preferences {
    pub fn new() -> Preferences {
        Preferences {
            recipe_search_dirs: vec![PathBuf::from(".")],
            cache_dir: default_cache_dir(),
            recipe_override_dir: default_recipe_override_dir(),
            recipe_repo_dir: default_recipe_repo_dir(),
            github_token_path: default_github_token_path(),
            recipe_map_path: default_recipe_map_path(),
            disable_code_signature_verification: default_disable_code_signature_verification(),
            prefs_path: default_prefs_path(),
            munki_repo: None,
            extras: None,
        }
    }

    /// Read in the JSON preferences file and return a Preferences object
    pub fn read_from_disk(&self, path: &Path) -> Result<Preferences> {
        // Reading the file into a string first is significantly faster than
        // reading directly from a reader: https://github.com/serde-rs/json/issues/160
        let json_data = fs::read_to_string(path)?;
        let prefs: Preferences = serde_json::from_str(&json_data)?;
        Ok(prefs)
    }

    /// Append a path to the search dirs and write out to preferences
    pub fn add_to_search_dirs(&mut self, path: &Path) -> Result<()> {
        self.recipe_search_dirs.push(path.to_path_buf());
        self.write_to_disk()?;
        Ok(())
    }

    /// Remove a path from the search dirs and write out to preferences
    pub fn remove_from_search_dirs(&mut self, path: &Path) -> Result<()> {
        debug!("Search dir to remove: {}", path.display());
        // Remove all instances of matching path from the vec. This must be an
        // exact match for it to work; does nothing if not found.
        // debug!("Search dirs before retain: {:?}", &self.recipe_search_dirs);
        self.recipe_search_dirs.retain(|s| *s != path);
        // We always write out even if we didn't find a match.
        // debug!("Search dirs after retain: {:?}", &self.recipe_search_dirs);
        self.write_to_disk()?;
        Ok(())
    }

    /// Write the preferences out to disk
    /// For now, this only supports JSON
    pub fn write_to_disk(&self) -> Result<(), std::io::Error> {
        std::fs::write(
            &self.prefs_path,
            serde_json::to_string_pretty(self).unwrap(),
        )
    }
}

fn default_recipe_repo_dir() -> PathBuf {
    constants::DEFAULT_RECIPE_REPOS_DIR.to_path_buf()
}

fn default_recipe_override_dir() -> PathBuf {
    constants::DEFAULT_OVERRIDES_DIR.to_path_buf()
}

fn default_github_token_path() -> PathBuf {
    constants::DEFAULT_GH_TOKEN_PATH.to_path_buf()
}

fn default_cache_dir() -> PathBuf {
    constants::DEFAULT_CACHE_DIR.to_path_buf()
}

fn default_recipe_map_path() -> PathBuf {
    constants::DEFAULT_RECIPE_MAP.to_path_buf()
}

fn default_disable_code_signature_verification() -> bool {
    false
}

fn default_prefs_path() -> PathBuf {
    constants::PREFERENCES_PATH.to_path_buf()
}
