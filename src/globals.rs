use std::env::{self, VarError};
use std::path::PathBuf;

pub const URL_AUR: &str = "https://aur.archlinux.org";
pub const URL_PKGBASE: &str = "https://aur.archlinux.org/pkgbase.gz";
pub const URL_PKG: &str = "https://aur.archlinux.org/packages.gz";
pub const FILENAME_PKG: &str = "packages";
pub const FILENAME_PKGBASE: &str = "pkgbase";

#[derive(Debug)]
pub struct Globals {
    pub cache_path: PathBuf,
    pub config_path: PathBuf,
}

impl Globals {
    pub fn build() -> Result<Globals, &'static str> {
        let cache_path = match env::var("XDG_CACHE_HOME") {
            Ok(val) => val,
            Err(VarError::NotPresent) => {
                let mut home = match env::var("HOME") {
                    Ok(val) => val,
                    Err(VarError::NotPresent) => return Err("$HOME is not set"),
                    Err(VarError::NotUnicode(_)) => return Err("Invalid $HOME value"),
                };

                home.push_str("/.cache");

                home
            }
            Err(VarError::NotUnicode(_)) => return Err("Invalid $XDG_CACHE_HOME value"),
        };
        let cache_path = PathBuf::from(cache_path).join("saur");

        let config_path = match env::var("XDG_CONFIG_HOME") {
            Ok(val) => val,
            Err(VarError::NotPresent) => {
                let mut home = match env::var("HOME") {
                    Ok(val) => val,
                    Err(VarError::NotPresent) => return Err("$HOME is not set"),
                    Err(VarError::NotUnicode(_)) => return Err("Invalid $HOME value"),
                };

                home.push_str("/.config");

                home
            }
            Err(VarError::NotUnicode(_)) => return Err("Invalid $XDG_CONFIG_HOME value"),
        };
        let config_path = PathBuf::from(config_path).join("saur").join("config.toml");

        Ok(Globals {
            cache_path,
            config_path,
        })
    }
}
