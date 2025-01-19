use crate::config::Config;
use crate::gen_config;
use crate::globals::Globals;
use std::{fs, process};

pub const STR: &str = "upgrade";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    let config_path = g.config_path;

    if args.len() != 0 {
        return Err("unexpected arguments.".to_string());
    }

    // if !config_path.exists() {
    //     eprintln!("Run `saur {}` to generate config", gen_config::STR);
    //     return Err("config file doesn't exist.".to_string());
    // }

    let config: Config = Config::new("example.toml");
    config.check_config()?;

    Ok(())
}
