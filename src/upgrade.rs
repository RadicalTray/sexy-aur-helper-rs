use crate::alpm::get_local_foreign_pkgs;
use crate::build::PkgInfo;
use crate::build::build;
use crate::config::{Config, val_to_name};
use crate::fetch::fetch_pkgs;
use crate::gen_config;
use crate::globals::Globals;
use crate::pacman::Pacman;
use crate::utils::{prepare_clone, prompt_accept};
use crate::ver::get_pkgs_to_upgrade;
use alpm::{Alpm, Package, Version};
use std::path::PathBuf;
use std::process;

pub const STR: &str = "upgrade";

// PERF: a lot of clones with val_to_name

pub fn run(g: Globals, args: Vec<String>) -> Result<(), String> {
    prepare_clone(&g);
    let config_path = g.config_path;
    let clone_path = g.cache_path.join("clone");

    if args.len() != 0 {
        return Err("unexpected arguments.".to_string());
    }

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

    let status = Pacman::Syu_status();
    // BUG: if CTRL+C (and maybe other signal), the sudo will later show output
    // after process has left messing with user's prompt
    if !status.success() {
        process::exit(status.code().unwrap());
    }

    let infos = get_versions(local_foreign_pkgs, config.upgrade.install.packages);

    let (mut old_pkgs, new_pkgs, fetch_err_pkgs) = fetch_pkgs(&clone_path, infos.clone()); // damn
    if fetch_err_pkgs.len() > 0 {
        for (pkg, _) in fetch_err_pkgs {
            let pkg_name = pkg.name;
            eprintln!("{pkg_name}: error while fetching AUR!");
        }
        eprintln!();
    }

    old_pkgs.extend(new_pkgs);
    let fetched_pkgs = old_pkgs;

    let (pkgs_to_upgrade, upgrade_err_pkgs) = get_pkgs_to_upgrade(&clone_path, fetched_pkgs);
    if upgrade_err_pkgs.len() > 0 {
        for pkg in upgrade_err_pkgs {
            eprintln!("{pkg}: error while fetching package source!");
        }
        eprintln!();
    }

    let mut infos = infos;
    println!("All shits: {:#?}", infos);
    infos.retain(|(x, _)| pkgs_to_upgrade.contains(&x.name));
    let pkgs_to_upgrade = infos;

    println!("Ignore:");
    for pkg in config.upgrade.ignore.packages {
        println!("\t{}", val_to_name(pkg.clone()));
    }
    println!();

    if pkgs_to_upgrade.len() == 0 {
        println!("Nothing to upgrade.");
        return Ok(());
    }

    println!("{} packages to be upgraded:", pkgs_to_upgrade.len());
    for (pkg, _) in &pkgs_to_upgrade {
        println!("\t{}", pkg.name);
    }
    prompt_accept();

    build_and_install(&clone_path, pkgs_to_upgrade);

    Ok(())
}

fn get_versions(
    local_pkgs: Vec<&Package>,
    pkg_list: toml::value::Array,
) -> Vec<(PkgInfo, Option<Version>)> {
    pkg_list
        .into_iter()
        .map(|x| {
            let x_name = val_to_name(x.clone());
            let pkg_from_db = local_pkgs.iter().find(|y| y.name() == x_name).unwrap();
            let version = pkg_from_db.version();
            let pkg_info = PkgInfo::new(x_name);
            (pkg_info, Some(Version::new(version.as_str())))
        })
        .collect()
}

fn build_and_install(clone_path: &PathBuf, pkgs: Vec<(PkgInfo, Option<Version>)>) {
    let mut err_pkgs = Vec::new();
    let mut fail = false;
    for (pkg, _) in pkgs {
        let pkg_name = pkg.name;
        let cwd = clone_path.clone().join(&pkg_name);
        let install_infos = match build(cwd, true, PkgInfo::new(pkg_name.clone())) {
            Ok(v) => v,
            Err(pkg) => {
                err_pkgs.push(pkg);
                continue;
            }
        };

        for install_info in install_infos {
            let s = Pacman { yes: true }.U_all_status(&install_info).success();
            if !s {
                fail = true;
                break;
            }
        }
        if fail {
            eprintln!("Error while installing: {}", pkg_name);
            break;
        }
    }

    if err_pkgs.len() > 0 {
        eprintln!("Error while building:");
        for pkg in err_pkgs {
            eprintln!("\t{pkg}");
        }
        process::exit(1);
    }
    if fail {
        process::exit(1);
    }
}
