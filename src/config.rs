use std::collections::HashMap;
use std::path::PathBuf;

use figment::{Figment, Provider};
use itertools::Itertools;
use serde::Deserialize;

use crate::errors::ConfigError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    pub directories: HashMap<String, Directory>, // TODO deal with subdir relationship?
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

impl Config {
    pub fn from(provider: impl Provider) -> Result<Self, ConfigError> {
        let pre_config = PreConfig::from(provider)?;
        Ok(Self {
            directories: pre_config
                .directories
                .into_iter()
                .map(|(name, pre_directory)| {
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
                            pre_directory
                                .path
                                .canonicalize()
                                .map_err(|_| ConfigError::NotADirectory(pre_directory.path.clone()))
                                .and_then(|path| {
                                    path.is_dir()
                                        .then(|| path)
                                        .ok_or(ConfigError::NotADirectory(pre_directory.path))
                                })
                                .map(|path| (path, rules))
                        })
                        .map(|(path, rules)| {
                            // compose directory
                            (name, Directory { path, rules })
                        })
                })
                .try_collect()?,
        })
    }
}

#[derive(Deserialize)]
struct PreConfig {
    #[serde(default)]
    directories: HashMap<String, PreDirectory>,
    #[serde(default)]
    rules: HashMap<String, Rule>,
}

#[derive(Deserialize)]
struct PreDirectory {
    path: PathBuf,
    rules: Vec<String>,
}

impl PreConfig {
    fn from(provider: impl Provider) -> Result<Self, figment::Error> {
        Figment::from(provider).extract()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use std::str::FromStr;

    use figment::providers::{Data, Format, Yaml};
    use maplit::hashmap;

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
                directories: hashmap! {
                    String::from("directory_a") => Directory {
                        path: cwd_path!("tests/mock_dirs/path_a"),
                        rules: vec![rule_a.clone(), rule_b.clone()]
                    },
                    String::from("directory_b") => Directory {
                        path: cwd_path!("tests/mock_dirs/path_b"),
                        rules: vec![rule_b.clone(), rule_d.clone()]
                    },
                }
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
            ConfigError::NotADirectory(path!("tests/mock_dirs/some_file"))
        );
    }
}
