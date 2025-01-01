use std::path::PathBuf;
use std::env::{self, VarError};

#[derive(Debug)]
pub struct Globals {
    pub cache_path: PathBuf,
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

        Ok(Globals {
            cache_path,
        })
    }
}
