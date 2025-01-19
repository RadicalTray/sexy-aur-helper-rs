use crate::alpm::get_local_aur_pkgs;
use crate::build;
use crate::globals::Globals;
use alpm::Alpm;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    generated: Option<bool>,
    upgrade: UpgradeConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpgradeConfig {
    install: InstallConfig,
    ignore: InstallConfig,
}

const VALID_PACKAGE_KEYS: &[&str] = &["name"];

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallConfig {
    packages: toml::value::Array,
}

impl Config {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        let config_content = fs::read_to_string("example.toml").unwrap();
        toml::from_str(&config_content).unwrap()
    }

    pub fn check_config(&self) -> Result<(), String> {
        if self.generated == Some(true) {
            return Err("Remove or set `generated` key to false to use this config.".to_string());
        }
        self.upgrade.install.check_array("install")?;
        self.upgrade.ignore.check_array("upgrade")?;

        Ok(())
    }

    pub fn gen_config(alpm_handle: &Alpm, g: &Globals) {
        let (aur_pkgs, _err_pkgs) = get_local_aur_pkgs(alpm_handle, g);
        let build_stack = build::setup_build_stack(&aur_pkgs);
        let mut pkg_set = HashSet::new();
        let pkgs = build_stack
            .into_iter()
            .rev()
            .filter(|x| {
                let name = x.name();
                if !pkg_set.contains(name) {
                    pkg_set.insert(name);
                    return true;
                }
                false
            })
            .map(|x| toml::Value::try_from(x.name()).unwrap())
            .collect();
        let config = Config {
            generated: Some(true),
            upgrade: UpgradeConfig {
                install: InstallConfig { packages: pkgs },
                ignore: InstallConfig {
                    packages: [].to_vec(),
                },
            },
        };
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&g.config_path, content).unwrap();
    }
}

impl InstallConfig {
    fn check_array(&self, name: &str) -> Result<(), String> {
        for member in &self.packages {
            match member {
                toml::Value::String(_) => continue,
                toml::Value::Table(tbl) => {
                    if !tbl.contains_key("name") {
                        eprintln!("A table member in {name}.packages doesn't contain `name` key");
                        return Err("invalid config.".to_string());
                    }

                    if tbl
                        .keys()
                        .any(|k| !VALID_PACKAGE_KEYS.contains(&k.as_str()))
                    {
                        eprintln!("Unexpected key of a table member in {name}.packages");
                        return Err("invalid config.".to_string());
                    }
                }
                _ => {
                    eprintln!("Unexpected member in {name}.packages");
                    return Err("invalid config.".to_string());
                }
            }
        }
        Ok(())
    }
}
