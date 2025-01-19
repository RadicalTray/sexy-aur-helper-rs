use crate::alpm::get_local_aur_pkgs;
use crate::build;
use crate::globals::Globals;
use alpm::{Alpm, Package};
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

pub struct IgnoreConfig {
    packages: Vec<String>,
}

impl Config {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Self {
        let config_content = fs::read_to_string(file_path).unwrap();
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

    pub fn check_pkgs(&self, local_pkgs: &Vec<&Package>) -> Result<(), ()> {
        let mut err = false;
        let install_packages = &self.upgrade.install.packages;
        let ignore_packages = &self.upgrade.ignore.packages;
        let val_to_name = |x| -> String {
            match x {
                toml::Value::String(s) => s,
                toml::Value::Table(t) => match &t["name"] {
                    toml::Value::String(rs) => rs.to_string(),
                    _ => panic!("What the hell!?"),
                },
                _ => panic!("What the hell!?"),
            }
        };

        for pkg in local_pkgs {
            let pkg_name = pkg.name();
            if !install_packages
                .iter()
                .any(|x| val_to_name(x.clone()) == pkg_name)
                && !ignore_packages
                    .iter()
                    .any(|x| val_to_name(x.clone()) == pkg_name)
            {
                eprintln!("Package `{}` not in config!", pkg_name);
                err = true;
            }
        }

        // Does this really need to be checked?
        let pkgs_in_config = install_packages.iter().chain(ignore_packages);
        for pkg in pkgs_in_config {
            let pkg_name = val_to_name(pkg.clone());
            if !local_pkgs.iter().any(|x| x.name() == pkg_name) {
                eprintln!("Package `{}` in config not in package database!", pkg_name);
                err = true;
            }
        }

        if err {
            return Err(());
        }
        Ok(())
    }

    pub fn gen_config(alpm_handle: &Alpm, g: &Globals) {
        let (base_pkgs, not_base_pkgs, _err_pkgs) = get_local_aur_pkgs(alpm_handle, g);
        let build_stack = build::setup_build_stack(&base_pkgs);
        let mut pkg_set = HashSet::new();
        let base_pkgs = build_stack
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
        let not_base_pkgs = not_base_pkgs
            .iter()
            .map(|x| toml::Value::try_from(x.name()).unwrap())
            .collect();
        let config = Config {
            generated: Some(true),
            upgrade: UpgradeConfig {
                install: InstallConfig {
                    packages: base_pkgs,
                },
                ignore: InstallConfig {
                    packages: not_base_pkgs,
                },
            },
        };
        let content = toml::to_string_pretty(&config).unwrap();
        let mut file_path = g.config_path.clone();
        if file_path.exists() {
            file_path.set_file_name("config.toml.generated");
        }
        fs::write(&file_path, content).unwrap();
        println!("Config file generated at {}", file_path.display());
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
