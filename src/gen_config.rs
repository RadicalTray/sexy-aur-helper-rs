use crate::config::Config;
use crate::globals::Globals;
use alpm::Alpm;
use std::fs;

pub const STR: &str = "gen-config";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    let config_path = &g.config_path;

    if args.len() > 0 {
        return Err("Unexpected arguments".to_string());
    }
    if !config_path.parent().unwrap().exists() {
        fs::create_dir(config_path.parent().unwrap()).unwrap();
    }

    let handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    Config::gen_config(&handle, &g);

    Ok(())
}
