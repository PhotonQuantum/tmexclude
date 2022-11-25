//! Defines all needed configs and views to them.
//!
//! The config is synchronized by design so it can be hot-reloaded.
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::ErrorKind;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, iter};

use directories::BaseDirs;
use itertools::Itertools;
use log::warn;
use maplit::hashset;
use serde::{Deserialize, Deserializer, Serialize};
use tap::TapFallible;
use ts_rs::TS;

use crate::error::{ConfigError, ConfigIOError};

/// Main config type used throughout the application.
#[derive(Debug, Clone)]
pub struct Config {
    /// Do not include files to backups if conditions are not met. Defaults to `false`.
    pub no_include: bool,
    /// Configs related to walking, including interested directories and corresponding rules.
    pub walk: Arc<WalkConfig>,
}

impl TryFrom<PreConfig> for Config {
    type Error = ConfigError;

    fn try_from(value: PreConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            no_include: value.no_include,
            walk: Arc::new(WalkConfig::from(
                value.directories,
                &value.rules,
                value.skips,
            )?),
        })
    }
}

impl Config {
    // TODO remove this function
    /// Load config from deserializer.
    ///
    /// # Errors
    /// `Deserializer` if error occurs when deserializing config.
    /// `Rule` if rule name is referenced but not defined.
    /// `NotADirectory` if there's directory given but not found.
    pub fn from<'de>(deserializer: impl Deserializer<'de>) -> Result<Self, ConfigError> {
        let pre_config = PreConfig::from(deserializer)?;
        Ok(Self {
            no_include: pre_config.no_include,
            walk: Arc::new(WalkConfig::from(
                pre_config.directories,
                &pre_config.rules,
                pre_config.skips,
            )?),
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
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default, TS)]
#[ts(export, export_to = "../src/bindings/")]
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

fn dfs_union_rules<'a, 'b>(
    cache: &mut HashMap<String, HashSet<Rule>>,
    rules: &'a HashMap<String, PreRule>,
    node: &'b str,
    mut visited: HashSet<&'b str>,
) -> Result<HashSet<Rule>, ConfigError> {
    if let Some(hit) = cache.get(node) {
        return Ok(hit.clone());
    }
    if visited.contains(node) {
        return Err(ConfigError::Loop(node.to_string()));
    }

    let resolved = rules
        .get(node)
        .ok_or_else(|| ConfigError::Rule(node.to_string()))?;
    match resolved {
        PreRule::Concrete(rule) => Ok(hashset![rule.clone()]),
        PreRule::Union(referenced) => {
            visited.insert(node);
            referenced
                .iter()
                .map(|node| dfs_union_rules(cache, rules, node, visited.clone()))
                .try_fold(hashset![/* can have rules later */], |mut acc, x| {
                    x.map(|x| {
                        acc.extend(x);
                        acc
                    })
                })
        }
    }
    .tap_ok(|result| {
        cache.insert(node.to_string(), result.clone()); // avoid to_string?
    })
}

fn follow_symlinks(path: PathBuf) -> impl Iterator<Item = PathBuf> {
    let mut visited = hashset![path.clone()];
    iter::successors(Some(path), move |path| {
        path.read_link()
            .map(|parts| path.parent().unwrap_or(path).join(parts))
            .tap_err(|e| match e.kind() {
                ErrorKind::NotFound | ErrorKind::InvalidInput => (),
                _ => warn!("Error when following symlink: {}", e),
            })
            .ok()
            .and_then(|path| {
                if visited.contains(&path) {
                    warn!("Cyclic symlink detected");
                    None
                } else {
                    visited.insert(path.clone());
                    Some(path)
                }
            })
    })
}

fn absolute(path: impl AsRef<Path>) -> PathBuf {
    std::env::current_dir()
        .expect("current dir must exist")
        .join(path)
}

impl WalkConfig {
    fn from(
        directories: Vec<PreDirectory>,
        rules: &HashMap<String, PreRule>,
        skips: Vec<String>,
    ) -> Result<Self, ConfigError> {
        let mut cache = HashMap::new();
        Ok(Self {
            directories: directories
                .into_iter()
                .map(|pre_directory| {
                    // start parsing rule names into rules
                    pre_directory
                        .rules
                        .into_iter()
                        .map(|rule_name| {
                            // try to get rule by dfs
                            dfs_union_rules(&mut cache, rules, rule_name.as_str(), hashset![])
                        })
                        .try_fold(vec![], |mut acc, x| {
                            x.map(|x| {
                                acc.extend(x.into_iter());
                                acc
                            })
                        })
                        .and_then(|rules| {
                            PathBuf::from(shellexpand::tilde(&pre_directory.path).as_ref())
                                .canonicalize() // canonicalize here because fsevent api always returns absolute paths
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
                .flat_map(|path| follow_symlinks(PathBuf::from(shellexpand::tilde(&path).as_ref())))
                .map(absolute)
                .collect(),
        })
    }

    /// Get common root of all directories.
    ///
    /// # Errors
    /// `ConfigError` if there's no scanning directory specified in the config.
    pub fn root(&self) -> Result<PathBuf, ConfigError> {
        get_root(&self.directories).ok_or(ConfigError::NoDirectory)
    }

    /// Squash nested directory paths.
    #[must_use]
    pub fn paths(&self) -> HashSet<&Path> {
        get_paths(&self.directories)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
#[serde(rename_all = "kebab-case")]
pub struct PreConfig {
    #[serde(default)]
    no_include: bool,
    #[serde(default)]
    directories: Vec<PreDirectory>,
    #[serde(default)]
    skips: Vec<String>,
    #[serde(default)]
    rules: HashMap<String, PreRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
pub struct PreDirectory {
    path: String,
    rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/bindings/")]
#[serde(untagged)]
pub enum PreRule {
    Concrete(Rule),
    Union(Vec<String>),
}

impl PreConfig {
    fn from<'de>(deserializer: impl Deserializer<'de>) -> Result<Self, ConfigError> {
        Self::deserialize(deserializer)
            .map_err(|e| ConfigError::Deserialize(Box::new(AdhocError(e.to_string()))))
    }
}

#[derive(Debug)]
struct AdhocError(String);

impl Display for AdhocError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for AdhocError {}

const DEFAULT_CONFIG: &str = include_str!("../../../config.example.yaml");

#[derive(Debug, Clone)]
pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, ConfigIOError> {
        let config_dir = BaseDirs::new()
            .ok_or_else(|| ConfigIOError::MissingHome)?
            .home_dir()
            .join(".config");
        fs::create_dir_all(&config_dir).map_err(ConfigIOError::CreateConfigDir)?;

        let path = config_dir.join("tmexclude.yaml");
        if !path.exists() {
            fs::write(&path, DEFAULT_CONFIG).map_err(ConfigIOError::WriteConfig)?;
        }

        Ok(Self { path })
    }
    pub fn load(&self) -> Result<PreConfig, ConfigIOError> {
        let content = fs::read_to_string(&self.path).map_err(ConfigIOError::ReadConfig)?;
        serde_yaml::from_str(&content).map_err(|e| ConfigIOError::Deserialize(Box::new(e)))
    }
    pub fn save(&self, config: &PreConfig) -> Result<(), ConfigIOError> {
        let content =
            serde_yaml::to_string(config).map_err(|e| ConfigIOError::Serialize(Box::new(e)))?;
        fs::write(&self.path, content).map_err(ConfigIOError::WriteConfig)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use std::{env, fs};

    use itertools::Itertools;
    use maplit::hashset;

    use crate::config::{get_paths, get_root, Config, Directory, Rule, WalkConfig};
    use crate::error::ConfigError;

    macro_rules! path {
        ($s: expr) => {
            PathBuf::from_str($s).unwrap()
        };
    }
    macro_rules! cwd_path {
        ($s: expr) => {
            std::env::current_dir()
                .unwrap()
                .canonicalize()
                .unwrap()
                .join($s)
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
        with_directory(|| {
            let config = Config::from(serde_yaml::Deserializer::from_str(include_str!(
                "../../tests/configs/simple.yaml"
            )))
            .expect("must parse config");

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

            assert!(config.no_include);
            assert_eq!(
                config.walk,
                Arc::new(WalkConfig {
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
                })
            );
        });
    }

    #[test]
    fn must_test_inherit_rule() {
        with_directory(|| {
            let config = Config::from(serde_yaml::Deserializer::from_str(include_str!(
                "../../tests/configs/inherit_rule.yaml"
            )))
            .expect("must parse config");

            let directories_rules = config
                .walk
                .directories
                .clone()
                .into_iter()
                .map(|directory| {
                    directory
                        .rules
                        .into_iter()
                        .map(|rule| rule.excludes[0].clone())
                        .collect::<HashSet<_>>()
                })
                .collect_vec();
            assert_eq!(
                directories_rules,
                [
                    hashset![path!("a"), path!("c"), path!("d")],
                    hashset![path!("a"), path!("c"), path!("d"), path!("e")]
                ]
            );
        });
    }

    #[test]
    fn must_fail_inherit_rule_loop() {
        let error = Config::from(serde_yaml::Deserializer::from_str(include_str!(
            "../../tests/configs/inherit_rule_loop.yaml"
        )))
        .expect_err("must fail");

        match error {
            ConfigError::Loop(path) => assert_eq!(path, "a"),
            _ => panic!("Error type mismatch"),
        }
    }

    #[test]
    fn must_fail_broken_rule() {
        let error = Config::from(serde_yaml::Deserializer::from_str(include_str!(
            "../../tests/configs/broken_rule.yaml"
        )))
        .expect_err("must fail");

        match error {
            ConfigError::Rule(path) => assert_eq!(path, "a"),
            _ => panic!("Error type mismatch"),
        }
    }

    #[test]
    fn must_fail_broken_dir() {
        with_directory(|| {
            let error = Config::from(serde_yaml::Deserializer::from_str(include_str!(
                "../../tests/configs/broken_dir.yaml"
            )))
            .expect_err("must fail");

            match error {
                ConfigError::InvalidPath { path, source } => {
                    assert_eq!(path, "tests/mock_dirs/some_file");
                    assert_eq!(source.to_string(), "not a directory");
                }
                _ => panic!("Error type mismatch"),
            }
        });
    }

    #[test]
    fn must_fail_missing_dir() {
        with_directory(|| {
            let error = Config::from(serde_yaml::Deserializer::from_str(include_str!(
                "../../tests/configs/missing_dir.yaml"
            )))
            .expect_err("must fail");

            match error {
                ConfigError::InvalidPath { path, source } => {
                    assert_eq!(path, "tests/mock_dirs/non_exist");
                    assert_eq!(source.kind(), ErrorKind::NotFound);
                }
                _ => panic!("Error type mismatch"),
            }
        });
    }

    #[test]
    fn must_allow_missing_skip_dir() {
        with_directory(|| {
            let config = Config::from(serde_yaml::Deserializer::from_str(include_str!(
                "../../tests/configs/allow_missing_skip_dir.yaml"
            )))
            .expect("must parse config");

            assert_eq!(
                config.walk.skips,
                hashset![cwd_path!("tests/mock_dirs/non_exist")]
            );
        });
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

    #[test]
    fn must_canonicalize_rule_follow_skip() {
        with_directory(|| {
            let config = Config::from(serde_yaml::Deserializer::from_str(include_str!(
                "../../tests/configs/follow_symlink.yaml"
            )))
            .expect("must parse config");

            assert_eq!(
                config.walk.skips,
                hashset![
                    cwd_path!("tests/symlinks/three"),
                    cwd_path!("tests/symlinks/two"),
                    cwd_path!("tests/symlinks/one"),
                    cwd_path!("tests/symlinks/concrete"),
                    cwd_path!("tests/symlinks/invalid"),
                    cwd_path!("tests/symlinks/missing"),
                    cwd_path!("tests/symlinks/cyclic_a"),
                    cwd_path!("tests/symlinks/cyclic_b"),
                ]
            );
        });
    }

    fn with_directory(f: impl FnOnce()) {
        static LOCK: Mutex<()> = Mutex::new(());
        let temp_dir = tempfile::TempDir::new().unwrap();
        let base_path = temp_dir.path().canonicalize().unwrap();
        let path = base_path.join("tests");

        fs::create_dir(&path).unwrap();

        fs::create_dir(path.join("mock_dirs")).unwrap();
        fs::create_dir(path.join("mock_dirs").join("path_a")).unwrap();
        fs::create_dir(path.join("mock_dirs").join("path_b")).unwrap();
        fs::write(path.join("mock_dirs").join("some_file"), []).unwrap();

        fs::create_dir(path.join("symlinks")).unwrap();
        symlink(
            path.join("symlinks").join("concrete"),
            path.join("symlinks").join("one"),
        );
        symlink(
            path.join("symlinks").join("two"),
            path.join("symlinks").join("three"),
        );
        symlink(
            path.join("symlinks").join("one"),
            path.join("symlinks").join("two"),
        );
        symlink(
            path.join("symlinks").join("missing"),
            path.join("symlinks").join("invalid"),
        );
        symlink(
            path.join("symlinks").join("cyclic_b"),
            path.join("symlinks").join("cyclic_a"),
        );
        symlink(
            path.join("symlinks").join("cyclic_a"),
            path.join("symlinks").join("cyclic_b"),
        );

        let _guard = LOCK.lock().unwrap();
        let original_cwd = env::current_dir().unwrap();
        env::set_current_dir(base_path).unwrap();
        f();
        env::set_current_dir(original_cwd).unwrap();
    }

    fn symlink(original: impl AsRef<Path>, link: impl AsRef<Path>) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(original, link).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(original, link).unwrap();
        }
        #[cfg(not(any(unix, windows)))]
        {
            panic!("Unsupported platform");
        }
    }
}
