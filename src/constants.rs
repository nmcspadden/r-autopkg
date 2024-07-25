use once_cell::sync::Lazy;
use std::path::PathBuf;

const TOP_DIR_NAME: &str = "AutoPkg";
const RECIPES_DIR_NAME: &str = "Recipes";
const RECIPE_REPO_DIR_NAME: &str = "RecipeRepos";
const RECIPE_OVERRIDES_NAME: &str = "RecipeOverrides";
const CACHE_DIR_NAME: &str = "Cache";
const RECIPE_MAP_FILENAME: &str = "recipe_map.json";
const GH_TOKEN_FILENAME: &str = "gh_token";
pub const REPO_LIST_FILENAME: &str = "repo_list.json";
const PREFERENCES_FILENAME: &str = "autopkg_prefs.json";
const REPO_MAP_FILENAME: &str = "repo_map.json";
pub const GITHUB_ORG_NAME: &str = "autopkg";

// Why are we using Lazy statics here instead of just constant strings?
//
// It turns out that expanding env variables/shell strings is surprisingly
// hard to do correctly, and then converting it into a Path or PathBuf later
// is frought with peril. The ultimate goal here is to use the built-in OS
// config directories correctly, which the `dirs` crate provides.
// So instead, we're creating Lazy static PathBufs that are constructed at
// runtime.

pub static DEFAULT_LIBRARY_DIR: Lazy<PathBuf> = Lazy::new(|| {
    [dirs::config_dir().unwrap(), PathBuf::from(TOP_DIR_NAME)]
        .iter()
        .collect()
});
// pub const DEFAULT_LIBRARY_DIR: &str = "%PROGRAMDATA%/AutoPkg";
// pub const DEFAULT_LIBRARY_DIR: &str = "/Library/Application Support/AutoPkg";

pub static USER_LIBRARY_DIR: Lazy<PathBuf> = Lazy::new(|| {
    [dirs::config_dir().unwrap(), PathBuf::from(TOP_DIR_NAME)]
        .iter()
        .collect()
});
// pub const USER_LIBRARY_DIR: &str = "%APPDATA%/AutoPkg";
// pub const USER_LIBRARY_DIR: &str = "~/Library/Application Support/AutoPkg";

pub static USER_RECIPES_DIR: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(RECIPES_DIR_NAME),
    ]
    .iter()
    .collect()
});
// pub const USER_RECIPES_DIR: &str = "%APPDATA%/AutoPkg/Recipes";
// pub const USER_RECIPES_DIR: &str = "~/Library/Application Support/AutoPkg/Recipes";

pub static DEFAULT_CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(CACHE_DIR_NAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_CACHE_DIR: &str = "%APPDATA%/AutoPkg/Cache";
// pub const DEFAULT_CACHE_DIR: &str = "~/Library/Application Support/AutoPkg/Cache";

pub static DEFAULT_RECIPE_REPOS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(RECIPE_REPO_DIR_NAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_RECIPE_REPOS_DIR: &str = "%APPDATA%/AutoPkg/RecipeRepos";
// pub const DEFAULT_RECIPE_REPOS_DIR: &str = "~/Library/Application Support/AutoPkg/RecipeRepos";

pub static DEFAULT_OVERRIDES_DIR: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(RECIPE_OVERRIDES_NAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_OVERRIDES_DIR: &str = "%APPDATA%/AutoPkg/RecipeOverrides";
// pub const DEFAULT_OVERRIDES_DIR: &str = "~/Library/Application Support/AutoPkg/RecipeOverrides";

pub static DEFAULT_RECIPE_MAP: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(RECIPE_MAP_FILENAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_RECIPE_MAP: &str = "%APPDATA%/AutoPkg/recipe_map.json";
// pub const DEFAULT_RECIPE_MAP: &str = "~/Library/Application Support/AutoPkg/recipe_map.json";

pub static DEFAULT_GH_TOKEN_PATH: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(GH_TOKEN_FILENAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_RECIPE_MAP: &str = "%APPDATA%/AutoPkg/gh_token";
// pub const DEFAULT_RECIPE_MAP: &str = "~/Library/Application Support/AutoPkg/gh_token";

pub static PREFERENCES_PATH: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(PREFERENCES_FILENAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_RECIPE_MAP: &str = "%APPDATA%/AutoPkg/autopkg_prefs.json";
// pub const DEFAULT_RECIPE_MAP: &str = "~/Library/Application Support/AutoPkg/autopkg_prefs.json";

pub static REPO_MAP_PATH: Lazy<PathBuf> = Lazy::new(|| {
    [
        dirs::config_dir().unwrap(),
        PathBuf::from(TOP_DIR_NAME),
        PathBuf::from(RECIPE_REPO_DIR_NAME),
        PathBuf::from(REPO_MAP_FILENAME),
    ]
    .iter()
    .collect()
});
// pub const DEFAULT_RECIPE_MAP: &str = "%APPDATA%/AutoPkg/RecipeRepos/repo_map.json";
// pub const DEFAULT_RECIPE_MAP: &str = "~/Library/Application Support/AutoPkg/RecipeRepos/repo_map.json";
