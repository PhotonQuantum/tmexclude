use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use figment::{Figment, Provider};
use itertools::Itertools;
use log::warn;
use serde::Deserialize;
use tap::TapFallible;

use crate::errors::ConfigError;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Config {
    pub directories: Vec<Directory>,
    pub skips: HashSet<PathBuf>,
}

/// A `CoW` view of [`Config`](Config).
pub struct ConfigView<'a> {
    pub directories: Cow<'a, [Directory]>,
    pub skips: Cow<'a, HashSet<PathBuf>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Directory {
    pub path: PathBuf,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Rule {
    pub excludes: Vec<PathBuf>,
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
            directories: pre_config
                .directories
                .into_iter()
                .map(|pre_directory| {
                    // start parsing rule names into rules
                    pre_directory
                        .rules
                        .into_iter()
                        .map(|rule_name| {
                            // try get rule by name
                            pre_config
                                .rules
                                .get(rule_name.as_str())
                                .cloned()
                                .ok_or(ConfigError::Rule(rule_name))
                        })
                        .try_collect()
                        .and_then(|rules| {
                            PathBuf::from(shellexpand::tilde(&pre_directory.path).as_ref())
                                .canonicalize()
                                .map_err(|_| ConfigError::NotFound(pre_directory.path.clone()))
                                .and_then(|path| {
                                    path.is_dir()
                                        .then(|| path.clone())
                                        .ok_or(ConfigError::NotADirectory(path))
                                })
                                .map(|path| (path, rules))
                        })
                        .map(|(path, rules)| {
                            // compose directory
                            Directory { path, rules }
                        })
                })
                .try_collect()?,
            skips: pre_config
                .skips
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
    #[must_use]
    pub fn root(&self) -> Option<PathBuf> {
        ConfigView::from(self).root()
    }
}

impl ConfigView<'_> {
    /// Get common root of all directories.
    #[must_use]
    pub fn root(&self) -> Option<PathBuf> {
        self.directories
            .iter()
            .map(|item| &item.path)
            .fold(None, |acc, x| {
                acc.map_or_else(|| Some(x.clone()), |acc| Some(max_common_path(acc, x)))
            })
    }

    pub fn into_owned(self) -> Config {
        Config {
            directories: self.directories.into_owned(),
            skips: self.skips.into_owned(),
        }
    }
}

impl From<Config> for ConfigView<'static> {
    fn from(c: Config) -> Self {
        Self {
            directories: c.directories.into(),
            skips: Cow::Owned(c.skips),
        }
    }
}

impl<'a> From<&'a Config> for ConfigView<'a> {
    fn from(c: &'a Config) -> Self {
        Self {
            directories: (&c.directories).into(),
            skips: Cow::Borrowed(&c.skips),
        }
    }
}

#[derive(Deserialize)]
struct PreConfig {
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
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::str::FromStr;

    use figment::providers::{Format, Yaml};
    use maplit::hashset;

    use crate::config::{Config, Directory, Rule};
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

    #[test]
    fn must_parse_simple() {
        static SIMPLE: &str = include_str!("../tests/configs/simple.yaml");
        let provider = Yaml::string(SIMPLE);
        let config = Config::from(provider).expect("must parse config");
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

        assert_eq!(
            config,
            Config {
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
        static BROKEN: &str = include_str!("../tests/configs/broken_rule.yaml");
        let provider = Yaml::string(BROKEN);
        assert_eq!(
            Config::from(provider).expect_err("must fail"),
            ConfigError::Rule(String::from("a"))
        );
    }

    #[test]
    fn must_fail_broken_dir() {
        static BROKEN: &str = include_str!("../tests/configs/broken_dir.yaml");
        let provider = Yaml::string(BROKEN);
        assert_eq!(
            Config::from(provider).expect_err("must fail"),
            ConfigError::NotADirectory(cwd_path!("tests/mock_dirs/some_file"))
        );
    }

    #[test]
    fn must_fail_missing_dir() {
        static BROKEN: &str = include_str!("../tests/configs/missing_dir.yaml");
        let provider = Yaml::string(BROKEN);
        assert_eq!(
            Config::from(provider).expect_err("must fail"),
            ConfigError::NotFound(String::from("tests/mock_dirs/non_exist"))
        );
    }

    #[test]
    fn must_allow_missing_skip_dir() {
        static BROKEN: &str = include_str!("../tests/configs/allow_missing_skip_dir.yaml");
        let provider = Yaml::string(BROKEN);
        assert_eq!(
            Config::from(provider).expect("must parse config"),
            Config {
                directories: vec![],
                skips: HashSet::new(),
            }
        );
    }
}
