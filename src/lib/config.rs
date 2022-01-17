//! Defines all needed configs and views to them.
//!
//! The config is synchronized by design so it can be hot-reloaded.
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::ErrorKind;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use figment::{Figment, Provider};
use itertools::Itertools;
use log::warn;
use serde::Deserialize;
use tap::TapFallible;

use crate::errors::ConfigError;

/// Main config type used throughout the application.
#[derive(Debug, Clone)]
pub struct Config {
    /// The apply mode this instance is working on.
    pub mode: ApplyMode,
    /// Configs related to walking, including interested directories and corresponding rules.
    pub walk: WalkConfig,
}

/// The apply mode for `TimeMachine` operations.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApplyMode {
    /// Don't touch the system.
    DryRun,
    /// Only add exclusions.
    AddOnly,
    /// Add and remove exclusions.
    All,
}

impl Default for ApplyMode {
    fn default() -> Self {
        Self::AddOnly
    }
}

impl Config {
    /// Load config from provider.
    ///
    /// # Errors
    /// `Figment` if error occurs when collecting config.
    /// `Rule` if rule name is referenced but not defined.
    /// `NotADirectory` if there's directory given but not found.
    pub fn from(provider: impl Provider) -> Result<Self, ConfigError> {
        let pre_config = PreConfig::from(provider)?;
        Ok(Self {
            mode: pre_config.mode,
            walk: WalkConfig::from(pre_config.directories, &pre_config.rules, pre_config.skips)?,
        })
    }
}

/// Configs related to walking, including interested directories and corresponding rules.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct WalkConfig {
    /// Interested directories and corresponding rules.
    pub directories: Vec<Directory>,
    /// Directories to be skipped when scanning and watching.
    pub skips: HashSet<PathBuf>,
}

/// An interested directory and its corresponding rules.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Directory {
    /// The interested directory.
    pub path: PathBuf,
    /// Rules bound to this directory.
    pub rules: Vec<Rule>,
}

/// Rules to be applied on a specific set of directories.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Rule {
    /// Paths to be excluded.
    pub excludes: Vec<PathBuf>,
    /// Exclude paths if *any* of these paths exist in the same directory as the path to be excluded.
    #[serde(default)]
    pub if_exists: Vec<PathBuf>,
}

fn max_common_path(path_1: impl AsRef<Path>, path_2: impl AsRef<Path>) -> PathBuf {
    let max_common_path = path_1
        .as_ref()
        .components()
        .zip(path_2.as_ref().components())
        .try_fold(vec![], |mut prev, (l, r)| {
            if l == r {
                prev.push(l);
                ControlFlow::Continue(prev)
            } else {
                ControlFlow::Break(prev)
            }
        });
    match max_common_path {
        ControlFlow::Continue(components) | ControlFlow::Break(components) => {
            components.into_iter().collect()
        }
    }
}

/// Squash nested directory paths.
fn get_paths(directories: &[Directory]) -> HashSet<&Path> {
    directories
        .iter()
        .map(|item| &item.path)
        // Sort paths by length so that child paths appear later than their ancestor.
        .sorted_unstable_by_key(|path| path.as_os_str().len())
        .fold(HashSet::new(), |mut acc, x| {
            if !x.ancestors().any(|ancestor| acc.contains(ancestor)) {
                // No ancestors of this path is in the set.
                acc.insert(x.as_path());
            }
            acc
        })
}

/// Get common root of all directories.
fn get_root(directories: &[Directory]) -> Option<PathBuf> {
    directories
        .iter()
        .map(|item| &item.path)
        .fold(None, |acc, x| {
            acc.map_or_else(|| Some(x.clone()), |acc| Some(max_common_path(acc, x)))
        })
}

impl WalkConfig {
    fn from(
        directories: Vec<PreDirectory>,
        rules: &HashMap<String, Rule>,
        skips: Vec<String>,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            directories: directories
                .into_iter()
                .map(|pre_directory| {
                    // start parsing rule names into rules
                    pre_directory
                        .rules
                        .into_iter()
                        .map(|rule_name| {
                            // try get rule by name
                            rules
                                .get(rule_name.as_str())
                                .cloned()
                                .ok_or(ConfigError::Rule(rule_name))
                        })
                        .try_collect()
                        .and_then(|rules| {
                            PathBuf::from(shellexpand::tilde(&pre_directory.path).as_ref())
                                .canonicalize()
                                .map_err(|e| ConfigError::InvalidPath {
                                    path: pre_directory.path.clone(),
                                    source: e,
                                })
                                .and_then(|path| {
                                    path.is_dir().then(|| path.clone()).ok_or(
                                        ConfigError::InvalidPath {
                                            path: pre_directory.path.clone(),
                                            // TODO change this to ErrorKind::NotADirectory once `io_error_more` is stabilized.
                                            source: std::io::Error::new(
                                                ErrorKind::Other,
                                                "not a directory",
                                            ),
                                        },
                                    )
                                })
                                .map(|path| (path, rules))
                        })
                        .map(|(path, rules)| Directory { path, rules })
                })
                .try_collect()?,
            skips: skips
                .into_iter()
                .filter_map(|path| {
                    PathBuf::from(shellexpand::tilde(&path).as_ref())
                        .canonicalize()
                        .tap_err(|e| {
                            warn!("Error when parsing skipped path {}: {}", path, e);
                        })
                        .ok()
                })
                .collect(),
        })
    }

    /// Get common root of all directories.
    ///
    /// # Errors
    /// `ConfigError` if there's no scanning directory specified in the config.
    pub fn root(&self) -> Result<PathBuf, ConfigError> {
        get_root(&*self.directories).ok_or(ConfigError::NoDirectory)
    }

    /// Squash nested directory paths.
    #[must_use]
    pub fn paths(&self) -> HashSet<&Path> {
        get_paths(&*self.directories)
    }
}

#[derive(Deserialize)]
struct PreConfig {
    #[serde(default)]
    mode: ApplyMode,
    #[serde(default)]
    directories: Vec<PreDirectory>,
    #[serde(default)]
    skips: Vec<String>,
    #[serde(default)]
    rules: HashMap<String, Rule>,
}

#[derive(Deserialize)]
struct PreDirectory {
    path: String,
    rules: Vec<String>,
}

impl PreConfig {
    fn from(provider: impl Provider) -> Result<Self, figment::Error> {
        Figment::from(provider).extract()
    }
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

    use figment::providers::{Format, Yaml};
    use maplit::hashset;

    use crate::config::{get_paths, get_root, ApplyMode, Config, Directory, Rule, WalkConfig};
    use crate::errors::ConfigError;

    macro_rules! path {
        ($s: expr) => {
            PathBuf::from_str($s).unwrap()
        };
    }
    macro_rules! cwd_path {
        ($s: expr) => {
            std::env::current_dir().unwrap().join($s)
        };
    }

    macro_rules! directory {
        ($s: expr) => {
            Directory {
                path: path!($s),
                rules: vec![],
            }
        };
    }

    macro_rules! directories {
        ($( $s:expr ),*) => {
            [$(directory!($s)),*]
        }
    }

    #[test]
    fn must_parse_simple() {
        static SIMPLE: &str = include_str!("../../tests/configs/simple.yaml");
        let config = Config::from(Yaml::string(SIMPLE)).expect("must parse config");

        let rule_a = Rule {
            excludes: vec![path!("exclude_a")],
            if_exists: vec![],
        };
        let rule_b = Rule {
            excludes: vec![path!("exclude_b")],
            if_exists: vec![],
        };
        let rule_d = Rule {
            excludes: vec![path!("exclude_d1"), path!("exclude_d2")],
            if_exists: vec![path!("a"), path!("b")],
        };

        assert_eq!(config.mode, ApplyMode::DryRun);
        assert_eq!(
            config.walk,
            WalkConfig {
                directories: vec![
                    Directory {
                        path: cwd_path!("tests/mock_dirs/path_a"),
                        rules: vec![rule_a, rule_b.clone()],
                    },
                    Directory {
                        path: cwd_path!("tests/mock_dirs/path_b"),
                        rules: vec![rule_b, rule_d],
                    },
                ],
                skips: hashset![cwd_path!("tests/mock_dirs/path_b")],
            }
        );
    }

    #[test]
    fn must_fail_broken_rule() {
        static BROKEN: &str = include_str!("../../tests/configs/broken_rule.yaml");
        let provider = Yaml::string(BROKEN);
        let error = Config::from(provider).expect_err("must fail");
        match error {
            ConfigError::Rule(path) => assert_eq!(path, "a"),
            _ => panic!("Error type mismatch"),
        }
    }

    #[test]
    fn must_fail_broken_dir() {
        static BROKEN: &str = include_str!("../../tests/configs/broken_dir.yaml");
        let provider = Yaml::string(BROKEN);
        let error = Config::from(provider).expect_err("must fail");

        match error {
            ConfigError::InvalidPath { path, source } => {
                assert_eq!(path, "tests/mock_dirs/some_file");
                assert_eq!(source.to_string(), "not a directory");
            }
            _ => panic!("Error type mismatch"),
        }
    }

    #[test]
    fn must_fail_missing_dir() {
        static BROKEN: &str = include_str!("../../tests/configs/missing_dir.yaml");
        let provider = Yaml::string(BROKEN);
        let error = Config::from(provider).expect_err("must fail");

        match error {
            ConfigError::InvalidPath { path, source } => {
                assert_eq!(path, "tests/mock_dirs/non_exist");
                assert_eq!(source.kind(), ErrorKind::NotFound);
            }
            _ => panic!("Error type mismatch"),
        }
    }

    #[test]
    fn must_allow_missing_skip_dir() {
        static BROKEN: &str = include_str!("../../tests/configs/allow_missing_skip_dir.yaml");
        let provider = Yaml::string(BROKEN);
        assert_eq!(
            Config::from(provider).expect("must parse config").walk,
            WalkConfig::default()
        );
    }

    #[test]
    fn must_get_directory_root() {
        assert_eq!(
            get_root(&directories!["/a/b/c/d", "/a/b/c"]),
            Some(path!("/a/b/c"))
        );
        assert_eq!(
            get_root(&directories!["/a/e/a", "/a/c", "/a/c/d"]),
            Some(path!("/a"))
        );
        assert_eq!(get_root(&directories!["/a", "/b"]), Some(path!("/")));
    }

    #[test]
    fn must_get_squashed_paths() {
        assert_eq!(
            get_paths(&directories!["/a/b/c", "/a/b", "/a/b/d"]),
            hashset! {Path::new("/a/b")}
        );
        assert_eq!(
            get_paths(&directories!["/a/b/c", "/a/e", "/a/b/d"]),
            hashset! {Path::new("/a/b/c"), Path::new("/a/b/d"), Path::new("/a/e")}
        );
        assert_eq!(
            get_paths(&directories!["/e", "/a/b/c", "/a/e", "/a/b/d", "/a/b/d/e"]),
            hashset! {Path::new("/a/b/c"), Path::new("/a/b/d"), Path::new("/a/e"), Path::new("/e")}
        );
    }
}
