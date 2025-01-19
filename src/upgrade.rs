use crate::alpm::get_local_foreign_pkgs;
use crate::config::Config;
use crate::gen_config;
use crate::globals::Globals;
use crate::utils::prepare_clone;
use alpm::Alpm;
use std::process;

pub const STR: &str = "upgrade";

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    prepare_clone(&g);

    if args.len() != 0 {
        return Err("unexpected arguments.".to_string());
    }

    // let status = Pacman::Syu_status();
    // if !status.success() {
    //     process::exit(status.code().unwrap());
    // }

    let config_path = g.config_path;
    if !config_path.exists() {
        eprintln!("Run `saur {}` to generate config", gen_config::STR);
        return Err("config file doesn't exist.".to_string());
    }

    let config: Config = Config::new(config_path);
    config.check_config()?;

    let handle = Alpm::new("/", "/var/lib/pacman").unwrap();
    let local_foreign_pkgs = get_local_foreign_pkgs(&handle);
    if let Err(_) = config.check_pkgs(&local_foreign_pkgs) {
        process::exit(1);
    }

    Ok(())
}
